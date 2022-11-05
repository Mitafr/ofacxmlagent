use super::{alias::Alias, feature::Feature};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Default, Eq)]
pub struct DistinctParties {
    #[serde(rename = "DistinctParty")]
    pub parties: Vec<DistinctParty>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DistinctParty {
    #[serde(rename = "FixedRef")]
    pub fixed_ref: i32,
    #[serde(rename = "Comment")]
    pub comment: Option<String>,
    #[serde(rename = "Profile")]
    pub profile: Profile,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Profile {
    #[serde(rename = "PartySubTypeID")]
    pub party_sub_id: i32,
    #[serde(rename = "Identity")]
    pub identity: Identity,
    #[serde(rename = "Feature")]
    pub feature: Option<Vec<Feature>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Identity {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "Alias")]
    pub alias: Vec<Alias>,
    #[serde(rename = "NamePartGroups")]
    pub name_part_groups: NamePartGroups,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct NamePartGroups {
    #[serde(rename = "MasterNamePartGroup")]
    pub master_name_part_group: Vec<MasterNamePartGroup>,
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct MasterNamePartGroup {
    #[serde(rename = "NamePartGroup")]
    pub name_part_group: NamePartGroup,
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct NamePartGroup {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "NamePartTypeID")]
    pub name_part_type_id: i32,
}
