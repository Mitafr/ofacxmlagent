use serde::Deserialize;

use super::feature::DatePeriod;

#[derive(Debug, Deserialize, PartialEq, Default, Eq)]
pub struct IDRegDocuments {
    #[serde(rename = "IDRegDocument")]
    pub documents: Vec<Document>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Document {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "IDRegDocTypeID")]
    pub type_id: i32,
    #[serde(rename = "IdentityID")]
    pub identity_id: i32,
    #[serde(rename = "IssuedBy-CountryID")]
    pub issued_by: Option<i32>,
    #[serde(rename = "Comment")]
    pub comment: Option<String>,
    #[serde(rename = "IDRegistrationNo")]
    pub registration_number: Option<String>,
    #[serde(rename = "DocumentedNameReference")]
    pub reference: Option<DocumentedNameReference>,
    #[serde(rename = "DocumentDate")]
    pub dates: Option<Vec<DocumentDate>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DocumentDate {
    #[serde(rename = "IDRegDocDateTypeID")]
    pub type_id: i32,
    #[serde(rename = "DatePeriod")]
    pub period: DatePeriod,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DocumentedNameReference {
    #[serde(rename = "DocumentedNameID")]
    pub id: i32,
}
