use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Alias {
    #[serde(rename = "AliasTypeID")]
    pub alias_type: i32,
    #[serde(rename = "Primary")]
    pub primary: bool,
    #[serde(rename = "DocumentedName")]
    pub documented_name: Vec<DocumentedName>,
    #[serde(rename = "LowQuality")]
    pub quality: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DocumentedName {
    #[serde(rename = "DocumentedNamePart")]
    pub parts: Vec<DocumentedNamePart>,
    #[serde(rename = "DocNameStatusID")]
    pub doc_name_status: i32,
    #[serde(rename = "ID")]
    pub id: i32,
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DocumentedNamePart {
    #[serde(rename = "NamePartValue")]
    pub name_part: NamePartValue,
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct NamePartValue {
    #[serde(rename = "$value")]
    pub name: Option<String>,
    #[serde(rename = "ScriptID")]
    pub script_id: i32,
    #[serde(rename = "NamePartGroupID")]
    pub name_part_group_id: i32,
}
