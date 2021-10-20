use rusqlite::{params, Connection, Result};

use crate::book::{Book, DataBook};

pub fn init_db(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "create table if not exists books (
            id integer primary key,
            title text not null,
            author text not null,
            pages integer,
            open_work_key text,
            open_edition_key text
        )",
        [],
    )?;
    conn.execute(
        "create table if not exists book_subjects (
            id integer primary_key,
            book_id integer not null references books(id),
            subject text not null
        )",
        [],
    )?;

    conn.execute(
        "create unique index if not exists 
             idx_title_author on books (title,author)",
        [],
    )?;

    Ok(conn)
}

pub fn get_book(conn: &Connection, book: &Book) -> Result<i64> {
    let mut stmt = conn.prepare(
        "SELECT id FROM books
             where title = (?1)
               and author = (?2)",
    )?;

    let book_id = stmt.query_row(params![book.title, book.author], |row| Ok(row.get(0)?))?;

    Ok(book_id)
}

pub async fn insert_book(conn: &Connection, book: &DataBook) -> Result<i64> {
    let book_id = get_book(conn, &book.book);

    match book_id {
        Ok(s) => return Ok(s),
        Err(rusqlite::Error::QueryReturnedNoRows) => {}
        Err(x) => return Err(x),
    }

    conn.execute(
        "INSERT INTO books (
                title,
                author,
                pages, 
                open_work_key, 
                open_edition_key
            ) 
            values (?1, ?2, ?3, ?4, ?5)",
        params![
            book.book.title,
            book.book.author,
            book.pages.unwrap_or_default(),
            book.open_work_key,
            book.open_edition_key,
        ],
    )?;
    let last_id = conn.last_insert_rowid();

    let subjects: Vec<String> = match serde_json::from_str(&book.subjects) {
        Ok(s) => s,
        Err(_) => vec![],
    };
    for subject in subjects {
        conn.execute(
            "insert into book_subjects (book_id, subject) values (?1, ?2)",
            params![last_id, subject],
        )?;
    }

    Ok(last_id)
}
