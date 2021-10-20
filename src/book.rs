
use serde::Deserialize;
use serde::Serialize;



#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Book {
    pub author: String,
    pub title: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DataBook {
    pub book: Book,
    pub subjects: String,
    pub pages: Option<i64>,
    pub open_work_key: String,
    pub open_edition_key: String,
}
