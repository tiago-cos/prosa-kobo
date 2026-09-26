use serde::Deserialize;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ProsaSync {
    pub new_sync_token: i64,
    pub unsynced_books: ProsaBookSync,
    pub unsynced_shelves: ProsaShelfSync,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ProsaBookSync {
    pub file: Vec<String>,
    pub metadata: Vec<String>,
    pub cover: Vec<String>,
    pub state: Vec<String>,
    pub annotations: Vec<String>,
    pub deleted: Vec<String>,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ProsaShelfSync {
    pub metadata: Vec<String>,
    pub contents: Vec<String>,
    pub deleted: Vec<String>,
}
