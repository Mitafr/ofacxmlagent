pub mod fofdbof;
pub mod fofnasy;

use crate::config::Config;

pub use super::*;

#[async_trait]
pub trait Exporter {
    /// Flushes the Exporter
    ///
    /// self.filename will be created
    /// and old content (if file already exists) will be erased after this
    fn flush(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// Process the Exporter with parameters
    ///
    /// * `db` - Initialiazed DatabaseConnection where Exporter will fetch data.
    /// * `doc_type` - DocumentType.
    /// * `config` - &Config initialized Config.
    async fn process(&mut self, db: &DatabaseConnection, doc_type: &DocumentType, config: &Config) -> Result<(), Box<dyn Error>>;

    /// Exporter will write formatted data in provided Buffer
    ///
    /// * `buffer` - Mutable buffer with Write trait
    fn write_in<W: Write>(&self, buffer: &mut W) -> Result<(), Box<dyn Error>>;
}
