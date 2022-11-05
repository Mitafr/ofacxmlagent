use sea_orm::{entity::prelude::*, EntityTrait, RelationTrait, Set};

use crate::document::models::feature::FeatureVersion;

use crate::db::{impl_topmaj, OfacEntity};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "dob")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub dob: String,
    #[sea_orm(column_type = "Custom(\"TINYTEXT\".to_owned())")]
    pub topmaj: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::dob_identity::Entity")]
    DobIdentity,
}

impl Related<super::dob_identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DobIdentity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// TODO : simplified conditions here
    pub fn from_ofac_document(entity: &FeatureVersion) -> Model {
        let mut dob = Model {
            id: entity.id,
            topmaj: "N".to_owned(),
            ..Default::default()
        };
        let date_of_period = entity.date_period.as_ref().unwrap();
        let start = date_of_period.start.as_ref().unwrap();
        let end = date_of_period.end.as_ref().unwrap();
        // Unique dmy date
        if start.from == start.to && end.from == end.to && start.from == end.from {
            if !start.approximate && !end.approximate {
                dob.dob = start.from.to_ofac_string_dmy();
            } else {
                dob.dob = format!("CIRCA {}", start.from.to_ofac_string_dmy());
            }
        }
        // Only year date
        else if start.from.is_first_day_of_month()
            && start.to.is_first_day_of_month()
            && end.from.is_last_day_of_month()
            && end.to.is_last_day_of_month()
            && start.from.is_first_month_of_year()
            && start.to.is_first_month_of_year()
            && end.from.is_last_month_of_year()
            && end.to.is_last_month_of_year()
            && start.from.year == start.to.year
            && end.from.year == end.to.year
            && start.from.year == end.from.year
        {
            if !start.approximate && !end.approximate {
                dob.dob = start.from.year.to_uppercase();
            } else {
                dob.dob = format!("CIRCA {}", start.from.year);
            }
        }
        // Only month date
        else if start.from.is_first_day_of_month()
            && start.to.is_first_day_of_month()
            && end.from.is_last_day_of_month()
            && end.to.is_last_day_of_month()
            && start.from.year == start.to.year
            && end.from.year == end.to.year
            && start.from.year == end.from.year
            && start.from.month == start.to.month
            && end.from.month == end.to.month
            && start.from.month == end.from.month
        {
            if !start.approximate && !end.approximate {
                dob.dob = start.from.to_ofac_string_my();
            } else {
                dob.dob = format!("CIRCA {}", start.from.to_ofac_string_my());
            }
        }
        // Range of years
        else if start.from.is_first_day_of_month()
            && start.to.is_last_day_of_month()
            && end.from.is_first_day_of_month()
            && end.to.is_last_day_of_month()
            && start.from.is_first_month_of_year()
            && start.to.is_last_month_of_year()
            && end.from.is_first_month_of_year()
            && end.to.is_last_month_of_year()
            && start.from.year == start.to.year
            && end.from.year == end.to.year
            && start.from.year != end.from.year
        {
            dob.dob = format!("{} TO {}", start.from.year, end.from.year);
        }
        // Range of months
        else if start.from.is_first_day_of_month()
            && start.to.is_first_day_of_month()
            && end.from.is_last_day_of_month()
            && end.to.is_last_day_of_month()
            && start.from.is_first_month_of_year()
            && start.to.is_first_month_of_year()
            && end.from.is_first_month_of_year()
            && end.to.is_first_month_of_year()
            && start.from.year == start.to.year
            && end.from.year == end.to.year
            && start.from.year != end.from.year
        {
            dob.dob = format!("{} TO {}", start.from.to_ofac_string_my(), start.to.to_ofac_string_my());
        }
        // Range same year only
        else if start.from.year == start.to.year && end.from.year == end.to.year {
            dob.dob = format!("{} TO {}", start.from.to_ofac_string_dmy(), end.from.to_ofac_string_dmy());
        }
        dob
    }
}

impl_topmaj! {
    Entity, Model, super::dob_identity::ActiveModel, ActiveModel
}

#[cfg(test)]
mod dob {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn unique_y_date() {
        let feature: FeatureVersion = from_str(
            r#"
		<FeatureVersion ID="41040" ReliabilityID="1">
				<Comment />
				<DatePeriod CalendarTypeID="1" YearFixed="false" MonthFixed="false" DayFixed="false">
				<Start Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
					<From>
					<Year>1961</Year>
					<Month>1</Month>
					<Day>1</Day>
					</From>
					<To>
					<Year>1961</Year>
					<Month>1</Month>
					<Day>1</Day>
					</To>
				</Start>
				<End Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
					<From>
					<Year>1961</Year>
					<Month>12</Month>
					<Day>31</Day>
					</From>
					<To>
					<Year>1961</Year>
					<Month>12</Month>
					<Day>31</Day>
					</To>
				</End>
				</DatePeriod>
				<VersionDetail DetailTypeID="1430" />
			</FeatureVersion>"#,
        )
        .unwrap();
        let dob = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 41040,
            dob: "1961".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, dob);
    }

    #[test]
    fn unique_approximate_y_date() {
        let feature: FeatureVersion = from_str(
            r#"
			<FeatureVersion ID="6684" ReliabilityID="1">
			  <Comment />
			  <DatePeriod CalendarTypeID="1" YearFixed="false" MonthFixed="false" DayFixed="false">
				<Start Approximate="true" YearFixed="false" MonthFixed="false" DayFixed="false">
				  <From>
					<Year>1948</Year>
					<Month>1</Month>
					<Day>1</Day>
				  </From>
				  <To>
					<Year>1948</Year>
					<Month>1</Month>
					<Day>1</Day>
				  </To>
				</Start>
				<End Approximate="true" YearFixed="false" MonthFixed="false" DayFixed="false">
				  <From>
					<Year>1948</Year>
					<Month>12</Month>
					<Day>31</Day>
				  </From>
				  <To>
					<Year>1948</Year>
					<Month>12</Month>
					<Day>31</Day>
				  </To>
				</End>
			  </DatePeriod>
			  <VersionDetail DetailTypeID="1430" />
			</FeatureVersion>"#,
        )
        .unwrap();
        let dob = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 6684,
            dob: "CIRCA 1948".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, dob);
    }

    #[test]
    fn unique_my_date() {
        let feature: FeatureVersion = from_str(
            r#"
		<FeatureVersion ID="37877" ReliabilityID="1">
            <Comment />
            <DatePeriod CalendarTypeID="1" YearFixed="false" MonthFixed="false" DayFixed="false">
              <Start Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
                <From>
                  <Year>1968</Year>
                  <Month>2</Month>
                  <Day>1</Day>
                </From>
                <To>
                  <Year>1968</Year>
                  <Month>2</Month>
                  <Day>1</Day>
                </To>
              </Start>
              <End Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
                <From>
                  <Year>1968</Year>
                  <Month>2</Month>
                  <Day>29</Day>
                </From>
                <To>
                  <Year>1968</Year>
                  <Month>2</Month>
                  <Day>29</Day>
                </To>
              </End>
            </DatePeriod>
            <VersionDetail DetailTypeID="1430" />
          </FeatureVersion>"#,
        )
        .unwrap();
        let dob = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 37877,
            dob: "FEB 1968".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, dob);
    }

    #[test]
    fn unique_my_feb_date() {
        let feature: FeatureVersion = from_str(
            r#"
		<FeatureVersion ID="37877" ReliabilityID="1">
            <Comment />
            <DatePeriod CalendarTypeID="1" YearFixed="false" MonthFixed="false" DayFixed="false">
              <Start Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
                <From>
                  <Year>1967</Year>
                  <Month>2</Month>
                  <Day>1</Day>
                </From>
                <To>
                  <Year>1967</Year>
                  <Month>2</Month>
                  <Day>1</Day>
                </To>
              </Start>
              <End Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
                <From>
                  <Year>1967</Year>
                  <Month>2</Month>
                  <Day>28</Day>
                </From>
                <To>
                  <Year>1967</Year>
                  <Month>2</Month>
                  <Day>28</Day>
                </To>
              </End>
            </DatePeriod>
            <VersionDetail DetailTypeID="1430" />
          </FeatureVersion>"#,
        )
        .unwrap();
        let dob = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 37877,
            dob: "FEB 1967".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, dob);
    }
    #[test]
    fn unique_dmy_date() {
        let feature: FeatureVersion = from_str(
            r#"
		<FeatureVersion ID="46531" ReliabilityID="1">
            <Comment />
            <DatePeriod CalendarTypeID="1" YearFixed="false" MonthFixed="false" DayFixed="false">
              <Start Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
                <From>
                  <Year>1975</Year>
                  <Month>4</Month>
                  <Day>13</Day>
                </From>
                <To>
                  <Year>1975</Year>
                  <Month>4</Month>
                  <Day>13</Day>
                </To>
              </Start>
              <End Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
                <From>
                  <Year>1975</Year>
                  <Month>4</Month>
                  <Day>13</Day>
                </From>
                <To>
                  <Year>1975</Year>
                  <Month>4</Month>
                  <Day>13</Day>
                </To>
              </End>
            </DatePeriod>
            <VersionDetail DetailTypeID="1430" />
          </FeatureVersion>"#,
        )
        .unwrap();
        let dob = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 46531,
            dob: "13 APR 1975".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, dob);
    }

    #[test]
    fn unique_approximate_dmy_date() {
        let feature: FeatureVersion = from_str(
            r#"
		<FeatureVersion ID="6516" ReliabilityID="1">
			<Comment />
			<DatePeriod CalendarTypeID="1" YearFixed="false" MonthFixed="false" DayFixed="false">
			  <Start Approximate="true" YearFixed="false" MonthFixed="false" DayFixed="false">
				<From>
				  <Year>1966</Year>
				  <Month>7</Month>
				  <Day>7</Day>
				</From>
				<To>
				  <Year>1966</Year>
				  <Month>7</Month>
				  <Day>7</Day>
				</To>
			  </Start>
			  <End Approximate="true" YearFixed="false" MonthFixed="false" DayFixed="false">
				<From>
				  <Year>1966</Year>
				  <Month>7</Month>
				  <Day>7</Day>
				</From>
				<To>
				  <Year>1966</Year>
				  <Month>7</Month>
				  <Day>7</Day>
				</To>
			  </End>
			</DatePeriod>
			<VersionDetail DetailTypeID="1430" />
		</FeatureVersion>"#,
        )
        .unwrap();
        let dob = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 6516,
            dob: "CIRCA 07 JUL 1966".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, dob);
    }

    #[test]
    fn range_dmy_date() {
        let feature: FeatureVersion = from_str(
            r#"
		<FeatureVersion ID="1" ReliabilityID="1">
			<Comment />
			<DatePeriod CalendarTypeID="1" YearFixed="false" MonthFixed="false" DayFixed="false">
			<Start Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
				<From>
				<Year>1946</Year>
				<Month>9</Month>
				<Day>26</Day>
				</From>
				<To>
				<Year>1946</Year>
				<Month>9</Month>
				<Day>26</Day>
				</To>
			</Start>
			<End Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
				<From>
				<Year>1946</Year>
				<Month>12</Month>
				<Day>7</Day>
				</From>
				<To>
				<Year>1946</Year>
				<Month>12</Month>
				<Day>7</Day>
				</To>
			</End>
			</DatePeriod>
			<VersionDetail DetailTypeID="1430" />
		</FeatureVersion>"#,
        )
        .unwrap();
        let dob = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 1,
            dob: "26 SEP 1946 TO 07 DEC 1946".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, dob);
    }

    #[test]
    fn range_y_date() {
        let feature: FeatureVersion = from_str(
            r#"
			<FeatureVersion ID="17587" ReliabilityID="1">
			  <Comment />
			  <DatePeriod CalendarTypeID="1" YearFixed="false" MonthFixed="false" DayFixed="false">
				<Start Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
				  <From>
					<Year>1984</Year>
					<Month>1</Month>
					<Day>1</Day>
				  </From>
				  <To>
					<Year>1984</Year>
					<Month>12</Month>
					<Day>31</Day>
				  </To>
				</Start>
				<End Approximate="false" YearFixed="false" MonthFixed="false" DayFixed="false">
				  <From>
					<Year>1986</Year>
					<Month>1</Month>
					<Day>1</Day>
				  </From>
				  <To>
					<Year>1986</Year>
					<Month>12</Month>
					<Day>31</Day>
				  </To>
				</End>
			  </DatePeriod>
			  <VersionDetail DetailTypeID="1430" />
			</FeatureVersion>"#,
        )
        .unwrap();
        let dob = Model::from_ofac_document(&feature);
        let excepted = Model {
            id: 17587,
            dob: "1984 TO 1986".to_owned(),
            topmaj: "N".to_owned(),
        };
        assert_eq!(excepted, dob);
    }
}
