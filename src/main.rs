use chrono::Days;
use chrono::Local;
use chrono::NaiveDate;
use reqwest::Client;
use reqwest::StatusCode;
use scraper::selectable::Selectable;
use scraper::{Html, Selector};
use sqlx::PgPool;
use std::thread::sleep as std_sleep;
use tokio::time::sleep as tokio_sleep;
use tokio::time::Duration;
use tracing::debug;
use tracing::error;

struct CustomService {
    ctx: Client,
    db: PgPool,
}

// Set up our user agent
const USER_AGENT: &str = "Mozilla/5.0 (Linux x86_64; rv:115.0) Gecko/20100101 Firefox/115.0";

// note that we add our Database as an annotation here so we can easily get it provisioned to us

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] db: PgPool,
) -> Result<CustomService, shuttle_runtime::Error> {
    // automatically attempt to do migrations
    // we only create the table if it doesn't exist which prevents data wiping
    sqlx::migrate!().run(&db).await.expect("Migrations failed");
    // initialise Reqwest client here so we can add it in later on
    let ctx = Client::builder().user_agent(USER_AGENT).build().unwrap();
    Ok(CustomService { ctx, db })
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for CustomService {
    async fn bind(mut self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        scrape(self.ctx, self.db)
            .await
            .expect("scraping should not finish");
        error!("The web scraper loop shouldn't finish!");
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Product {
    name: String,
    price: String,
    old_price: Option<String>,
    link: String,
    // scraped_at: NaiveDate,
}

async fn scrape(ctx: Client, db: PgPool) -> Result<(), String> {
    loop {
        let mut vec: Vec<Product> = Vec::new();
        let mut pagenum = 1;
        let mut retry_attempts = 0;

        loop {
            let url = format!("https://www.amazon.com/s?k=raspberry+pi&page={pagenum}");

            let res = match ctx.get(url).send().await {
                Ok(res) => res,
                Err(e) => {
                    error!("Error while attempting to send HTTP request: {e}");
                    break;
                }
            };

            if res.status() == StatusCode::SERVICE_UNAVAILABLE {
                error!("Amazon returned a 503 at page {pagenum}");
                retry_attempts += 1;
                if retry_attempts >= 10 {
                    // take a break if too many retry attempts
                    error!("It looks like Amazon is blocking us! We will rest for an hour.");
                    // sleep for an hour then retry on current iteration
                    std_sleep(Duration::from_secs(3600));
                    continue;
                } else {
                    std_sleep(Duration::from_secs(15));
                    continue;
                }
            }
            let body = match res.text().await {
                Ok(res) => res,
                Err(e) => {
                    error!("Error while attempting to get the HTTP body: {e}");
                    break;
                }
            };

            debug!("Page {pagenum} was scraped");

            let html = Html::parse_fragment(&body);
            let selector = Selector::parse("div[data-component-type='s-search-result']").unwrap();

            if html.select(&selector).count() == 0 {
                error!("There's nothing to parse here!");
                break;
            };

            html.select(&selector).for_each(|entry| {
                // declaring more Selectors to use on each entry
                let price_selector = Selector::parse("span.a-price > span.a-offscreen").unwrap();
                let productname_selector = Selector::parse("h2 > a").unwrap();

                let name = entry
                    .select(&productname_selector)
                    .next()
                    .expect("Couldn't find the product name")
                    .text()
                    .next()
                    .unwrap()
                    .to_string();

                // Amazon products can have two prices : a current price, and an "old price".
                // We iterate through both of these and map them to a Vec<String>.
                let price_text = entry
                    .select(&price_selector)
                    .map(|x| x.text().next().unwrap().to_string())
                    .collect::<Vec<String>>();

                // get local date from chrono for database storage purposes
                let scraped_at = Local::now().date_naive();

                // here we find the anchor element and find the value of the href attribute - this should always exist so we can safely unwrap
                let link = entry
                    .select(&productname_selector)
                    .map(|link| {
                        format!("https://amazon.co.uk{}", link.value().attr("href").unwrap())
                    })
                    .collect::<String>();

                vec.push(Product {
                    name,
                    price: price_text[0].clone(),
                    old_price: Some(price_text[1].clone()),
                    link,
                    // scraped_at,
                });
            });
            pagenum += 1;
            retry_attempts = 0;
            std_sleep(Duration::from_secs(20));
        }

        let transaction = db.begin().await.unwrap();

        for product in vec {
            if let Err(e) = sqlx::query(
                "INSERT INTO 
        products
        (name, price, old_price, link, scraped_at)
        VALUES
        ($1, $2, $3, $4, $5)",
            )
            .bind(product.name)
            .bind(product.price)
            .bind(product.old_price)
            .bind(product.link)
            // .bind(product.scraped_at)
            .execute(&db)
            .await
            {
                error!("There was an error: {e}");
                error!("This web scraper will now shut down.");
                // transaction.rollback().await.unwrap();
                break;
            }
        }
        transaction.commit().await.unwrap();

        // get the local time, add a day then get the NaiveDate and set a time of 00:00 to it
        let tomorrow_midnight = Local::now()
            .checked_add_days(Days::new(1))
            .unwrap()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        // get the local time now
        let now = Local::now().naive_local();

        // check the amount of time between now and midnight tomorrow
        let duration_to_midnight = tomorrow_midnight
            .signed_duration_since(now)
            .to_std()
            .unwrap();

        // sleep for the required time
        tokio_sleep(Duration::from_secs(duration_to_midnight.as_secs())).await;
    }
    Ok(())
}
