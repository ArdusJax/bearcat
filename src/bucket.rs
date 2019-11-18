use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use rusoto_s3::{GetObjectRequest, CreateMultipartUploadRequest, UploadPartRequest};
use crate::rusoto_s3::S3;

// Download contents to an S3 bucket
// Todo: Add actual type safe results
pub fn download<'a,'b>(client: &rusoto_s3::S3Client, path: &'a str, bucket: &'b str) -> Result<(),()> {
  let req = GetObjectRequest{ bucket: String::from(bucket), ..Default::default()};
  let res = client.get_object(req);
  
    Ok(())
}

// Upload using multipart method, the contents to an S3 bucket
// Todo: Add actual type safe results
pub fn upload<'a,'b,'c>(client: &rusoto_s3::S3Client, path: &'a str, filename: &'b str, bucket: &'c str) -> Result<(),()> {
  // Create the request for a multiport upload to the S3 bucket
  let req = CreateMultipartUploadRequest {
    bucket: bucket.to_owned(),
    key: filename.to_owned(),
    ..Default::default()
  };

  // Make the request and log the result
  let res = client.create_multipart_upload(req)
  .sync()
  .expect("Could not create multipart upload.");
  println!("{:#?}", res);
  // Get the upload id from the resposne
  let upload_id = res.upload_id.unwrap();
  // Create all of the parts for uploading
  processes_object(filename, bucket, filename, &upload_id);

  Ok(())
}

// Processses the file and return a vec of UploadPartRequest
pub fn processes_object(path: &str, bucket: &str, filename: &str, upload_id: &str) -> Result<Vec<UploadPartRequest>, std::io::Error>  {
  // Get the file ready for processing
  let file = File::open(path)?;
  let mut reader = BufReader::with_capacity(10, file);
  // Get all of the parameters for the upload parts initialized
  let index = 0;
  let mut upload_requests: Vec<UploadPartRequest>;
  // Anonymous function for creating the UploadPartRequest
  let create_upload_part = |body: Vec<u8>, part_number: i64| -> UploadPartRequest {
        UploadPartRequest {
            body: Some(body.into()),
            bucket: bucket.to_owned(),
            key: filename.to_owned(),
            upload_id: upload_id.to_owned(),
            part_number,
            ..Default::default()
        }
    };
  // Start parsing the file and creating the parts, collecting them in a vector
  while !reader.fill_buf()?.is_empty() {
    println!("{:?}", reader.buffer());
    upload_requests.push(create_upload_part(reader.buffer().to_vec(), index));
    reader.consume(10);
    index = index + 1;
  }
  // If there are no errors return the vec of UploadPartRequest
  Ok(upload_requests)
}