# Amazon Raspberry Web Scraper

## Overview
`amazon_raspberry_webscraper` is a Rust-based web scraping project designed to extract information about Raspberry Pi products from Amazon's website. The project utilizes `reqwest`, `tokio`, `scraper`, and `sqlx` libraries to fetch, parse, and store data from Amazon. The scraped results are saved in a SQLite database for easy access and analysis.

## Prerequisites
Before running the scraper, ensure you have the following installed:
- Rust programming language (https://www.rust-lang.org/tools/install)
- Cargo package manager (usually installed with Rust)
- SQLite database

## Installation
1. Clone the repository to your local machine:
   git clone https://github.com/yourusername/amazon_raspberry_webscraper.git
   
3. Navigate to the project directory:
   cd amazon_raspberry_webscraper
  
4. Build the project using Cargo:
   cargo build --release


## Usage
1. Run the scraper using Cargo:
   cargo run
   
2. The scraper will start fetching data from Amazon's website for Raspberry Pi products.
3. Once the scraping process is complete, the gathered data will be saved in the SQLite database (`raspberry_pi_data.db`) in the project directory.

## Project Structure
- `main.rs`: Contains the source code of the scraper.
  - The code in `main.rs` handles the entire scraping process including fetching data from Amazon, parsing HTML, and storing results in the SQLite database.

- `Cargo.toml`: Cargo configuration file specifying project dependencies and metadata.
- `README.md`: Project documentation file (this file).
- `LICENSE`: License information for the project.

## Contributing
Contributions are welcome! If you have any suggestions, improvements, or bug fixes, feel free to open an issue or submit a pull request.

## Disclaimer
This scraper is intended for educational and personal use only. Scraping data from websites may violate the terms of service of those websites. Use it responsibly and at your own risk.

## Acknowledgements
- This project is made possible by the Rust community and the developers of `reqwest`, `tokio`, `scraper`, and `sqlx` libraries.


Happy scraping! 🚀

Acknowledgements
This project is made possible by the Rust community and the developers of reqwest, tokio, scraper, and sqlx libraries.

Happy scraping! 🚀
