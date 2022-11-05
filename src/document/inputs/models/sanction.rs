use serde::Deserialize;

use super::feature::Date;

#[derive(Debug, Deserialize, PartialEq, Eq, Default)]
pub struct SanctionsEntries {
    #[serde(rename = "SanctionsEntry")]
    pub entries: Vec<SanctionsEntry>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct SanctionsEntry {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "ProfileID")]
    pub profile_id: i32,
    #[serde(rename = "EntryEvent")]
    pub events: Vec<EntryEvent>,
    #[serde(rename = "SanctionsMeasure")]
    pub measures: Vec<SanctionsMeasure>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct EntryEvent {
    #[serde(rename = "Date")]
    pub date: Date,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct SanctionsMeasure {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "Comment")]
    pub program: Option<String>,
}
