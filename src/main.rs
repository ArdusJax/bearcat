#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate simplelog;

extern crate regex;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_s3;
extern crate rusoto_sts;
extern crate rusoto_ssm;
extern crate futures;
extern crate bytes;

mod bucket;
mod credentials;
mod data;

use bucket::{download, upload, get_bucket_object_keys};
use clap::{App, ArgMatches};
use credentials::{CredentialProvider};
use log::{info, warn};
use std::env;
use std::fs::File;
use std::str::FromStr;
use rusoto_core::region::Region;
use rusoto_core::request::{HttpClient};
use rusoto_credential::{ChainProvider, StaticProvider};
use rusoto_s3::S3Client;
use simplelog::*;
// Flow of the application
// Set the AWS profile for the commercial role with bucket access
// Download the contents of the commercial bucket
// Set AWS profile for the GovCloud role
// Upload was downloaded from the commercial bucket into the GovCloud bucket
fn main() {
    // Initialize the logger
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
    info!(target: "INITIALIZATION", "Loading the application cli");
    // Initialize the application
    let yaml = load_yaml!("cli.yml");
    let app = App::from(yaml).version("0.1.0");
    // Gather the config/app args
    info!(target: "INITIALIZATION", "Parsing cli parameters");
    let matches = app.get_matches();
    let profile = matches.value_of("profile"); // Get rid of this option
    let ssm_key = matches.value_of("ssm_key");
    let source_bucket = matches.value_of("source");
    let destination_bucket = matches.value_of("destination");
    let source_region = matches.value_of("region_source").unwrap_or("us-west-2");
    let destination_region = matches
        .value_of("region_destination")
        .unwrap_or("us-west-2");

    if let Some(config) = matches.value_of("config") {
        info!(target: "ARGUMENTS", "Using the config file: {}", config)
    } else {
        info!(target: "ARGUMENTS", "No config file found. Using the defaults")
    }

    // These can be simplified as well
    match source_bucket {
        Some(p) => info!(target: "ARGUMENTS", "You provided the source bucket, {}", p),
        None => panic!("No source bucket provided. You need to specify a source bucket"),
    }

    match destination_bucket {
        Some(p) => info!(target: "ARGUMENTS", "You provided the destination bucket, {}", p),
        None => panic!("No destination bucket provided. You need to specify a source bucket"),
    }

    match profile {
        Some(p) => info!(target: "ARGUMENTS", "You provided the profile, {}", p),
        None => info!(target:
            "ARGUMENTS",
            "No profile provided. Falling back to using IAM default"
        ),
    }
    match ssm_key {
        Some(p) => info!(target: "ARGUMENTS", "You provided the ssm key, {}", p),
        None => info!(target:
            "ARGUMENTS",
            "No ssm key provided. Falling back to using IAM default"
        ),
    }

    ////// MOVE/BREAK THIS OUT ////////////
    // Use the chain provider credentials for access to ssm
    // This will move to using just the containerprovider credentials provider later
    info!(target: "CRED EVENTS", "Creating the credentials provider");
    let cred_prov = CredentialProvider::AwsSsm {
        key: ssm_key.unwrap().to_owned(),
        region: Region::from_str(&source_region).unwrap(),
    };
    let sync_creds = cred_prov.get_credentials().unwrap();
    info!(target: "CRED EVENTS", "Sync Creds:\n{:?}", &sync_creds);
    // Upload the artifact from the local machine to the destination bucket
    info!(target: "UPLOAD CLIENT", "Creating upload client...");
    let upload_client = rusoto_s3::S3Client::new_with(
        HttpClient::new().unwrap(),
        StaticProvider::from(sync_creds),
        Region::from_str(&destination_region).unwrap(),
    );
    info!(target: "UPLOAD CLIENT", "Upload client created successfully");

    // The path, region, etc... will come from environment variables, command line args or can be
    // parsed out of a config file if that is present.
    info!(target: "DOWNLOAD CLIENT", "Creating download client...");
    let client = rusoto_s3::S3Client::new_with(
        HttpClient::new().unwrap(),
        ChainProvider::new(),
        Region::from_str(&source_region).unwrap(),
    );
    info!(target: "DOWNLOAD CLIENT", "Download client created successfully");
    // If there are objects in the bucket then get all of the objects and
    // sync them over to the destination bucket
    if let Ok(file_names) = get_bucket_object_keys(&client, source_bucket.unwrap()) {
        for file_name in file_names {
            // Download the artifact from the source S3 bucket
            let download_result = download(&client, &file_name, source_bucket.unwrap());
            match download_result {
                Ok(res) => info!(
                    target:
                    "S3 DOWNLOAD",
                    "Download of {:?} completed successfully!\n{:?}", file_name, res
                ),
                Err(e) => panic!(format!(
                    "Download of {:?} failed with error:\n{:?}",
                    file_name, e
                )),
            }

            let upload_result = upload(
                &upload_client,
                "data",
                &file_name,
                destination_bucket.unwrap(),
            );
            match upload_result {
                Ok(res) => info!(target:"UPLOAD", "Upload was successful!\n{:?}", res),
                Err(e) => panic!(format!("Could not upload artifact...\nError:\n{:?}", e)),
            }
        }
    }
    ////// MOVE/BREAK THIS OUT END //////////////////
}
