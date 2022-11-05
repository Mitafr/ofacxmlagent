use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Clone, Eq)]
pub struct AreaCodeValues {
    #[serde(rename = "AreaCode")]
    pub area_codes: Vec<AreaCode>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Eq)]
pub struct AreaCode {
    #[serde(rename = "CountryID")]
    pub id: i32,
    #[serde(rename = "Description")]
    pub name: String,
}
