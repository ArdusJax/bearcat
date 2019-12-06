use crate::rusoto_s3::S3;
use rusoto_s3::{
    CompleteMultipartUploadRequest, CompletedMultipartUpload, CompletedPart,
    CreateMultipartUploadRequest, GetObjectRequest, UploadPartRequest, GetObjectOutput,
    GetObjectError, HeadBucketRequest, ListObjectsV2Request, ListObjectsV2Error,
    ListObjectsV2Output,
};
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use futures::prelude::*;
use std::path::Path;
use bytes::Bytes;

// Download contents to an S3 bucket
pub fn download<'a, 'b>(
    client: &rusoto_s3::S3Client,
    path: &'a str,
    bucket: &'b str,
) -> Result<(), ()> {
    // Always check if the permissions are correct
    if check_bucket_access(client, bucket) {
        let req = GetObjectRequest {
            bucket: String::from(bucket),
            key: String::from(path),
            ..Default::default()
        };
        let res = client
            .get_object(req)
            .sync()
            .expect("error getting the object");
        let stream = res.body.unwrap();
        let body = stream.concat2().wait().unwrap();
        match create_download_file(&body, path, "data") {
            Ok(()) => (),
            Err(e) => {
                panic!("unable to create files or directories");
            }
        }
    }
    Ok(())
}

// Upload using multipart method, the contents to an S3 bucket
// Todo: Add actual type safe results
pub fn upload<'a, 'b, 'c>(
    client: &rusoto_s3::S3Client,
    path: &'a str,
    filename: &'b str,
    bucket: &'c str,
) -> Result<(), std::io::Error> {
    // Always check if the permissions are correct
    if check_bucket_access(client, bucket) {
        // Create the request for a multiport upload to the S3 bucket
        let req = CreateMultipartUploadRequest {
            bucket: bucket.to_owned(),
            key: filename.to_owned(),
            ..Default::default()
        };

        // Make the request and log the result
        let res = client
            .create_multipart_upload(req)
            .sync()
            .expect("Could not create multipart upload.");
        println!("{:#?}", res);
        // Get the upload id from the resposne
        let upload_id = res.upload_id.unwrap();
        // Create all of the parts for uploading
        let parts = processes_object(filename, bucket, filename, &upload_id)?;
        let mut completed_parts = Vec::new();
        for part in parts {
            let part_num = part.part_number;
            let response = client
                .upload_part(part)
                .sync()
                .expect("Failed to upload part");
            // Collect the completed  parts for finalizing later
            completed_parts.push(CompletedPart {
                e_tag: response.e_tag.clone(),
                part_number: Some(part_num),
            });
        }
        // Create the completed multipart upload with the added e-tags
        let completed_upload = CompletedMultipartUpload {
            parts: Some(completed_parts),
        };

        let complete_req = CompleteMultipartUploadRequest {
            bucket: bucket.to_owned(),
            key: filename.to_owned(),
            upload_id: upload_id.to_owned(),
            multipart_upload: Some(completed_upload),
            ..Default::default()
        };

        let result = client.complete_multipart_upload(complete_req).sync();
        match result {
            Err(e) => println!("Could not complete the multipart upload.\n{:?}", e),
            Ok(result) => println!("Result: \n{:?}", result),
        }
    }

    Ok(())
}

// Processes the file and return a vec of UploadPartRequest
fn processes_object(
    path: &str,
    bucket: &str,
    filename: &str,
    upload_id: &str,
) -> Result<Vec<UploadPartRequest>, std::io::Error> {
    // Get the file ready for processing
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(5242880, file);
    // Get all of the parameters for the upload parts initialized
    let mut index = 1;
    let mut upload_requests = Vec::new();
    // Start parsing the file and creating the parts, collecting them in a vector
    while !reader.fill_buf()?.is_empty() {
        upload_requests.push(create_upload_part(
            bucket,
            filename,
            upload_id,
            reader.buffer().to_vec(),
            index,
        ));
        reader.consume(5242880);
        index += 1;
    }
    // If there are no errors return the vec of UploadPartRequest
    Ok(upload_requests)
}

fn create_upload_part(
    bucket: &str,
    filename: &str,
    upload_id: &str,
    body: Vec<u8>,
    part_number: i64,
) -> UploadPartRequest {
    UploadPartRequest {
        body: Some(body.into()),
        bucket: bucket.to_owned(),
        key: filename.to_owned(),
        upload_id: upload_id.to_owned(),
        part_number,
        ..Default::default()
    }
}
fn check_bucket_access(client: &rusoto_s3::S3Client, bucket_name: &str) -> bool {
    let req = HeadBucketRequest {
        bucket: bucket_name.to_owned(),
    };
    match client.head_bucket(req).sync() {
        Ok(_) => true,
        Err(e) => {
            println!("error accessing the bucket: {:?}\n{:?}", bucket_name, e);
            false
        }
    }
}

pub fn get_bucket_object_keys(
    client: &rusoto_s3::S3Client,
    bucket_name: &str,
) -> Result<Vec<String>, ()> {
    let req = ListObjectsV2Request {
        bucket: bucket_name.to_owned(),
        ..Default::default()
    };
    match client.list_objects_v2(req).sync() {
        Ok(result) => {
            let mut key_list: Vec<String> = Vec::new();
            match result.contents {
                Some(objects) => {
                    for object in objects {
                        // if the object is isn't None unwrap and add to the list
                        if let Some(e) = &object.key {
                            key_list.push(e.to_string());
                        }
                    }
                    println!("Key List:\n{:?}", &key_list);
                    Ok(key_list)
                }
                None => Err(()),
            }
        }
        Err(e) => {
            // Only want to log the error
            println!("Error: \n{:?}", e);
            Err(())
        }
    }
}
fn create_download_file(content: &bytes::Bytes, key: &str, base: &str) -> std::io::Result<()> {
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
    Ok(())
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
    fn create_upload_part_test() {
        let part = create_upload_part("bearcat-test", "resources/test.txt", "34", vec![1, 2, 3], 1);
        assert_eq!(part.bucket, "bearcat-test");
        assert_eq!(part.content_length, None);
        assert_eq!(part.part_number, 1);
        assert_eq!(part.upload_id, "34");
        assert_eq!(part.key, "resources/test.txt");
        assert_eq!(part.body.is_some(), true);
    }

    #[test]
    fn create_file_normal_names_test() {
        let _file = bytes::Bytes::from(FILE_CONTENT);
        for (_, _object) in FILE_OBJECTS.iter().enumerate() {
            let _file_path = format!("{}/{}", TEST_DIR, _object).to_string();
            let res = create_download_file(&_file, _object, TEST_DIR);
            assert_eq!(res.unwrap(), ());
            assert_eq!(Path::new(&_file_path).exists(), true);
        }
    }
    #[test]
    fn create_file_abnormal_names_test() {
        let _file = bytes::Bytes::from(FILE_CONTENT);

        for (_, _object) in FILE_OBJECTS_ABNORMAL.iter().enumerate() {
            let _file_path = format!("{}/{}", TEST_DIR, _object).to_string();
            let res = create_download_file(&_file, _object, TEST_DIR);
            assert_eq!(res.unwrap(), ());
            assert_eq!(Path::new(&_file_path).exists(), true);
        }
    }
    #[test]
    fn create_object_dir_test() {
        let _file = bytes::Bytes::from(FILE_CONTENT);

        for (_, _object) in FILE_OBJECTS_DIRECTORIES.iter().enumerate() {
            let _file_path = format!("{}/{}", TEST_DIR, _object).to_string();
            let res = create_download_file(&_file, _object, TEST_DIR);
            assert_eq!(res.unwrap(), ());
            assert_eq!(Path::new(&_file_path).exists(), true);
        }
    }
}
