#[macro_use]
extern crate clap;
extern crate rusoto_s3;
extern crate rusoto_core;

mod bucket;

use clap::{App, ArgMatches};
use rusoto_s3::{S3Client};
use rusoto_core::{region};
use bucket::{upload, download};

// Flow of the application
// Set the AWS profile for the commercial role with bucket access
// Download the contents of the commercial bucket
// Set AWS profile for the GovCloud role
// Upload was downloaded from the commercial bucket into the GovCloud bucket
fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml).version("0.1.0");
    let matches = app.get_matches().clone();
    let profile= matches.value_of("profile");

    if let Some(config) = matches.value_of("config") {
        println!("Using the config file: {}", config)
    } else {
        println!("No config file found. Using the defaults")
    }

    match profile {
        Some(p) => println!("You provided the profile, {}", p),
        None => println!("No profile provided. Falling back to using IAM default"),
    }

    let client = rusoto_s3::S3Client::new(region::Region::UsWest2);
    let download_result = download(&client, String::from("path"));
    let upload_result= download(&client, String::from("path"));

}
