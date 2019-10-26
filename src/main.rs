#[macro_use]
extern crate clap;


use clap::{App, ArgMatches};

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
    //let config = matches.value_of("config").unwrap().to_owned();
    //let username = matches.value_of("verbose").unwrap().to_owned();

    // you could do this
    if let Some(config) = matches.value_of("config") {
        println!("You have a config file: {}", config)
    } else {
        println!("Dude... seriously??!! No config file?!")
    }

    // or this
    let username = matches.value_of("username").unwrap_or("fred");
    println!("Username: {}", username);

    // or maybe this... so many choices for you!
    match profile {
        Some(p) => println!("You provided the profile, {}", p),
        None => println!("WTF... you need a profile bro!!"),
    }
}
