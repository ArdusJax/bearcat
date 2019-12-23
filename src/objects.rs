use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

pub fn create_file_parts(path: &str) -> std::io::Result<()> {
    let mut buffer = File::open(path)?;
}