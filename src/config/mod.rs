use std::{env, error::Error, path::PathBuf};

use log::{debug, info, warn, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};

use crate::document::DocumentType;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Import mode
    #[clap(short = 'm', long, default_value = "import", value_parser = mode_parsing)]
    pub mode: String,
    /// Debug mode (sql output included)
    #[clap(short = 'd', long, action = clap::ArgAction::SetTrue, default_value = "false")]
    debug: bool,
    /// Ofac data type
    #[clap(short = 't', name = "type", long, default_value = "ALL", value_parser = data_type_parsing)]
    pub datatype: String,
    /// Force import (i.e same date in db)
    #[clap(short = 'f', long, action = clap::ArgAction::SetTrue, default_value = "false")]
    force: bool,
}

fn mode_parsing(s: &str) -> Result<String, &'static str> {
    match s {
        "import" => Ok(String::from(s)),
        "export" => Ok(String::from(s)),
        _ => Err("mode must be `import` or `export`"),
    }
}

fn data_type_parsing(s: &str) -> Result<String, &'static str> {
    match s {
        "OFAC" => Ok(String::from(s)),
        "OFACNS" => Ok(String::from(s)),
        "FOFDBOF" => Ok(String::from(s)),
        "FOFNASY" => Ok(String::from(s)),
        "ALL" => Ok(String::from(s)),
        _ => Err("\ndatatype for import must be one of `OFAC` | `OFACNS` | `ALL`\n
		datatype for export must be one of | `FOFDBOF` | `FOFNASY` | `ALL`"),
    }
}

#[derive(Debug)]
pub struct Config {
    pub data_type: DocumentType,
    pub debug: bool,
    pub mode: String,
    pub force: bool,
    loaded: bool,
}

impl Config {
    pub fn init(args: &Args) -> Result<Config, Box<dyn Error>> {
        let data_type = match args.datatype.as_str() {
            "OFAC" => DocumentType::OFAC,
            "OFACNS" => DocumentType::OFACNS,
            _ => return Err(format!("Data type {} not recognized (must be one of `OFAC` | `OFACNS`)", args.datatype).into()),
        };
        let config = Config {
            data_type,
            debug: args.debug,
            mode: args.mode.to_owned(),
            loaded: false,
            force: args.force,
        };
        info!("Config has been loadded successfully (force mode: {})", if config.force { "enabled" } else { "disabled" });
        debug!("Config values {:?}", config);
        Ok(config)
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn init_logging(&mut self) {
        if self.loaded {
            return;
        }

        let mut log_path = PathBuf::from("logs");
        if !log_path.is_dir() {
            warn!("logs directory doesn't exist");
        }
        log_path.push("ofacxmlparser.log");
        if !log_path.is_file() {
            warn!("logs file doesn't exist and will be created");
        }
        let stdout = ConsoleAppender::builder().encoder(Box::new(PatternEncoder::new("{d(%H:%M:%S.%f)} {l} {M}:{L} - {m}{n}"))).build();
        let in_file = FileAppender::builder().encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S.%f)} {l} - {m}{n}"))).build(log_path).unwrap();

        let filter = if self.debug { LevelFilter::Debug } else { LevelFilter::Info };

        let config = log4rs::Config::builder()
            .appender(Appender::builder().filter(Box::new(ThresholdFilter::new(LevelFilter::Info))).build("stdout", Box::new(stdout)))
            .appender(Appender::builder().filter(Box::new(ThresholdFilter::new(filter))).build("in_file", Box::new(in_file)))
            .build(Root::builder().appender("stdout").appender("in_file").build(filter))
            .unwrap();

        log4rs::init_config(config).unwrap();
        info!("Config has been loadded successfully (force mode: {})", if self.force { "enabled" } else { "disabled" });
        self.loaded = true;
    }
    pub fn get_connection_string(&self) -> String {
        match self.data_type {
            DocumentType::OFAC => env::var("OFAC_DATABASE_URL").expect("OFAC_DATABASE_URL environment variable must be set"),
            DocumentType::OFACNS => env::var("OFAC_NS_DATABASE_URL").expect("OFAC_NS_DATABASE_URL environment variable must be set"),
        }
    }
    pub fn get_ddc_connection_string(&self) -> String {
        env::var("DDC_DATABASE_URL").expect("DDC_DATABASE_URL environment variable must be set")
    }
    pub fn get_mock_connection_string(&self) -> String {
        env::var("DATABASE_URL").unwrap()
    }
    pub fn get_database_name(&self) -> String {
        match self.data_type {
            DocumentType::OFAC => {
                let connection_str = env::var("OFAC_DATABASE_URL").expect("OFAC_DATABASE_URL environment variable must be set");
                connection_str[connection_str.rfind('/').unwrap()..connection_str.len()].to_owned()
            }
            DocumentType::OFACNS => {
                let connection_str = env::var("OFAC_NS_DATABASE_URL").expect("OFAC_NS_DATABASE_URL environment variable must be set");
                connection_str[connection_str.rfind('/').unwrap()..connection_str.len()].to_owned()
            }
        }
    }
    pub fn get_ddc_database_name(&self) -> String {
        let connection_str = env::var("DDC_DATABASE_URL").expect("DDC_DATABASE_URL environment variable must be set");
        connection_str[connection_str.rfind('/').unwrap()..connection_str.len()].to_owned()
    }
    pub fn get_data_folder_path(&self) -> String {
        match self.data_type {
            DocumentType::OFAC => env::var("OFAC_DATA_FOLDER").expect("OFAC_DATA_FOLDER environment variable must be set"),
            DocumentType::OFACNS => env::var("OFAC_NS_DATA_FOLDER").expect("OFAC_NS_DATA_FOLDER environment variable must be set"),
        }
    }
}
