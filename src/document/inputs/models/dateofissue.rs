use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Default, Clone, Eq)]
pub struct DateOfIssue {
    #[serde(rename = "Year")]
    pub year: String,
    #[serde(rename = "Month")]
    pub month: String,
    #[serde(rename = "Day")]
    pub day: String,
}

impl DateOfIssue {
    pub fn to_sql_date(&self) -> NaiveDate {
        NaiveDate::from_ymd(self.year.parse::<i32>().unwrap(), self.month.parse::<u32>().unwrap(), self.day.parse::<u32>().unwrap())
    }
}
