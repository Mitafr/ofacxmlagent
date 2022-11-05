
/////////////////////////////
// REFERENTIELS
/////////////////////////////

Table ref_feature {
  id int [pk, increment]
  value text [not null]
}

Table ref_reference {
  id int [pk, increment]
  value text [not null]
}

Table ref_document {
  id int [pk, increment]
  value text [not null]
}

Table ref_country {
  id int [pk, increment]
  value text [not null]
}

Table ref_type {
  id int [pk, increment]
  value text [not null]
  program text
  type_fmm tinytext
}

Table ddc_pgm {
  id int [pk, increment]
  program text [not null]
  sanctioned boolean [not null]
}


/////////////////////////////
// FIN REFERENTIELS
/////////////////////////////

Table sdn {
  fixed_ref integer [unique, not null]
  record_id int [pk, increment]
  identity int [unique, not null]
  
  
  partysubtypeid int [not null]
  sdn_type text  [not null, note: '''
if PARTYSUBTYPEID = 1 then "Vessel",
if PARTYSUBTYPEID = 2 then "Aircraft",
if PARTYSUBTYPEID = 3 then "Entity",
if PARTYSUBTYPEID = 4 then "Individual"
  ''']
  gender text [note: "feature 224"]
  
  
  // citizen_id int
  // citizen text
  title text [note: "feature 26"]
  additional_sanctions_information int [note: "feature 125"]
  secondary_sanctions_risks int [note: "feature 504"]
  organization_established_date date [note: "feature 646"]
  organization_type int [note: "feature 647"]
  
  locode text [note: "feature 264"]
  micex_code text [note: "feature 304"]
  duns_number int [note: "feature 364"]
  registration_country int [note: "feature 404"]
  prohibited_transactions int [note: "feature 626"]
  
  
  vessel_call_sign text [note: "feature 1"]
  vessel_type int  [note: "feature 2"]
  vessel_flag text  [note: "feature 3"]
  vessel_owner text [note: "feature 4"]
  vessel_tonnage int  [note: "feature 5"]
  vessel_gross_registered_tonnage int [note: "feature 6"]
  
  other_vessel_type int [note: "feature 526"]
  other_vessel_call_sign text [note: "feature 425"]
  
  cmic_effective_date date [note: "feature 867"]
  cmic_sales_date date [note: "feature 868"]
  cmic_listing_date date [note: "feature 869"]
  
  ifca_determination int [note: "feature 104"]
  
  // Adresses crypto
  dca_bch text [note: '''feature 726 concat with a "/"''']
  dca_bsv text [note: '''feature 706 concat with a "/"''']
  dca_btg text [note: '''feature 688 concat with a "/"''']
  dca_dash text [note: '''feature 687 concat with a "/"''']
  dca_etc text [note: '''feature 689 concat with a "/"''']
  dca_eth text [note: '''feature 345 concat with a "/"''']
  dca_ltc text [note: '''feature 566 concat with a "/"''']
  dca_usdt text [note: '''feature 887 concat with a "/"''']
  dca_xbt text [note: '''feature 344 concat with a "/"''']
  dca_xmr text [note: '''feature 444 concat with a "/"''']
  dca_xrp text [note: '''feature 907 concat with a "/"''']
  dca_xvh text [note: '''feature 746 concat with a "/"''']
  dca_zec text [note: '''feature 686 concat with a "/"''']
  // fin Adresses crypto
  
  // Sanction
  sanction_date date [not null]
  sanction_status text [not null]
  // Fin Sanction
  
  // Aircraft
  construction_number text
  manufacturer_serial_number text
  manufacture_date date
  transpondeur_code text
  previous_tail_number text
  tail_number text
  model text
  // Fin Aircraft
  
  peesa_information int [note: "feature 827"]
  
  comment text
  topmaj tinytext [not null]
  updated_by text [note: "Possible values : [Batch, User]"]
  last_update date
  indexes {
    (fixed_ref, identity)
  }
}

Ref: sdn.additional_sanctions_information > ref_reference.id
Ref: sdn.secondary_sanctions_risks > ref_reference.id  
Ref: sdn.prohibited_transactions > ref_reference.id 
Ref: sdn.vessel_type > ref_reference.id 
Ref: sdn.peesa_information > ref_reference.id 
Ref: sdn.other_vessel_type > ref_reference.id 
Ref: sdn.registration_country > ref_country.id 
Ref: sdn.ifca_determination > ref_reference.id 
Ref: sdn.organization_type > ref_reference.id
Ref: sdn.partysubtypeid > ref_type.id


// Operator d'un aircraft
Table aircraft_operator {
  id int [pk]
  operator text [not null]
  topmaj tinytext [not null]
}

Table aircraft_operator_sdn {
  aircraft_operator_id int [ref: > aircraft_operator.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (aircraft_operator_id, sdn_id) [pk]
  }
}

// Bateau d'un SDN
Table former_vessel_flag {
  id int [pk, increment]
  value text
  topmaj tinytext [not null]
}

Table former_vessel_flag_sdn {
  former_vessel_flag_id int [ref: > former_vessel_flag.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (former_vessel_flag_id, sdn_id) [pk]
  }
}

Table program {
  id int [pk]
  program text
  topmaj tinytext [not null]
}

Table sdn_program {
  program_id int [ref: > program.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (program_id, sdn_id) [pk]
  }
}

// Ref: sanction.id > sdn.fixed_ref

// Adresse d'un SDN
Table address {
  id int [pk]
  address text
  city text
  country int
  postal_code text
  region text
  state text
  is_primary bool [not null]
  topmaj tinytext [not null]
}

Table address_sdn {
  address_id int [ref: > address.id]
  identity_id int [ref: > sdn.identity]
  indexes {
    (address_id, identity_id) [pk]
  }
}

// Code ISIN d'un SDN
Table isin {
  id int [pk, increment]
  isin text
  topmaj tinytext [not null]
}

Table isin_sdn {
  isin_id int [ref: > isin.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (isin_id, sdn_id) [pk]
  }
}

// Issuer name d'un SDN
Table issuer_name {
  id int [pk]
  issuer_name text
  topmaj tinytext [not null]
}

Table issuer_name_sdn {
  issuer_name_id int [ref: > issuer_name.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (issuer_name_id, sdn_id) [pk]
  }
}

// Executive Order 13662 Directive Determination d'un SDN
Table eo13662dd {
  id int [pk]
  reference_id int
  topmaj tinytext [not null]
}

Ref: eo13662dd.reference_id > ref_reference.id

Table eo13662dd_sdn {
  eo13662dd_id int [ref: > eo13662dd.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (eo13662dd_id, sdn_id) [pk]
  }
}

// Executive Order 13846 information d'un SDN
Table eo13846inf {
  id int [pk]
  reference_id int
  topmaj tinytext [not null]
}

Ref: eo13846inf.reference_id > ref_reference.id

Table eo13846inf_sdn {
  eo13846inf_id int [ref: > eo13846inf.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (eo13846inf_id, sdn_id) [pk]
  }
}

// Executive Order 14024 Directive Information d'un SDN
Table eo14024dd {
  id int [pk]
  reference_id int
  topmaj tinytext [not null]
}

Ref: eo14024dd.reference_id > ref_reference.id

Table eo14024dd_sdn {
  eo14024dd_id int [ref: > eo14024dd.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (eo14024dd_id, sdn_id) [pk]
  }
}

// Nationality Registration d'un SDN
Table nationality_registration {
  id int [pk]
  location text
  topmaj tinytext [not null]
}

Table nationality_registration_sdn {
  nationality_registration_id int [ref: > nationality_registration.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (nationality_registration_id, sdn_id) [pk]
  }
}

// Citizen d'un SDN
Table citizen {
  id int [pk, increment]
  location text
  topmaj tinytext [not null]
}

Table citizen_sdn {
  citizen_id int [ref: > citizen.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (citizen_id, sdn_id) [pk]
  }
}

// Executive Order 14024 Directive Information d'un SDN
Table caatsa235 {
  id int [pk, increment]
  reference_id int
  topmaj tinytext [not null]
}

Ref: caatsa235.reference_id > ref_reference.id

Table caatsa235_sdn {
  caatsa235_id int [ref: > caatsa235.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (caatsa235_id, sdn_id) [pk]
  }
}

Table equity_ticker {
  id int [pk, increment]
  equity_ticker text [not null]
  topmaj tinytext [not null]
}

Table equity_ticker_sdn {
  equity_ticker_id int [ref: > equity_ticker.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (equity_ticker_id, sdn_id) [pk]
  }
}

// Email d'un SDN
Table email {
  id int [pk, increment]
  email text [not null]
  topmaj tinytext [not null]
}

Table email_sdn {
  email_id int [ref: > email.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (email_id, sdn_id) [pk]
  }
}

Table target {
  id int [pk, increment]
  target int [not null]
  topmaj tinytext [not null]
}

Ref: target.target > ref_reference.id 

Table target_sdn {
  target_id int [ref: > target.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (target_id, sdn_id) [pk]
  }
}

Ref: address.country > ref_country.id

// 

Table name {
  id int [pk, increment]
  type text [not null]
  script int [not null]
  last_name text
  first_name text
  middle_name text
  maiden_name text
  aircraft_name text
  entity_name text
  vessel_name text
  nickname text
  patronymic text
  matronymic text
  quality text
  topmaj tinytext [not null]
}

Table name_sdn {
  name_id int [ref: > name.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (name_id, sdn_id) [pk]
  }
}

// Année de naissance d'un SDN
Table dob {
  id int [pk, increment]
  dob text [not null]
  topmaj tinytext [not null]
}

Table dob_identity {
  dob_id int [ref: > dob.id]
  identity_id int [ref: > sdn.identity]
  indexes {
    (dob_id, identity_id) [pk]
  }
}

// Lieu de naissance d'un SDN
Table pob {
  id int [pk, increment]
  pob text [not null]
  topmaj tinytext [not null]
}

Table pob_identity {
  pob_id int [ref: > pob.id]
  identity_id int [ref: > sdn.identity]
  indexes {
    (pob_id, identity_id) [pk]
  }
}

// Site internet d'un SDN
Table website {
  id int [pk, increment]
  website text [not null]
  topmaj tinytext [not null]
}

Table website_identity {
  website_id int [ref: > website.id]
  identity_id int [ref: > sdn.identity]
  indexes {
    (website_id, identity_id) [pk]
  }
}

// Bic d'un SDN
Table bic {
  id int [pk, increment]
  bic text [not null]
  topmaj tinytext [not null]
}

Table bic_sdn {
  bic_id int [ref: > bic.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (bic_id, sdn_id) [pk]
  }
}

Table dateofissue {
  id int [pk, default: 0]
  last_document date [not null]
}

// Nationalité d'un SDN
Table nationality {
  id int [pk]
  nationality int
  topmaj tinytext [not null]
}

Table nationality_identity {
  nationality_id int [ref: > nationality.id]
  identity_id int [ref: > sdn.identity]
  indexes {
    (nationality_id, identity_id) [pk]
  }
}

Ref: nationality.nationality > ref_country.id

// Document relatif à un SDN
Table document {
  id int [pk]
  doctype int
  registration_number text
  issued_by int
  issued_date date
  expiration_date date
  topmaj tinytext [not null]
}


Table document_identity {
  document_id int [ref: > document.id]
  identity_id int [ref: > sdn.identity]
  indexes {
    (document_id, identity_id) [pk]
  }
}

Ref: document.issued_by > ref_country.id
Ref: document.doctype > ref_document.id


// Relation d'un SDN
Table relation {
  id int [pk]
  linked_to int [not null] // FixedRef SDN
  relation_type_id int [not null]
  topmaj tinytext [not null]
}

Ref: relation.linked_to > sdn.fixed_ref

Table relation_sdn {
  relation_id int [ref: > relation.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (relation_id, sdn_id) [pk]
  }
}

// Bateau d'un SDN
Table other_vessel_flag {
  id int [pk, increment]
  value text
  topmaj tinytext [not null]
}

Table other_vessel_flag_sdn {
  other_vessel_flag_id int [ref: > other_vessel_flag.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (other_vessel_flag_id, sdn_id) [pk]
  }
}

Table bik {
  id int [pk]
  bik text
  topmaj tinytext [not null]
}

Table bik_sdn {
  bik_id int [ref: > bik.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (bik_id, sdn_id) [pk]
  }
}

Table phone_number {
  id int [pk]
  phone_number text [not null]
  topmaj tinytext [not null]
}

Table phone_number_sdn {
  phone_number_id int [ref: > phone_number.id]
  sdn_id int [ref: > sdn.fixed_ref]
  indexes {
    (phone_number_id, sdn_id) [pk]
  }
}

// Tables DDC utilisée pour enrichir les SDNs
Table ddc_alias {
  id int [pk, increment]
  name text [not null]
  quality text
}

Table ddc_alias_sdn {
  ddc_alias_id int [ref: > ddc_alias.id]
  sdn_id int [ref: > sdn.record_id]
  indexes {
    (ddc_alias_id, sdn_id) [pk]
  }
}
Table ddc_bic {
  id int [pk, increment]
  bic text [not null]
}

Table ddc_bic_sdn {
  ddc_bic_id int [ref: > ddc_bic.id]
  sdn_id int [ref: > sdn.record_id]
  indexes {
    (ddc_bic_id, sdn_id) [pk]
  }
}

// Tables DDC utilisée pour ajouter des noms sous sanction
Table ddc_name {
  id int [pk, increment]
  name text [not null]
}
