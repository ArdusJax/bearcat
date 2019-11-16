use rusoto_s3::{GetObjectRequest, CreateMultipartUploadRequest, UploadPartRequest};
use crate::rusoto_s3::S3;
// Download contents to an S3 bucket
// Todo: Add actual type safe results
pub fn download(client: &rusoto_s3::S3Client, path: String, bucket: &String) -> Result<(),()> {
  let req = GetObjectRequest{ bucket: bucket.clone(), ..Default::default()};
  let res = client.get_object(req);
  
    Ok(())
}

// Upload using multipart method, the contents to an S3 bucket
// Todo: Add actual type safe results
pub fn upload(client: &rusoto_s3::S3Client, path: String, filename: &String, bucket: &String) -> Result<(),()> {
  let req = CreateMultipartUploadRequest {
    bucket: bucket.clone(),
    key: filename.clone(),
    ..Default::default()
  };
  let res = client.create_multipart_upload(req)
  .sync()
  .expect("Could not create multipart upload.");
  println!("{:#?}", res);
  let upload_id = res.upload_id.unwrap();

  let create_upload_part = |body: Vec<u8>, part_number: i64| -> UploadPartRequest {
        UploadPartRequest {
            body: Some(body.into()),
            bucket: bucket.to_owned(),
            key: filename.to_owned(),
            upload_id: upload_id.to_owned(),
            part_number: part_number,
            ..Default::default()
        }
    };

  Ok(())
}
