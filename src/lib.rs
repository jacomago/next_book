use book::Book;


pub mod db;
pub mod open_library_api;
pub mod book;


fn query_string(input: &str) -> String {
    input
        .to_lowercase()
        .replace(" ", "+")
        .replace("'", "")
        .replace("’", "")
}

pub fn input_book_key(book: &Book) -> String {
    let converted_title = query_string(&book.title);
    let converted_author = query_string(&book.author);
    format!(
        "title={title}&author={author}",
        title = converted_title,
        author = converted_author
    )
}