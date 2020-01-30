use std::fs::File;
use std::fs;
use std::error::Error;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use bytes::Bytes;
use log::{info, warn};

struct DataFile {}

pub fn delete_data_file(base: &str, key: &str) -> Result<bool, Box<dyn Error>> {
    // Duplicate code that will get removed after I go over this again
    let object = format!("{}/{}", base, key).to_string();
    let object_path = Path::new(&object);
    if object_path.exists() {
        fs::remove_file(object_path)?;
    };
    info!(target: "DELETE", "Deleted data file {:?} successfully", object);
    Ok(true)
}

pub fn create_data_file(
    base: &str,
    key: &str,
    content: &bytes::Bytes,
) -> Result<bool, Box<dyn Error>> {
    let object = format!("{}/{}", base, key).to_string();
    let object_path = Path::new(&object);
    if !object_path.exists() {
        // Create a directory for the object
        // Panics if you can't create the directory
        let mut create_path = object_path.to_str().unwrap();
        if !create_path.ends_with('/') {
            create_path = object_path.parent().unwrap().to_str().unwrap();
        }
        fs::create_dir_all(create_path).expect(
            format!(
                "failed to create directory {:?} for download",
                &object_path.as_os_str()
            )
            .as_str(),
        );
        if !Path::new(object_path).is_dir() {
            let mut file =
                File::create(&object_path).expect("unable to write the file for download");
            file.write_all(&content)
                .expect("failed to write body to the file");
        }
    }
    // The path of the file created might be a better return type than bool?
    info!(target: "CREATED", "Created data file {:?} successfully", object);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes;

    const FILE_CONTENT: &str = "I prematurely shot my wad on what was supposed to be a dry run..";
    static TEST_DIR: &str = "file_tests";
    const FILE_OBJECTS: [&'static str; 7] = [
        "file_test",
        "$sec)",
        "",
        "shared",
        "se90",
        "fake_file/bremerton",
        "fake_file/another_deep/anderson",
    ];
    const FILE_OBJECTS_ABNORMAL: [&'static str; 7] = [
        "fil42$est",
        "$sec)",
        "",
        "c321",
        "se90__",
        "d!(&$^#",
        "$$__",
    ];
    const FILE_OBJECTS_DIRECTORIES: [&'static str; 7] = [
        "flin/",
        "sampl3/",
        "suds/",
        "sample/nested/",
        "sample/nested/another_nest/",
        "flack/",
        "Simple/Nested/",
    ];

    #[test]
    fn create_file_normal_names_test() {
        let _file = bytes::Bytes::from(FILE_CONTENT);
        for (_, _object) in FILE_OBJECTS.iter().enumerate() {
            let _file_path = format!("{}/{}", TEST_DIR, _object).to_string();
            let res = create_data_file(TEST_DIR, _object, &_file);
            assert_eq!(res.unwrap(), true);
            assert_eq!(Path::new(&_file_path).exists(), true);
        }
    }
    #[test]
    fn create_file_abnormal_names_test() {
        let _file = bytes::Bytes::from(FILE_CONTENT);

        for (_, _object) in FILE_OBJECTS_ABNORMAL.iter().enumerate() {
            let _file_path = format!("{}/{}", TEST_DIR, _object).to_string();
            let res = create_data_file(TEST_DIR, _object, &_file);
            assert_eq!(res.unwrap(), true);
            assert_eq!(Path::new(&_file_path).exists(), true);
        }
    }
    #[test]
    fn create_object_dir_test() {
        let _file = bytes::Bytes::from(FILE_CONTENT);

        for (_, _object) in FILE_OBJECTS_DIRECTORIES.iter().enumerate() {
            let _file_path = format!("{}/{}", TEST_DIR, _object).to_string();
            let res = create_data_file(TEST_DIR, _object, &_file);
            assert_eq!(res.unwrap(), true);
            assert_eq!(Path::new(&_file_path).exists(), true);
        }
    }
}
