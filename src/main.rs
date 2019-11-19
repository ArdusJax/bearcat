#[macro_use]
extern crate clap;
extern crate rusoto_s3;
extern crate rusoto_core;

mod bucket;

use clap::{App, ArgMatches};
use rusoto_s3::{S3Client};
use rusoto_core::{region};
use bucket::{upload, download, processes_object};

// Flow of the application
// Set the AWS profile for the commercial role with bucket access
// Download the contents of the commercial bucket
// Set AWS profile for the GovCloud role
// Upload was downloaded from the commercial bucket into the GovCloud bucket
fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml).version("0.1.0");
    let matches = app.get_matches().clone();
    let profile = matches.value_of("profile");
    let source_bucket = matches.value_of("source");
    let destination_bucket = matches.value_of("destination");

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
    let sour_bucket = "bearcat-commercial";
    let dest_bucket = "bearcat-commercial";
    //let client = rusoto_s3::S3Client::new(region::Region::UsEast1);
    //let download_result = download(&client, "path", sour_bucket);
    //let upload_result= upload(&client, "path", "test.txt", dest_bucket);
}
