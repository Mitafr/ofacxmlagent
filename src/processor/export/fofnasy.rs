use super::Exporter;
use async_trait::async_trait;
use std::{error::Error, fs::OpenOptions, io::Write, path::Path};

use crate::config::Config;
use crate::db::find_fixed_ref_with_names;
use crate::document::outputs::FofnasyRecord;
use crate::document::DocumentType;
use sea_orm::DatabaseConnection;

pub struct FofnasyExporter {
    pub filepath: String,
    doc_type: DocumentType,
    records: Vec<FofnasyRecord>,
}

impl Default for FofnasyExporter {
    fn default() -> Self {
        let path = format!("./output/{}", String::from("FOFNASY.t"));
        if Path::new(&path).exists() {
            std::fs::remove_file(&path).unwrap();
        }
        Self {
            filepath: path,
            records: Vec::new(),
            doc_type: DocumentType::OFAC,
        }
    }
}

#[async_trait]
impl Exporter for FofnasyExporter {
    async fn process(&mut self, db: &DatabaseConnection, doc_type: &DocumentType, _config: &Config) -> Result<(), Box<dyn Error>> {
        self.doc_type = *doc_type;
        for (fixed_ref, aliases) in find_fixed_ref_with_names(db).await? {
            for alias in aliases {
                self.records.push(FofnasyRecord {
                    doc_type: *doc_type,
                    t_id: fixed_ref,
                    t_alias: alias,
                });
            }
        }
        Ok(())
    }

    fn flush(&self) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new().read(true).write(true).append(true).create(true).truncate(false).open(&self.filepath).unwrap();
        self.write_in(&mut file)?;
        Ok(())
    }

    fn write_in<W: Write>(&self, buffer: &mut W) -> Result<(), Box<dyn Error>> {
        for record in self.records.iter() {
            let doc_type = match record.doc_type {
                DocumentType::OFAC => "OFAC",
                DocumentType::OFACNS => "OFNS",
            };
            buffer
                .write_all(
                    format!(
                        "{}{:0>zeros$}{: <spaces$}\n",
                        doc_type,
                        record.t_id,
                        record.t_alias.to_uppercase(),
                        zeros = 6,
                        spaces = if record.t_alias.len() <= 4 { 300 } else { 0 }
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
        buffer.flush().unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod fofdbof {
    use std::io::BufWriter;

    use super::*;

    impl FofnasyExporter {
        fn load_records(&mut self, records: &[(i32, Vec<String>)], doc_type: &DocumentType) {
            self.doc_type = *doc_type;
            for (fixed_ref, aliases) in records {
                for alias in aliases {
                    self.records.push(FofnasyRecord {
                        doc_type: *doc_type,
                        t_id: *fixed_ref,
                        t_alias: alias.clone(),
                    });
                }
            }
        }
    }

    fn init_simple_records() -> Vec<(i32, Vec<String>)> {
        let mut records = Vec::new();

        for i in 0..5 {
            let mut aliases = Vec::new();
            for j in 0..2 {
                aliases.push(format!("ALIAS{}{}", j, i));
            }
            records.push((i, aliases))
        }
        records
    }

    #[test]
    fn write_simple_records() {
        let mut buffer = Vec::new();
        let mut exporter = FofnasyExporter::default();
        let records = init_simple_records();
        exporter.load_records(&records, &DocumentType::OFAC);
        exporter.write_in(&mut BufWriter::new(&mut buffer)).unwrap();
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            String::from_utf8(
                br#"OFAC000000ALIAS00
OFAC000000ALIAS10
OFAC000001ALIAS01
OFAC000001ALIAS11
OFAC000002ALIAS02
OFAC000002ALIAS12
OFAC000003ALIAS03
OFAC000003ALIAS13
OFAC000004ALIAS04
OFAC000004ALIAS14
"#
                .to_vec()
            )
            .unwrap()
        );
    }
}
