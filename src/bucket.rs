use crate::rusoto_s3::S3;
use rusoto_s3::{
    CompleteMultipartUploadRequest, CompletedMultipartUpload, CompletedPart,
    CreateMultipartUploadRequest, GetObjectRequest, UploadPartRequest, GetObjectOutput,
    GetObjectError,
};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use futures::prelude::*;

// Download contents to an S3 bucket
pub fn download<'a, 'b>(
    client: &rusoto_s3::S3Client,
    path: &'a str,
    bucket: &'b str,
) -> Result<(), ()> {
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
    let mut file = File::create(&path).expect("failed to create file for download");
    file.write_all(&body)
        .expect("failed to write body to the file");
    Ok(())
}

// Download contents to an S3 bucket
// Todo: Add actual type safe results
pub fn download_all<'a, 'b>(
    client: &rusoto_s3::S3Client,
    path: &'a str,
    bucket: &'b str,
) -> Result<(), ()> {
    let req = GetObjectRequest {
        bucket: String::from(bucket),
        ..Default::default()
    };
    let res = client.get_object(req);
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

    Ok(())
}

// Processses the file and return a vec of UploadPartRequest
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
        assert_eq!(part.key, "test.txt");
        assert_eq!(part.body.is_some(), true);
    }
}
