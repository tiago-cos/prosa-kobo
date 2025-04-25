use super::service::random_string;
use crate::{app::state::service::unix_millis_to_string, client::ProsaMetadata};
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BookMetadata {
    pub cross_revision_id: String,
    pub revision_id: String,
    pub publisher: Publisher,
    pub publication_date: Option<String>,
    pub language: Option<String>,
    pub isbn: Option<String>,
    pub subtitle: Option<String>,
    pub genre: Option<String>,
    pub slug: Option<String>,
    pub cover_image_id: String,
    pub is_social_enabled: bool,
    pub work_id: String,
    pub external_ids: Vec<String>,
    pub is_pre_order: bool,
    pub contributor_roles: Vec<Contributor>,
    pub is_internet_archive: bool,
    pub is_annotation_export_disabled: bool,
    pub is_ai_summary_disabled: bool,
    pub entitlement_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub categories: Vec<String>,
    pub download_urls: Vec<DownloadUrl>,
    pub contributors: Vec<String>,
    pub series: Option<Series>,
    pub current_display_price: CurrentDisplayPrice,
    pub current_love_display_price: CurrentLoveDisplayPrice,
    pub is_eligible_for_kobo_love: bool,
    pub phonetic_pronunciations: Option<String>,
    pub related_group_id: Option<String>,
    pub locale: Locale,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Publisher {
    name: Option<String>,
    imprint: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Contributor {
    name: String,
    role: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DownloadUrl {
    drm_type: String,
    format: String,
    url: String,
    platform: String,
    size: u64,
}

impl DownloadUrl {
    pub fn new(download_url: &str, download_size: u64) -> Self {
        DownloadUrl {
            drm_type: "None".to_string(),
            format: "KEPUB".to_string(),
            url: download_url.to_string(),
            platform: "Generic".to_string(),
            size: download_size,
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Series {
    pub id: String,
    pub name: String,
    pub number: String,
    pub number_float: f32,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CurrentDisplayPrice {
    pub total_amount: i64,
    pub currency_code: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CurrentLoveDisplayPrice {
    pub total_amount: i64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Locale {
    pub language_code: String,
    pub script_code: String,
    pub country_code: String,
}

impl BookMetadata {
    pub fn new(book_id: &str, metadata: ProsaMetadata, download_url: DownloadUrl) -> Self {
        let publisher = Publisher {
            name: metadata.publisher.clone(),
            imprint: metadata.publisher,
        };

        let contributors: Vec<Contributor> = metadata
            .contributors
            .unwrap_or_default()
            .into_iter()
            .map(|c| Contributor {
                name: c.name,
                role: c.role,
            })
            .collect();

        let series = metadata.series.map(|s| Series {
            id: s.title.clone(),
            name: s.title,
            number: s.number.to_string(),
            number_float: s.number,
        });

        let current_display_price = CurrentDisplayPrice {
            total_amount: -1,
            currency_code: "".to_string(),
        };

        let current_love_display_price = CurrentLoveDisplayPrice { total_amount: 0 };

        let locale = Locale {
            language_code: metadata.language.clone().unwrap_or("eng".to_string()),
            script_code: "".to_string(),
            country_code: "".to_string(),
        };

        BookMetadata {
            cross_revision_id: book_id.to_string(),
            revision_id: book_id.to_string(),
            publisher,
            publication_date: metadata.publication_date.map(unix_millis_to_string),
            language: metadata.language,
            isbn: metadata.isbn,
            subtitle: metadata.subtitle,
            genre: None,
            slug: None,
            cover_image_id: format!("{}[[{}]]", book_id, random_string(6)),
            is_social_enabled: true,
            work_id: book_id.to_string(),
            external_ids: Vec::new(),
            is_pre_order: false,
            contributor_roles: contributors.clone(),
            is_internet_archive: false,
            is_annotation_export_disabled: false,
            is_ai_summary_disabled: false,
            entitlement_id: book_id.to_string(),
            title: metadata.title,
            description: metadata.description,
            categories: Vec::new(),
            download_urls: vec![download_url],
            contributors: contributors.iter().map(|c| c.name.clone()).collect(),
            series,
            current_display_price,
            current_love_display_price,
            is_eligible_for_kobo_love: false,
            phonetic_pronunciations: None,
            related_group_id: None,
            locale,
        }
    }
}

impl Default for BookMetadata {
    fn default() -> Self {
        let metadata_response = ProsaMetadata::default();
        let download_url = DownloadUrl::new("placeholder", 1);
        BookMetadata::new("placeholder", metadata_response, download_url)
    }
}
