use std::collections::HashMap;
use std::{error::Error, fs};

use next_book::book::DataBook;
use next_book::db::{init_db, insert_book};
use next_book::input_book_key;
use next_book::open_library_api::{BookKey, InputBook, convert_input};
use rusqlite::{Connection, Result};
use tokio;

struct Config {
    input_filename: String,
    converted_input_filename: String,
    db_path: String,
}

#[tokio::main]
async fn main() {
    let config = Config {
        input_filename: "resources/1001books.csv".to_string(),
        converted_input_filename: "resources/converted_input.csv".to_string(),
        db_path: "books.db".to_string(),
    };
    let conn = init_db(&config.db_path).expect("fail on init db");
    process_files(&config, &conn).await.expect("fail on read");
}

async fn process_files(config: &Config, conn: &Connection) -> Result<(), Box<dyn Error>> {
    let input_file = fs::read_to_string(&config.input_filename).expect("failed to read file");

    let mut input_contents = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(input_file.as_bytes());

    let convert_map = get_convert_map(config).await?;

    for result in input_contents.deserialize() {
        let record: InputBook = result?;
        let input_book_key = &input_book_key(&record.book());

        match convert_map.get(input_book_key) {
            Some(b) => {
                insert_book(&conn, b).await?;
            }
            None => {
                let book = convert_input(&record).await?;

                insert_book(&conn, &book).await?;
            }
        };
    }
    Ok(())
}

async fn get_convert_map(config: &Config) -> Result<HashMap<String, DataBook>, Box<dyn Error>> {
    let input_file =
        fs::read_to_string(&config.converted_input_filename).expect("failed to read file");

    let mut contents = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_reader(input_file.as_bytes());

    let mut output = HashMap::new();

    for result in contents.deserialize() {
        let record: DataBook = result?;
        let key = input_book_key(&record.book);
        output.insert(key, record);
    }
    Ok(output)
}
