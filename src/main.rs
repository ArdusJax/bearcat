#[macro_use]
extern crate clap;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_s3;
extern crate rusoto_sts;
extern crate futures;
extern crate bytes;

mod bucket;
mod credentials;

use bucket::{download, upload, get_bucket_object_keys};
use clap::{App, ArgMatches};
use rusoto_core::region;
use rusoto_s3::S3Client;

// Flow of the application
// Set the AWS profile for the commercial role with bucket access
// Download the contents of the commercial bucket
// Set AWS profile for the GovCloud role
// Upload was downloaded from the commercial bucket into the GovCloud bucket
fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from(yaml).version("0.1.0");
    let matches = app.get_matches();
    let profile = matches.value_of("profile");
    let source_bucket = matches.value_of("source");
    let destination_bucket = matches.value_of("destination");
    let region = matches.value_of("region");

    if let Some(config) = matches.value_of("config") {
        println!("Using the config file: {}", config)
    } else {
        println!("No config file found. Using the defaults")
    }

    match source_bucket {
        Some(p) => println!("You provided the source bucket, {}", p),
        None => panic!("No source bucket provided. You need to specify a source bucket"),
    }

    match destination_bucket {
        Some(p) => println!("You provided the destination bucket, {}", p),
        None => panic!("No destination bucket provided. You need to specify a source bucket"),
    }

    match profile {
        Some(p) => println!("You provided the profile, {}", p),
        None => println!("No profile provided. Falling back to using IAM default"),
    }

    // The path, region, etc... will come from environment variables, command line args or can be
    // parsed out of a config file if that is present.
    let client = rusoto_s3::S3Client::new(region::Region::UsWest2);
    // If there are objects in the bucket then get all of the objects and
    // sync them over to the destination bucket
    if let Ok(file_names) = get_bucket_object_keys(&client, source_bucket.unwrap()) {
        for file_name in file_names {
            // Download the artifact from the source S3 bucket
            let download_result = download(&client, &file_name, source_bucket.unwrap());
            match download_result {
                Ok(res) => println!(
                    "Download of {:?} completed successfully!\n{:?}",
                    file_name, res
                ),
                Err(e) => panic!(format!(
                    "Download of {:?} failed with error:\n{:?}",
                    file_name, e
                )),
            }
            // Upload the artifact from the local machine to the destination bucket
            let upload_result = upload(&client, "path", &file_name, destination_bucket.unwrap());
            match upload_result {
                Ok(res) => println!("Upload was successful!\n{:?}", res),
                Err(e) => panic!(format!("Could not upload artifact...\nError:\n{:?}", e)),
            }
        }
    }
}
