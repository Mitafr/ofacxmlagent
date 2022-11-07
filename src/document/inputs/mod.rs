use chrono::Local;
use log::info;
use quick_xml::de::from_str;
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use std::error::Error;
use std::path::{Path, PathBuf};

pub mod models;

use self::models::areacode::AreaCode;
use self::models::dateofissue::DateOfIssue;
use self::models::distinctparty::DistinctParties;
use self::models::document::IDRegDocuments;
use self::models::location::Locations;
use self::models::profilerelationship::ProfileRelationships;
use self::models::referencevaluesets::{DetailReferenceValues, FeatureTypeValues, IDRegDocTypeValues, PartySubTypeValues, ReferenceValueSets, ScriptValues};
use self::models::sanction::SanctionsEntries;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DocumentType {
    OFAC,
    OFACNS,
}

impl Default for DocumentType {
    fn default() -> Self {
        Self::OFAC
    }
}

#[derive(Default)]
pub struct OfacDocumentReferences {
    pub area_codes: Vec<AreaCode>,
    pub date_of_issue: DateOfIssue,
    pub detail_references: DetailReferenceValues,
    pub feature_types: FeatureTypeValues,
    pub party_sub_type_values: PartySubTypeValues,
    pub reg_doc_types: IDRegDocTypeValues,
    pub script_values: ScriptValues,
}

#[derive(Default)]
pub struct OfacDocument {
    pub distinct_parties: DistinctParties,
    pub document_type: DocumentType,
    pub documents: IDRegDocuments,
    pub locations: Locations,
    pub profile_relationships: ProfileRelationships,
    pub references: OfacDocumentReferences,
    pub root_folder: PathBuf,
    pub sanction_entries: SanctionsEntries,
    pub is_loaded: bool,
}

impl OfacDocument {
    pub fn new<P>(folder: P, document_type: DocumentType) -> OfacDocument
    where
        P: AsRef<Path>,
    {
        OfacDocument {
            document_type,
            root_folder: folder.as_ref().to_owned(),
            ..Default::default()
        }
    }

    /// Load the current document from file
    ///
    /// Reading and loading xml with needed tag
    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        let filename = &self.file_name()[..];
        let file = self.root_folder.join(filename);
        info!("Reading xml file : {:?}", self.root_folder.join(filename));
        let xml = match std::fs::read_to_string(&file) {
            Ok(content) => content,
            Err(err) => return Err(format!("Ofac document not found in {}, {}", file.to_string_lossy(), err).into()),
        };
        info!("Splitting xml ...");
        let mut reader = Reader::from_str(&xml);
        reader.trim_text(true).check_comments(false).expand_empty_elements(true).check_end_names(false);
        let mut start = 0;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name() == QName(b"DateOfIssue").into() => {
                    // Buf doesn't contain [<>] delimiters so we have to add 2
                    start = reader.buffer_position() - (buf.len() + 2);
                }
                Ok(Event::End(ref e)) if e.local_name() == QName(b"DateOfIssue").into() => {
                    self.references.date_of_issue = from_str(&xml[start..reader.buffer_position()])?;
                }
                Ok(Event::Start(ref e)) if e.local_name() == QName(b"IDRegDocuments").into() => {
                    start = reader.buffer_position() - (buf.len() + 2);
                }
                Ok(Event::End(ref e)) if e.local_name() == QName(b"IDRegDocuments").into() => {
                    self.documents = from_str(&xml[start..reader.buffer_position()])?;
                }
                Ok(Event::Start(ref e)) if e.local_name() == QName(b"ReferenceValueSets").into() => {
                    start = reader.buffer_position() - (buf.len() + 2);
                }
                Ok(Event::End(ref e)) if e.local_name() == QName(b"ReferenceValueSets").into() => {
                    let references: ReferenceValueSets = from_str(&xml[start..reader.buffer_position()])?;
                    self.references.area_codes = references.area_code_values.area_codes;
                    self.references.detail_references = references.detail_reference_values;
                    self.references.feature_types = references.feature_types;
                    self.references.party_sub_type_values = references.party_sub_type_values;
                    self.references.reg_doc_types = references.reg_doc_types_values;
                    self.references.script_values = references.script_values;
                }
                Ok(Event::Start(ref e)) if e.local_name() == QName(b"DistinctParties").into() => {
                    start = reader.buffer_position() - (buf.len() + 2);
                }
                Ok(Event::End(ref e)) if e.local_name() == QName(b"DistinctParties").into() => {
                    self.distinct_parties = from_str(&xml[start..reader.buffer_position()])?;
                }
                Ok(Event::Start(ref e)) if e.local_name() == QName(b"ProfileRelationships").into() => {
                    start = reader.buffer_position() - (buf.len() + 2);
                }
                Ok(Event::End(ref e)) if e.local_name() == QName(b"ProfileRelationships").into() => {
                    self.profile_relationships = from_str(&xml[start..reader.buffer_position()])?;
                }
                Ok(Event::Start(ref e)) if e.local_name() == QName(b"Locations").into() => {
                    start = reader.buffer_position() - (buf.len() + 2);
                }
                Ok(Event::End(ref e)) if e.local_name() == QName(b"Locations").into() => {
                    self.locations = from_str(&xml[start..reader.buffer_position()]).unwrap();
                }
                Ok(Event::Start(ref e)) if e.local_name() == QName(b"SanctionsEntries").into() => {
                    start = reader.buffer_position() - (buf.len() + 2);
                }
                Ok(Event::End(ref e)) if e.local_name() == QName(b"SanctionsEntries").into() => {
                    self.sanction_entries = from_str(&xml[start..reader.buffer_position()])?;
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
            buf.clear();
        }
        info!("Xml splitted, data processing has started ...");
        self.is_loaded = true;
        Ok(())
    }

    /// Get the filename for archive purpose from the current Ofac document type
    fn archive_file_name(&self) -> String {
        match self.document_type {
            DocumentType::OFAC => {
                format!("sdn_advanced{}.xml", Local::now().format("%Y%m%d_%H%M%S"))
            }
            DocumentType::OFACNS => {
                format!("cons_advanced{}.xml", Local::now().format("%Y%m%d_%H%M%S"))
            }
        }
    }

    /// Get the filename from the current Ofac document type
    fn file_name(&self) -> String {
        match self.document_type {
            DocumentType::OFAC => "sdn_advanced.xml".to_owned(),
            DocumentType::OFACNS => "cons_advanced.xml".to_owned(),
        }
    }

    /// Cleans the current ofac data folder.
    ///
    /// The current document loaded is moved to archive folder in the data folder and
    /// oldest archived files are removed if there is more than 5 files in the archive folder
    pub fn cleanup(&self) -> Result<(), std::io::Error> {
        let archive_folder = &self.root_folder.join("archive");
        std::fs::create_dir_all(archive_folder)?;
        let mut paths: Vec<_> = std::fs::read_dir(archive_folder).unwrap().map(|r| r.unwrap()).collect();
        paths.sort_by_key(|dir| dir.path());
        let count = paths.len();
        for (i, path) in paths.iter().enumerate() {
            if count < 5 || i >= count - 5 {
                break;
            }
            std::fs::remove_file(path.path())?;
        }
        let to = archive_folder.join(self.archive_file_name());
        info!("Backing up {} to {}", self.file_name(), to.to_string_lossy());
        //std::fs::rename(&self.root_folder.join(self.file_name()), to)?;
        Ok(())
    }
}
