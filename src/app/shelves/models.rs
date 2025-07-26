use crate::app::state::service::unix_millis_to_string;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct NewShelfResponse {
    pub new_tag: ShelfResponse,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DeletedShelfResponse {
    pub deleted_tag: ShelfResponse,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ShelfResponse {
    pub tag: Tag,
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub id: String,
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub items: Option<Vec<ShelfItem>>,
    pub created: Option<String>,
    pub last_modified: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ShelfItem {
    pub revision_id: String,
    pub r#type: String,
}

impl NewShelfResponse {
    pub fn new(id: &str, name: &str, book_ids: &Vec<String>) -> Self {
        let now: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get time since epoch")
            .as_millis()
            .try_into()
            .expect("Failed to get current timestamp");

        let now = unix_millis_to_string(now);

        let mut items = Vec::new();
        for id in book_ids {
            let item = ShelfItem {
                revision_id: id.to_string(),
                r#type: "ProductRevisionTagItem".to_string(),
            };
            items.push(item);
        }

        let tag = Tag {
            id: id.to_string(),
            name: Some(name.to_string()),
            r#type: Some("UserTag".to_string()),
            items: Some(items),
            created: Some(now.clone()),
            last_modified: now,
        };

        NewShelfResponse {
            new_tag: ShelfResponse { tag },
        }
    }
}

impl DeletedShelfResponse {
    pub fn new(id: &str) -> Self {
        let now: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get time since epoch")
            .as_millis()
            .try_into()
            .expect("Failed to get current timestamp");

        let now = unix_millis_to_string(now);

        let tag = Tag {
            id: id.to_string(),
            name: None,
            r#type: None,
            items: None,
            created: None,
            last_modified: now,
        };

        DeletedShelfResponse {
            deleted_tag: ShelfResponse { tag },
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CreateShelfRequest {
    pub items: Vec<ShelfItem>,
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RenameShelfRequest {
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AddBooksToShelfRequest {
    pub items: Vec<ShelfItem>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteBooksFromShelfRequest {
    pub items: Vec<ShelfItem>,
}
