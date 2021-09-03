use std::{error::Error, fs};

use csv::Writer;
use serde::Deserialize;
use serde::Serialize;
use serde_json::value::Value;
use tokio;

#[derive(Deserialize, Serialize, Debug)]
struct Book {
    author: String,
    title: String,
    subjects: Vec<String>,
    pages: Option<i64>,
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

#[tokio::main]
async fn main() {
    let data = import_file("resources/1001books.csv")
        .await
        .expect("fail on read");
    export_file("ouput.csv", data).expect("fail writing");
}

async fn import_file(filename: &str) -> Result<Vec<Book>, Box<dyn Error>> {
    let file = fs::read_to_string(filename).expect("couldn't read file");
    let mut contents = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file.as_bytes());
    let mut output = vec![];
    for result in contents.deserialize() {
        let record: InputBook = result?;

        output.push(convert_input(&record).await?);
    }
    Ok(output)
}

async fn convert_input(input: &InputBook) -> Result<Book, Box<dyn Error>> {
    let key = get_book_key(input).await?;
    get_book(&key).await
}

struct Key {
    title: String,
    author: String,
    work_key: String,
    edition_key: String,
}

async fn get_book_key(input: &InputBook) -> Result<Key, Box<dyn Error>> {
    let converted_title = query_title(&input.title);
    let request_url = format!(
        "https://openlibrary.org/search.json?title={title}&fields=key,edition_key&limit=1",
        title = converted_title
    );

    println!("current book is: {:?}", input.title);
    let response = reqwest::get(&request_url).await?;
    let json: Value = response.json().await?;
    let key_value = &json["docs"].as_array().unwrap()[0];
    let work_key = key_value["key"]
        .as_str()
        .expect(&format!("fail on work_key title: {:?}", &input.title))
        .to_string();
    let edition_key = key_value["edition_key"].as_array().unwrap()[0]
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
    let request_url = format!("https://openlibrary.org/{key}.json", key = key.work_key);

    let response = reqwest::get(&request_url).await?;
    let json: Value = response.json().await?;

    let subjects_value = json["subjects"].as_array().unwrap();
    let subjects = subjects_value
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    let pages = get_pages(&key.edition_key).await?;

    Ok(Book {
        title: key.title.to_string(),
        author: key.author.to_string(),
        subjects,
        pages,
    })
}

async fn get_pages(key: &str) -> Result<Option<i64>, Box<dyn Error>> {
    let request_url = format!("https://openlibrary.org/books/{key}.json?", key = key);

    let response = reqwest::get(&request_url).await?;
    let json: Value = response.json().await?;
    Ok(json["number_of_pages"].as_i64())
}

fn query_title(title: &str) -> String {
    title.to_lowercase().replace(" ", "+").replace("'", "")
}

fn export_file(filename: &str, records: Vec<Book>) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(filename)?;
    for record in records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}
