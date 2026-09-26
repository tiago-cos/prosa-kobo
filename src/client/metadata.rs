use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaContributor {
    pub name: String,
    pub role: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct ProsaSeries {
    pub title: String,
    pub number: Option<f32>,
}

#[derive(Deserialize, Clone, Default, Debug, PartialEq)]
pub struct ProsaMetadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub publication_date: Option<i64>,
    pub isbn: Option<String>,
    pub contributors: Option<Vec<ProsaContributor>>,
    pub genres: Option<Vec<String>>,
    pub series: Option<ProsaSeries>,
    pub page_count: Option<i64>,
    pub language: Option<String>,
}
