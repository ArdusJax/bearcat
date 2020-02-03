use crate::rusoto_s3::S3;
use crate::data::{create_data_file, delete_data_file};
use std::error::Error;
use rusoto_s3::{
    CompleteMultipartUploadRequest, CompletedMultipartUpload, CompletedPart,
    CreateMultipartUploadRequest, DeletedObject, DeleteObjectRequest, DeleteObjectOutput,
    GetObjectRequest, UploadPartRequest, GetObjectOutput, GetObjectError, HeadBucketRequest,
    ListObjectsV2Request, ListObjectsV2Error, ListObjectsV2Output,
};
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use futures::prelude::*;
use std::path::Path;
use bytes::Bytes;
use log::{info, warn};

static BASE_PATH: &str = "data";

// Download contents to an S3 bucket
pub fn download<'a, 'b>(
    client: &rusoto_s3::S3Client,
    path: &'a str,
    bucket: &'b str,
) -> Result<bool, Box<dyn Error>> {
    check_bucket_access(client, bucket)?;
    let req = GetObjectRequest {
        bucket: String::from(bucket),
        key: String::from(path),
        ..Default::default()
    };
    info!(target: "BUCKET DOWNLOAD", "Checking accesss to bucket: {:?}",&bucket);
    let res = client
        .get_object(req)
        .sync()
        .map_err(|e| format! {"Error getting object from source bucket {:?}", e})?;
    let stream = res.body.unwrap();
    let body = stream.concat2().wait().unwrap();
    create_data_file(BASE_PATH, path, &body)?;
    delete_bucket_object(client, bucket, path)?;
    info!(target: "BUCKET DOWNLOAD", "Download completed successfully from {:?}",&bucket);
    Ok(true)
}

// Upload using multipart method, the contents to an S3 bucket
pub fn upload<'a, 'b, 'c>(
    client: &rusoto_s3::S3Client,
    path: &'a str,
    filename: &'b str,
    bucket: &'c str,
) -> Result<bool, Box<dyn Error>> {
    check_bucket_access(client, bucket)?;
    let req = CreateMultipartUploadRequest {
        bucket: bucket.to_owned(),
        key: filename.to_owned(),
        ..Default::default()
    };

    let res = client
        .create_multipart_upload(req)
        .sync()
        .map_err(|e| format! {"Failed to create multipart"})?;
    let upload_id = res.upload_id.unwrap();

    // Create all of the parts for uploading
    info!(target: "UPLOAD", "Creating parts for multipart upload...");
    let parts = processes_object(path, bucket, filename, &upload_id)?;
    let mut completed_parts = Vec::new();
    for part in parts {
        let part_num = part.part_number;
        let response = client
            .upload_part(part)
            .sync()
            .map_err(|e| format! {"Failed to upload part"})?;
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

    info!(target: "UPLOAD", "Sending multipart upload completion request...");
    client
        .complete_multipart_upload(complete_req)
        .sync()
        .map_err(|e| format! {"Failed to complete the multipart upload"})?;

    info!(target: "UPLOAD", "Upload to {:?} completed successfully", &bucket);
    delete_data_file(BASE_PATH, &filename)
}

// Processes the file and return a vec of UploadPartRequest
fn processes_object(
    path: &str,
    bucket: &str,
    filename: &str,
    upload_id: &str,
) -> Result<Vec<UploadPartRequest>, std::io::Error> {
    // Get the file ready for processing
    let file = File::open(format!("{}/{}", path, filename))?;
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

fn delete_bucket_object(
    client: &rusoto_s3::S3Client,
    bucket: &str,
    key: &str,
) -> Result<DeleteObjectOutput, Box<dyn Error>> {
    let req = DeleteObjectRequest {
        bucket: bucket.to_owned(),
        key: key.to_owned(),
        ..Default::default()
    };
    let resp = client
        .delete_object(req)
        .sync()
        .map_err(|e| format! {"Error deleting the bucket object"})?;
    info!(target: "DELETE OBJECT", "Deleted object {:?} successfully", bucket);
    Ok(resp)
}

fn check_bucket_access(
    client: &rusoto_s3::S3Client,
    bucket_name: &str,
) -> Result<bool, Box<dyn Error>> {
    let req = HeadBucketRequest {
        bucket: bucket_name.to_owned(),
    };
    client
        .head_bucket(req)
        .sync()
        .map_err(|e| format! {"error accessing the bucket {:?}", e})?;
    Ok(true)
}

pub fn get_bucket_object_keys(
    client: &rusoto_s3::S3Client,
    bucket_name: &str,
) -> Result<Vec<String>, ()> {
    let req = ListObjectsV2Request {
        bucket: bucket_name.to_owned(),
        ..Default::default()
    };
    info!(target: "BUCKET GET OBJECT", "getting objects from bucket: {:?}", &bucket_name);
    match client.list_objects_v2(req).sync() {
        Ok(result) => {
            let mut key_list: Vec<String> = Vec::new();
            match result.contents {
                Some(objects) => {
                    for object in objects {
                        // if the object is isn't None unwrap and add to the list
                        if let Some(e) = &object.key {
                            if !e.ends_with('/') {
                                key_list.push(e.to_string());
                            }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
