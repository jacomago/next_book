use std::borrow::Borrow;
use std::{error::Error, fs};

use csv::WriterBuilder;
use serde::Deserialize;
use serde::Serialize;
use serde_json::value::Value;
use tokio;

#[derive(Deserialize, Serialize, Debug)]
struct Book {
    author: String,
    title: String,
    subjects: String,
    pages: Option<i64>,
    open_work_key: String,
    open_edition_key: String,
}

#[derive(Deserialize, Debug)]
struct InputBook {
    list06: String,
    list08: String,
    list10: String,
    list12: String,
    list18: String,
    randcol: String,
    number: String,
    title: String,
    author: String,
    list: String,
    orig_title: String,
    wilson: String,
    nation: String,
    period: String,
}

struct Config {
    input_filename: String,
    output_filename: String,
    temp_in_filename: String,
    temp_out_filename: String
}

#[tokio::main]
async fn main() {
    let config = Config{
        input_filename: "resources/1001books.csv".to_string(),
        output_filename: "resources/output.csv".to_string(),
        temp_in_filename: "resources/temp_input.csv".to_string(),
        temp_out_filename: "resources/temp_output.csv".to_string()
    };
    let data = process_file(&config)
        .await
        .expect("fail on read");
    export_file(&config.output_filename, data).expect("fail writing");
}

fn read_temp_file_keys(temp_filename: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let file = fs::read_to_string(temp_filename).expect("failed to read temp file");

    let mut contents = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_reader(file.as_bytes());

    let mut output = vec![];
    for result in contents.deserialize() {
        let record: Book = result?;
        let output_record = record.title;
        output.push(output_record);
    }
    Ok(output)
}

async fn process_file(
    config: &Config
) -> Result<Vec<Book>, Box<dyn Error>> {
    let file = fs::read_to_string(&config.input_filename).expect("failed to read file");

    let mut contents = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file.as_bytes());

    let temp_file_keys = read_temp_file_keys(&config.temp_in_filename).expect("get keys ok");

    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .flexible(true)
        .from_path(&config.temp_out_filename)?;

    let mut output = vec![];
    for result in contents.deserialize() {
        let record: InputBook = result?;
        if !temp_file_keys.contains(&record.title) {
            let output_record = convert_input(&record).await?;
            wtr.serialize(output_record.borrow())?;
            output.push(output_record);
        }
        wtr.flush()?;
    }
    wtr.flush()?;
    Ok(output)
}

async fn convert_input(input: &InputBook) -> Result<Book, Box<dyn Error>> {
    let key = get_book_key(input).await?;
    if key.work_key.eq("") {
        return Ok(Book {
            title: key.title.to_string(),
            author: key.author.to_string(),
            subjects: "".to_string(),
            pages: None,
            open_work_key: key.work_key.to_string(),
            open_edition_key: key.edition_key.to_string(),
        })
    }
    get_book(&key).await
}

struct Key {
    title: String,
    author: String,
    work_key: String,
    edition_key: String,
}

async fn get_book_key(input: &InputBook) -> Result<Key, Box<dyn Error>> {
    let converted_title = query_string(&input.title);
    let converted_author = query_string(&input.author);
    let request_url = format!(
        "https://openlibrary.org/search.json?title={title}&author={author}&fields=key,edition_key&limit=1",
        title = converted_title,
        author = converted_author
    );

    println!("url for getting key: {:?}", request_url);
    let response = reqwest::get(&request_url).await?;
    let json: Value = response.json().await?;

    let key_value = &json["docs"].as_array().unwrap();
    if key_value.is_empty() {
        return Ok(Key {
            title: input.title.to_string(),
            author: input.author.to_string(),
            work_key: "".to_string(),
            edition_key: "".to_string(),
        });
    }

    let work_key = key_value[0]["key"]
        .as_str()
        .expect(&format!("fail on work_key title: {:?}", &input.title))
        .to_string();
    let edition_key = key_value[0]["edition_key"].as_array().unwrap()[0]
        .as_str()
        .expect(&format!("fail on work_key title: {:?}", &input.title))
        .to_string();

    Ok(Key {
        title: input.title.to_string(),
        author: input.author.to_string(),
        work_key,
        edition_key,
    })
}

async fn get_book(key: &Key) -> Result<Book, Box<dyn Error>> {
    let request_url = format!("https://openlibrary.org{key}.json", key = key.work_key);
    println!("request url; {:?}", request_url);

    let response = reqwest::get(&request_url).await?;
    let json: Value = response.json().await?;

    let pages = get_pages(&key.edition_key).await?;

    Ok(Book {
        title: key.title.to_string(),
        author: key.author.to_string(),
        subjects: json["subjects"].to_string(),
        pages,
        open_work_key: key.work_key.to_string(),
        open_edition_key: key.edition_key.to_string(),
    })
}

async fn get_pages(key: &str) -> Result<Option<i64>, Box<dyn Error>> {
    let request_url = format!("https://openlibrary.org/books/{key}.json?", key = key);

    let response = reqwest::get(&request_url).await?;
    let json: Value = response.json().await?;
    Ok(json["number_of_pages"].as_i64())
}

fn query_string(input: &str) -> String {
    input
        .to_lowercase()
        .replace(" ", "+")
        .replace("'", "")
        .replace("’", "")
}

fn export_file(filename: &str, records: Vec<Book>) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new().from_path(filename)?;
    for record in records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}
