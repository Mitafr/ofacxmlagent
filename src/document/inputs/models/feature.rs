use std::fmt::Write;

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Feature {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "FeatureTypeID")]
    pub feature_type: i32,
    #[serde(rename = "FeatureVersion")]
    pub version: FeatureVersion,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct FeatureVersion {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "ReliabilityID")]
    pub reliability_id: i32,
    #[serde(rename = "VersionDetail", skip_serializing_if = "Option::is_none")]
    pub detail: Option<VersionDetail>,
    #[serde(rename = "VersionLocation", skip_serializing_if = "Option::is_none")]
    pub location: Option<VersionLocation>,
    #[serde(rename = "DatePeriod", skip_serializing_if = "Option::is_none")]
    pub date_period: Option<DatePeriod>,
}

#[derive(Debug, Deserialize, PartialEq, Copy, Clone, Eq)]
pub struct VersionLocation {
    #[serde(rename = "LocationID")]
    pub id: i32,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Eq)]
pub struct DatePeriod {
    #[serde(rename = "Start")]
    pub start: Option<DatePeriodRange>,
    #[serde(rename = "End")]
    pub end: Option<DatePeriodRange>,
}

impl DatePeriod {
    pub fn parse_from_to(&self) -> Option<NaiveDate> {
        let start = self.start.as_ref().unwrap();
        let end = self.end.as_ref().unwrap();
        if start.from == start.to && end.from == end.to {
            return Some(start.from.to_sql_date());
        }
        None
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone, Eq)]
pub struct DatePeriodRange {
    #[serde(rename = "From")]
    pub from: Date,
    #[serde(rename = "To")]
    pub to: Date,
    #[serde(rename = "Approximate")]
    pub approximate: bool,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Eq)]
pub struct Date {
    #[serde(rename = "Year")]
    pub year: String,
    #[serde(rename = "Month")]
    pub month: String,
    #[serde(rename = "Day")]
    pub day: String,
}

impl Date {
    pub fn to_sql_date(&self) -> NaiveDate {
        NaiveDate::from_ymd(self.year.parse::<i32>().unwrap(), self.month.parse::<u32>().unwrap(), self.day.parse::<u32>().unwrap())
    }

    pub fn to_string_date(&self) -> String {
        let mut sql_date = String::new();
        sql_date.push_str(&self.year);
        sql_date.push('-');
        write!(sql_date, "{:0>2}", &self.month).unwrap();
        sql_date.push('-');
        write!(sql_date, "{:0>2}", &self.day).unwrap();
        sql_date
    }

    pub fn to_ofac_string_dmy(&self) -> String {
        let mut ofac_date = String::new();
        write!(ofac_date, "{:0>2}", &self.day).unwrap();
        ofac_date.push(' ');
        ofac_date.push_str(&self.format_month());
        ofac_date.push(' ');
        ofac_date.push_str(&self.year);
        ofac_date
    }

    pub fn to_ofac_string_my(&self) -> String {
        let mut ofac_date = String::new();
        ofac_date.push_str(&self.format_month());
        ofac_date.push(' ');
        ofac_date.push_str(&self.year);
        ofac_date
    }

    fn format_month(&self) -> String {
        match &format!("{:0>2}", &self.month)[..] {
            "01" => "JAN".to_owned(),
            "02" => "FEB".to_owned(),
            "03" => "MAR".to_owned(),
            "04" => "APR".to_owned(),
            "05" => "MAY".to_owned(),
            "06" => "JUN".to_owned(),
            "07" => "JUL".to_owned(),
            "08" => "AUG".to_owned(),
            "09" => "SEP".to_owned(),
            "10" => "OCT".to_owned(),
            "11" => "NOV".to_owned(),
            "12" => "DEC".to_owned(),
            _ => panic!("Month format not recognized {:?} {}", self, self.month),
        }
    }

    pub fn is_last_day_of_month(&self) -> bool {
        let year = self.year.parse().unwrap();
        let month = self.month.parse().unwrap();
        let last_day = if month == 12 { NaiveDate::from_ymd(year + 1, 1, 1) } else { NaiveDate::from_ymd(year, month + 1, 1) }
            .signed_duration_since(NaiveDate::from_ymd(year, month, 1))
            .num_days();
        last_day == self.day.parse::<i64>().unwrap()
    }

    pub fn is_first_day_of_month(&self) -> bool {
        self.day == "1"
    }

    pub fn is_first_month_of_year(&self) -> bool {
        self.month == "1"
    }

    pub fn is_last_month_of_year(&self) -> bool {
        self.month == "12"
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone, Eq)]
pub struct VersionDetail {
    #[serde(rename = "DetailTypeID")]
    pub detail_type_id: Option<i32>,
    #[serde(rename = "DetailReferenceID")]
    pub detail_reference_id: Option<i32>,
    #[serde(rename = "$value")]
    pub value: Option<String>,
}
