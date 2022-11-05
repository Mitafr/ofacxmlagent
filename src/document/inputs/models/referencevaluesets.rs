use serde::Deserialize;

use super::areacode::AreaCodeValues;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ReferenceValueSets {
    #[serde(rename = "AreaCodeValues")]
    pub area_code_values: AreaCodeValues,
    #[serde(rename = "DetailReferenceValues")]
    pub detail_reference_values: DetailReferenceValues,
    #[serde(rename = "FeatureTypeValues")]
    pub feature_types: FeatureTypeValues,
    #[serde(rename = "PartySubTypeValues")]
    pub party_sub_type_values: PartySubTypeValues,
    #[serde(rename = "IDRegDocTypeValues")]
    pub reg_doc_types_values: IDRegDocTypeValues,
    #[serde(rename = "ScriptValues")]
    pub script_values: ScriptValues,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default, Eq)]
pub struct FeatureTypeValues {
    #[serde(rename = "FeatureType")]
    pub types: Vec<FeatureType>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Eq)]
pub struct FeatureType {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default, Eq)]
pub struct IDRegDocTypeValues {
    #[serde(rename = "IDRegDocType")]
    pub reg_doc_types: Vec<IDRegDocType>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Eq)]
pub struct IDRegDocType {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default, Eq)]
pub struct DetailReferenceValues {
    #[serde(rename = "DetailReference")]
    pub detail_references: Vec<DetailReference>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Eq)]
pub struct DetailReference {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Eq, Default)]
pub struct PartySubTypeValues {
    #[serde(rename = "PartySubType")]
    pub values: Vec<PartySubType>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
pub struct PartySubType {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default, Eq)]
pub struct ScriptValues {
    #[serde(rename = "Script")]
    pub scripts: Vec<Script>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Eq)]
pub struct Script {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "ScriptCode")]
    pub code: String,
    #[serde(rename = "$value")]
    pub value: String,
}
