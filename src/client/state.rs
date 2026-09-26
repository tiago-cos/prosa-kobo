use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProsaReadingStatus {
    Unread,
    Reading,
    Read,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ProsaState {
    pub location: Option<String>,
    pub statistics: ProsaStatistics,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ProsaStatistics {
    pub rating: Option<f32>,
    pub reading_status: ProsaReadingStatus,
}

/// `PATCH` leaves out what it does not change, so every field is optional and
/// an absent one must not reach the wire as `null`.
#[skip_serializing_none]
#[derive(Serialize, Debug)]
pub(super) struct ProsaStatePatch<'a> {
    pub(super) location: Option<&'a str>,
    pub(super) statistics: ProsaStatisticsPatch,
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
pub(super) struct ProsaStatisticsPatch {
    pub(super) rating: Option<f32>,
    pub(super) reading_status: Option<ProsaReadingStatus>,
}
