use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Default, Eq)]
pub struct Locations {
    #[serde(rename = "Location")]
    pub locations: Vec<Location>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Location {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "LocationAreaCode")]
    pub location_area_code: Option<LocationAreaCode>,
    #[serde(rename = "LocationCountry")]
    pub location_country: Option<LocationCountry>,
    #[serde(rename = "LocationPart")]
    pub location_parts: Option<Vec<LocationPart>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct LocationAreaCode {
    #[serde(rename = "AreaCodeID")]
    pub id: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct LocationCountry {
    #[serde(rename = "CountryID")]
    pub id: i32,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct LocationPart {
    #[serde(rename = "LocPartTypeID")]
    pub id: i32,
    #[serde(rename = "LocationPartValue")]
    pub values: Vec<LocationPartValue>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct LocationPartValue {
    #[serde(rename = "Primary")]
    pub primary: bool,
    #[serde(rename = "Value")]
    pub value: String,
}
