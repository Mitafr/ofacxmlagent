use super::Exporter;
use async_trait::async_trait;
use std::{
    error::Error,
    fs::OpenOptions,
    io::{Cursor, Write},
    path::Path,
};

use crate::config::Config;
use chrono::{Local, NaiveDate, NaiveDateTime};
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};
use sea_orm::DatabaseConnection;

use crate::{
    db::{entity::ddc_name::Model as DdcName, find_records, init_ddc_db},
    document::{DocumentType, FofdbofRecord},
    processor::entity::sdn::SdnRecord,
};

pub struct FofdbofExporter {
    pub filepath: String,
    records: Vec<FofdbofRecord>,
    template: Vec<u8>,
    doc_type: DocumentType,
    created_at: NaiveDateTime,
    author: String,
    title: String,
    appli: String,
    version: String,
    template_loaded: bool,
}

impl Default for FofdbofExporter {
    fn default() -> Self {
        let app_name = env!("CARGO_PKG_NAME").to_uppercase();
        let path = format!("./output/{}", String::from("FOFDBOF.t"));
        if Path::new(&path).exists() {
            std::fs::remove_file(&path).unwrap();
        }
        Self {
            filepath: path,
            records: Vec::new(),
            template: Vec::new(),
            doc_type: DocumentType::OFAC,
            created_at: NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0),
            author: "CAPS OFAC BATCH".to_owned(),
            title: "OFAC Lists".to_owned(),
            appli: app_name,
            version: env!("CARGO_PKG_VERSION").to_owned(),
            template_loaded: false,
        }
    }
}

#[async_trait]
impl Exporter for FofdbofExporter {
    async fn process(&mut self, db: &DatabaseConnection, doc_type: &DocumentType, config: &Config) -> Result<(), Box<dyn Error>> {
        let ddc_db = init_ddc_db(config).await?;
        self.doc_type = *doc_type;
        self.created_at = Local::now().naive_local();
        self.load_template();
        let records = find_records(db, &ddc_db).await.unwrap();
        self.load_from_db_records(&records.0, &records.1);
        Ok(())
    }

    fn flush(&self) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new().read(true).write(true).append(true).create(true).truncate(false).open(&self.filepath).unwrap();
        self.write_in(&mut file)?;
        Ok(())
    }

    fn write_in<W: Write>(&self, buffer: &mut W) -> Result<(), Box<dyn Error>> {
        let inner_content = &String::from_utf8(self.get_formatted_records()).unwrap()[..];
        let mut reader = Reader::from_str(std::str::from_utf8(&self.template).unwrap());
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 0);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(ref e)) if e.as_ref() == b"INNER_RECORDS" => {
                    writer.write_indent().unwrap();
                    assert!(writer.write_event(Event::Text(BytesText::from_escaped(inner_content))).is_ok());
                }
                Ok(Event::Eof) => break,
                Ok(e) => assert!(writer.write_event(e).is_ok()),
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            }
            buf.clear();
        }
        Ok(buffer.write_all(&writer.into_inner().into_inner())?)
    }
}

impl FofdbofExporter {
    fn load_from_db_records(&mut self, db_records: &[SdnRecord], other_names: &[DdcName]) {
        db_records
            .iter()
            .for_each(|db_record| self.records.append(&mut FofdbofRecord::from_db_record(db_record, &self.doc_type, other_names)));
    }

    fn get_formatted_records(&self) -> Vec<u8> {
        let mut formatted_records: Vec<u8> = Vec::new();
        self.records.iter().for_each(|record| formatted_records.append(&mut record.to_string().as_bytes().to_vec()));
        formatted_records
    }

    fn created_at(&self) -> String {
        self.created_at.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn load_template(&mut self) {
        if self.template_loaded {
            return;
        }
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 1);
        assert!(writer.write_event(Event::Start(BytesStart::new("FIRCO-OFAC-AGENT"))).is_ok());
        for inner_tags in [
            ("CREATED", &self.created_at()),
            ("AUTHOR", &self.author),
            ("TITLE", &self.title),
            ("APPLI", &self.appli),
            ("VERSION", &self.version),
        ] {
            assert!(writer.write_event(Event::Start(BytesStart::new(inner_tags.0))).is_ok());
            assert!(writer.write_event(Event::Text(BytesText::new(inner_tags.1))).is_ok());
            assert!(writer.write_event(Event::End(BytesEnd::new(inner_tags.0))).is_ok());
        }
        assert!(writer.write_event(Event::Start(BytesStart::new("FORMAT"))).is_ok());
        assert!(writer.write_event(Event::Start(BytesStart::new("VARIABLE"))).is_ok());
        for inner_tags in [
            ("EOR", "/010"),
            ("EOC", "/009"),
            ("OID", "1"),
            ("NAM", "2"),
            ("ADD", "3"),
            ("CIT", "4"),
            ("CTR", "5"),
            ("STA", "6"),
            ("TYP", "7"),
            ("BAD", "8"),
            ("SHK", "9"),
            ("SYN", "10"),
            ("SYC", "11"),
            ("SYK", "12"),
            ("SYS", "13"),
            ("ORI", "14"),
            ("DSG", "15"),
            ("US1", "16"),
            ("US2", "17"),
            ("REF", "18"),
            ("BIC", "19"),
            ("PSP", "20"),
            ("NID", "21"),
            ("POB", "22"),
            ("DOB", "23"),
            ("BGH", "24"),
            ("INF", "25"),
            ("ORH", "26"),
            ("TGH", "27"),
            ("IDH", "28"),
            ("UNH", "29"),
            ("SRH", "30"),
            ("PEP", "31"),
            ("FEP", "32"),
            ("KWS", "33"),
            ("HLK", "34"),
            ("NTL", "35"),
        ] {
            assert!(writer.write_event(Event::Start(BytesStart::new(inner_tags.0))).is_ok());
            assert!(writer.write_event(Event::Text(BytesText::new(inner_tags.1))).is_ok());
            assert!(writer.write_event(Event::End(BytesEnd::new(inner_tags.0))).is_ok());
        }
        assert!(writer.write_event(Event::End(BytesEnd::new("VARIABLE"))).is_ok());
        assert!(writer.write_event(Event::End(BytesEnd::new("FORMAT"))).is_ok());

        assert!(writer.write_event(Event::Start(BytesStart::new("RECORDS"))).is_ok());
        assert!(writer.write_event(Event::Text(BytesText::new("INNER_RECORDS"))).is_ok());
        assert!(writer.write_event(Event::End(BytesEnd::new("RECORDS"))).is_ok());

        assert!(writer.write_event(Event::End(BytesEnd::new("FIRCO-OFAC-AGENT"))).is_ok());
        self.template = writer.into_inner().into_inner();
        self.template_loaded = true;
    }
}

#[cfg(test)]
mod fofdbof {
    use std::io::BufWriter;

    use crate::processor::entity::sdn::{SdnRecordAddress, SdnRecordDocument};

    use super::*;

    fn init_complexe_record() -> SdnRecord {
        let mut record = SdnRecord {
            fixed_ref: 17636,
            name: String::from("TEST RECORD"),
            partysubtypeid: 1,
            last_update: String::from("1970-01-01"),
            ..Default::default()
        };
        record.normal_aliases = {
            let mut aliases = Vec::new();
            for i in 0..3 {
                aliases.push(format!("Alias {}", i));
            }
            aliases
        };

        record.programs = {
            let mut programs = Vec::new();
            for i in 0..3 {
                programs.push(format!("Program {}", i));
            }
            programs
        };

        record.bics = {
            let mut bics = Vec::new();
            for i in 0..3 {
                bics.push(format!("BICTEST {}", i));
            }
            bics
        };

        record.addresses = {
            let mut addresses = Vec::new();
            for i in 0..5 {
                let address = SdnRecordAddress {
                    id: i,
                    country: if i == 2 { Some("Country 1".to_owned()) } else { None },
                    address: if (2..3).contains(&i) { Some("Address 1".to_owned()) } else { None },
                    city: if [1, 5].contains(&i) { Some("City 1".to_owned()) } else { None },
                    region: None,
                    postal_code: Some("Postal Code".to_owned()),
                    state: Some("State 1".to_owned()),
                    is_primary: i == 1,
                };
                addresses.push(address);
            }
            addresses
        };
        record.documents = {
            let mut documents = Vec::new();
            for i in 0..5 {
                let document = SdnRecordDocument {
                    id: i,
                    doc_type: if (0..3).contains(&i) { 1571 } else { 1626 },
                    expiration_date: None,
                    issued_date: None,
                    issued_by: None,
                    doc_type_value: "TEST".to_owned(),
                    registration_number: format!("12345{}", i),
                };
                documents.push(document);
            }
            documents
        };

        record
    }

    fn init_template() -> FofdbofExporter {
        let mut exporter = FofdbofExporter::default();
        exporter.load_template();
        exporter
    }

    #[test]
    fn write_template() {
        let mut buffer = Vec::new();
        let mut exporter = FofdbofExporter::default();
        exporter.load_template();
        exporter.write_in(&mut BufWriter::new(&mut buffer)).unwrap();
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            String::from_utf8(
                br#"<FIRCO-OFAC-AGENT>
 <CREATED>1970-01-01 00:00:00</CREATED>
 <AUTHOR>CAPS OFAC BATCH</AUTHOR>
 <TITLE>OFAC Lists</TITLE>
 <APPLI>OFACXMLAGENT</APPLI>
 <VERSION>0.1.0</VERSION>
 <FORMAT>
  <VARIABLE>
   <EOR>/010</EOR>
   <EOC>/009</EOC>
   <OID>1</OID>
   <NAM>2</NAM>
   <ADD>3</ADD>
   <CIT>4</CIT>
   <CTR>5</CTR>
   <STA>6</STA>
   <TYP>7</TYP>
   <BAD>8</BAD>
   <SHK>9</SHK>
   <SYN>10</SYN>
   <SYC>11</SYC>
   <SYK>12</SYK>
   <SYS>13</SYS>
   <ORI>14</ORI>
   <DSG>15</DSG>
   <US1>16</US1>
   <US2>17</US2>
   <REF>18</REF>
   <BIC>19</BIC>
   <PSP>20</PSP>
   <NID>21</NID>
   <POB>22</POB>
   <DOB>23</DOB>
   <BGH>24</BGH>
   <INF>25</INF>
   <ORH>26</ORH>
   <TGH>27</TGH>
   <IDH>28</IDH>
   <UNH>29</UNH>
   <SRH>30</SRH>
   <PEP>31</PEP>
   <FEP>32</FEP>
   <KWS>33</KWS>
   <HLK>34</HLK>
   <NTL>35</NTL>
  </VARIABLE>
 </FORMAT>
 <RECORDS>
</RECORDS>
</FIRCO-OFAC-AGENT>"#
                    .to_vec()
            )
            .unwrap()
        );
    }

    #[test]
    fn write_multiple_records_with_different_types() {
        let mut exporter = init_template();
        let records = {
            let mut records = Vec::new();
            let mut db_record = SdnRecord::default();
            db_record.addresses = vec![SdnRecordAddress { is_primary: true, ..Default::default() }];
            db_record.last_update = "1970/01/01".to_owned();
            db_record.partysubtypeid = 1;
            for i in 0..6 {
                db_record.fixed_ref = i * 4;
                records.push(db_record.clone());
            }
            records
        };
        exporter.load_from_db_records(&records, &[]);
        assert_eq!(
            "OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000004\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000008\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000012\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000016\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000020\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
",
            String::from_utf8(exporter.get_formatted_records()).unwrap()
        );
    }
    #[test]
    fn write_multiple_records() {
        let mut exporter = init_template();
        let records = {
            let mut records = Vec::new();
            let mut db_record = SdnRecord::default();
            db_record.addresses = vec![SdnRecordAddress { is_primary: true, ..Default::default() }];
            db_record.last_update = "1970/01/01".to_owned();
            db_record.partysubtypeid = 1;
            for i in 0..6 {
                db_record.fixed_ref = i * 4;
                records.push(db_record.clone());
            }
            records
        };
        exporter.load_from_db_records(&records, &[]);
        assert_eq!(
            "OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000004\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000008\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000012\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000016\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000020\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
",
            String::from_utf8(exporter.get_formatted_records()).unwrap()
        );
    }
    #[test]
    fn write_a_complexe_record() {
        let mut exporter = FofdbofExporter::default();
        exporter.load_from_db_records(&[init_complexe_record()], &[]);
        assert_eq!(
"OFACZ00000\tTEST RECORD\tPOSTAL CODE\t\t\tSTATE 1\tV\t0\t\tOFAC017636\t\t\t\tOFAC\tOFAC\tNO\tOFAC017636\tOFAC_1970-01-01\t\t123450 123451 123452\t\t\t\t\tPROGRAM Program 0 / Program 1 / Program 2; PASSPORT 123450 / 123451 / 123452;  TEST 123453 / TEST 123454; ADDRESS  Postal Code State 1 /  Postal Code City 1 State 1 / Address 1 Postal Code State 1 Country 1 /  Postal Code State 1 /  Postal Code State 1;\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC017636\tTEST RECORD\tPOSTAL CODE\tCITY 1\t\tSTATE 1\tV\t0\t\tOFAC017636\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970-01-01\t\t123450 123451 123452\t\t\t\t\tPROGRAM Program 0 / Program 1 / Program 2; PASSPORT 123450 / 123451 / 123452;  TEST 123453 / TEST 123454; ADDRESS  Postal Code State 1 /  Postal Code City 1 State 1 / Address 1 Postal Code State 1 Country 1 /  Postal Code State 1 /  Postal Code State 1;\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFACZ00002\tTEST RECORD\tADDRESS 1, POSTAL CODE\t\tCOUNTRY 1\tSTATE 1\tV\t0\t\tOFAC017636\t\t\t\tOFAC\tOFAC\tNO\tOFAC017636\tOFAC_1970-01-01\t\t123450 123451 123452\t\t\t\t\tPROGRAM Program 0 / Program 1 / Program 2; PASSPORT 123450 / 123451 / 123452;  TEST 123453 / TEST 123454; ADDRESS  Postal Code State 1 /  Postal Code City 1 State 1 / Address 1 Postal Code State 1 Country 1 /  Postal Code State 1 /  Postal Code State 1;\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFACZ00003\tTEST RECORD\tPOSTAL CODE\t\t\tSTATE 1\tV\t0\t\tOFAC017636\t\t\t\tOFAC\tOFAC\tNO\tOFAC017636\tOFAC_1970-01-01\t\t123450 123451 123452\t\t\t\t\tPROGRAM Program 0 / Program 1 / Program 2; PASSPORT 123450 / 123451 / 123452;  TEST 123453 / TEST 123454; ADDRESS  Postal Code State 1 /  Postal Code City 1 State 1 / Address 1 Postal Code State 1 Country 1 /  Postal Code State 1 /  Postal Code State 1;\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFACZ00004\tTEST RECORD\tPOSTAL CODE\t\t\tSTATE 1\tV\t0\t\tOFAC017636\t\t\t\tOFAC\tOFAC\tNO\tOFAC017636\tOFAC_1970-01-01\t\t123450 123451 123452\t\t\t\t\tPROGRAM Program 0 / Program 1 / Program 2; PASSPORT 123450 / 123451 / 123452;  TEST 123453 / TEST 123454; ADDRESS  Postal Code State 1 /  Postal Code City 1 State 1 / Address 1 Postal Code State 1 Country 1 /  Postal Code State 1 /  Postal Code State 1;\t\t\t\t\t\t0\t0\t\t\t\tU\t
",
            String::from_utf8(exporter.get_formatted_records()).unwrap()
        );
    }

    #[test]
    fn write_a_record_with_alternative_address() {
        let mut exporter = FofdbofExporter::default();
        let record = {
            let mut db_record = SdnRecord::default();
            db_record.addresses = vec![
                SdnRecordAddress { is_primary: true, ..Default::default() },
                SdnRecordAddress { is_primary: false, ..Default::default() },
                SdnRecordAddress { is_primary: false, ..Default::default() },
            ];
            db_record.last_update = "1970/01/01".to_owned();
            db_record
        };
        exporter.load_from_db_records(&[record], &[]);
        assert_eq!(
            "OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFACZ00000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\tOFAC000000\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFACZ00000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\tOFAC000000\tOFAC_1970/01/01\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
",
            String::from_utf8(exporter.get_formatted_records()).unwrap()
        );
    }

    #[test]
    fn write_a_records_with_different_gender() {
        let mut exporter = FofdbofExporter::default();
        let mut records = Vec::new();
        for i in 0..10 {
            let mut db_record = SdnRecord::default();
            db_record.gender = if i % 2 == 0 {
                "MALE".to_owned()
            } else if i == 9 {
                "FEMALE".to_owned()
            } else {
                "".to_owned()
            };
            records.push(db_record);
        }
        exporter.load_from_db_records(&records, &[]);
        assert_eq!(
            "OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_\t\t\t\t\t\t\tGENDER MALE;\t\t\t\t\t\t0\t0\t\t\t\tM\t
OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_\t\t\t\t\t\t\tGENDER MALE;\t\t\t\t\t\t0\t0\t\t\t\tM\t
OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_\t\t\t\t\t\t\tGENDER MALE;\t\t\t\t\t\t0\t0\t\t\t\tM\t
OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_\t\t\t\t\t\t\tGENDER MALE;\t\t\t\t\t\t0\t0\t\t\t\tM\t
OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_\t\t\t\t\t\t\t\t\t\t\t\t\t\t0\t0\t\t\t\tU\t
OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_\t\t\t\t\t\t\tGENDER MALE;\t\t\t\t\t\t0\t0\t\t\t\tM\t
OFAC000000\t\t\t\t\t\tV\t0\t\t\t\t\t\tOFAC\tOFAC\tNO\t\tOFAC_\t\t\t\t\t\t\tGENDER FEMALE;\t\t\t\t\t\t0\t0\t\t\t\tF\t
",
            String::from_utf8(exporter.get_formatted_records()).unwrap()
        );
    }
}
