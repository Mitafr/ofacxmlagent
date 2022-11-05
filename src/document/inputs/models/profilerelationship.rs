use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Clone, Default, Eq)]
pub struct ProfileRelationships {
    #[serde(rename = "ProfileRelationship")]
    pub profile_relationships: Vec<ProfileRelationship>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ProfileRelationship {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "From-ProfileID")]
    pub from_profile_id: i32,
    #[serde(rename = "SanctionsEntryID")]
    pub sanction_entry_id: i32,
    #[serde(rename = "RelationTypeID")]
    pub relation_type_id: i32,
    #[serde(rename = "To-ProfileID")]
    pub to_profile_id: i32,
}
