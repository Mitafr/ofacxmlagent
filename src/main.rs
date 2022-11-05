use log::{error, info};
use ofacxmlagent::config::*;
use ofacxmlagent::db::*;
use ofacxmlagent::document::*;
use ofacxmlagent::processor::export::fofdbof::FofdbofExporter;
use ofacxmlagent::processor::export::fofnasy::FofnasyExporter;
use ofacxmlagent::processor::export::Exporter;
use ofacxmlagent::processor::import::Importer;
use std::{error::Error, time::Instant};

use clap::Parser;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let now = Instant::now();
    dotenv::dotenv().ok();
    let mut args = Args::parse();
    let mut configs = load_configs(&mut args);
    match &args.mode[..] {
        "import" => import_mode(&mut configs).await?,
        "export" => export_mode(&mut configs, &args).await?,
        _ => error!("Mode is not recognized : {}", &args.mode[..]),
    }
    info!("Elapsed time : {}.{}s", now.elapsed().as_secs(), now.elapsed().as_millis());
    Ok(())
}

fn load_configs(args: &mut Args) -> Vec<Config> {
    let mut configs = Vec::new();
    if args.datatype == "ALL" && args.mode == "import" {
        args.datatype = "OFACNS".to_owned();
        configs.push(Config::init(args).expect("Could not init config"));
        args.datatype = "OFAC".to_owned();
        configs.push(Config::init(args).expect("Could not init config"));
    } else if args.mode == "export" {
        let mut tmp_args = args.clone();
        tmp_args.datatype = "OFAC".to_owned();
        configs.push(Config::init(&tmp_args).expect("Could not init config"));
        tmp_args.datatype = "OFACNS".to_owned();
        configs.push(Config::init(&tmp_args).expect("Could not init config"));
    } else {
        configs.push(Config::init(args).expect("Could not init config"));
    }
    configs
}

async fn export_mode(configs: &mut [Config], args: &Args) -> Result<(), Box<dyn Error>> {
    match &args.datatype[..] {
        "FOFDBOF" => {
            let mut exporter = FofdbofExporter::default();
            for (i, config) in configs.iter_mut().enumerate() {
                if i == 0 {
                    config.init_logging();
                }
                let db = init_db(config).await.map_err(|err| exit(Box::new(err))).unwrap();
                exporter.process(&db, &config.data_type, config).await?;
            }
            exporter.flush()?;
            info!("FOFDBOF successfully saved to {}", exporter.filepath);
        }
        "FOFNASY" => {
            let mut exporter = FofnasyExporter::default();
            for (i, config) in configs.iter_mut().enumerate() {
                if i == 0 {
                    config.init_logging();
                }
                let db = init_db(config).await.map_err(|err| exit(Box::new(err))).unwrap();
                exporter.process(&db, &config.data_type, config).await?;
            }
            exporter.flush()?;
            info!("FOFNASY successfully saved to {}", exporter.filepath);
        }
        _ => panic!("Export must be FOFNASY or FOFDBOF"),
    }
    Ok(())
}

async fn import_mode(configs: &mut [Config]) -> Result<(), Box<dyn Error>> {
    for (i, config) in configs.iter_mut().enumerate() {
        if i == 0 {
            config.init_logging();
        }
        let db = init_db(config).await.map_err(|err| exit(Box::new(err))).unwrap();
        let mut ofac_document = inputs::OfacDocument::new(&config.get_data_folder_path(), config.data_type);
        ofac_document.load().map_err(|err| exit(err)).unwrap();
        let mut importer = Importer::init(&db).await;
        match importer.process_document(&db, &ofac_document, config.force).await {
            Ok(_) => {
                importer.commit().await?;
                ofac_document.cleanup().map_err(|err| exit(Box::new(err))).unwrap();
            }
            Err(_) => info!("This document has beed skipped"),
        }
    }
    Ok(())
}
fn exit(err: Box<dyn std::error::Error>) {
    log::error!("{:?}", err);
    log::info!("Exiting...");
    std::process::exit(1);
}
