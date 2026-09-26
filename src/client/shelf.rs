use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaShelfMetadata {
    pub name: String,
    pub owner_id: String,
    pub book_count: u64,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaShelfSearch {
    pub shelf_ids: Vec<String>,
    pub total_pages: u64,
    pub current_page: u64,
}

#[derive(Serialize, Debug)]
pub(super) struct ProsaShelfUpdateRequest<'a> {
    pub(super) name: &'a str,
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
pub(super) struct ProsaShelfCreateRequest<'a> {
    pub(super) name: &'a str,
    pub(super) owner_id: Option<&'a str>,
    pub(super) shelf_id: Option<&'a str>,
}

#[derive(Serialize, Debug)]
pub(super) struct ProsaAddBookShelfRequest<'a> {
    pub(super) book_id: &'a str,
}
