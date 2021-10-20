use crate::{
    book::{Book, DataBook},
    input_book_key,
};
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;

#[derive(Deserialize, Debug)]
pub struct InputBook {
    list06: String,
    list08: String,
    list10: String,
    list12: String,
    list18: String,
    randcol: String,
    number: String,
    pub title: String,
    pub author: String,
    list: String,
    orig_title: String,
    wilson: String,
    nation: String,
    period: String,
}
pub trait BookKey {
    fn book(&self) -> Book;
}
impl BookKey for InputBook {
    fn book(&self) -> Book {
        Book {
            title: self.title.to_string(),
            author: self.author.to_string(),
        }
    }
}
struct Key {
    book: Book,
    work_key: String,
    edition_key: String,
}

impl BookKey for Key {
    fn book(&self) -> Book {
        Book {
            title: self.book.title.to_string(),
            author: self.book.author.to_string(),
        }
    }
}
async fn get_book(key: &Key) -> Result<DataBook, Box<dyn Error>> {
    let request_url = format!("https://openlibrary.org{key}.json", key = key.work_key);
    println!("request url; {:?}", request_url);

    let response = reqwest::get(&request_url).await?;
    let json: Value = response.json().await?;

    let pages = get_pages(&key.edition_key).await?;

    Ok(DataBook {
        book: key.book(),
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

async fn get_book_key(input: &InputBook) -> Result<Key, Box<dyn Error>> {
    let book = input.book();
    let input_book_key = input_book_key(&book);
    let request_url = format!(
        "https://openlibrary.org/search.json?{key}&fields=key,edition_key&limit=1",
        key = input_book_key
    );

    println!("url for getting key: {:?}", request_url);
    let response = reqwest::get(&request_url).await?;
    let json: Value = response.json().await?;

    let key_value = &json["docs"].as_array().unwrap();
    if key_value.is_empty() {
        return Ok(Key {
            book,
            work_key: "".to_string(),
            edition_key: "".to_string(),
        });
    }

    let work_key = key_value[0]["key"]
        .as_str()
        .expect(&format!("fail on work_key title: {:?}", &input.title))
        .to_string();
    let edition_key = match key_value[0]["edition_key"].as_array() {
        Some(s) => s[0]
            .as_str()
            .expect(&format!("fail on work_key title: {:?}", &input.title))
            .to_string(),
        None => "".to_string(),
    };

    Ok(Key {
        book,
        work_key,
        edition_key,
    })
}

pub async fn convert_input(input: &InputBook) -> Result<DataBook, Box<dyn Error>> {
    let key = get_book_key(input).await?;
    if key.work_key.eq("") {
        return Ok(DataBook {
            book: input.book(),
            subjects: "".to_string(),
            pages: None,
            open_work_key: key.work_key.to_string(),
            open_edition_key: key.edition_key.to_string(),
        });
    }
    get_book(&key).await
}
