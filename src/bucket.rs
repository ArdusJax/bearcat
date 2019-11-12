use rusoto_s3::{GetObjectRequest};
use crate::rusoto_s3::S3;
// Download contents to an S3 bucket
// Todo: Add actual type safe results
pub fn download(client: &rusoto_s3::S3Client, path: String, bucket: &String) -> Result<(),()> {
  let req = GetObjectRequest{ bucket: bucket.clone(), ..Default::default()};
  let res = client.get_object( req);
    Ok(())
}

// Upload contents to an S3 bucket
// Todo: Add actual type safe results
pub fn upload(client: &rusoto_s3::S3Client, path: String, bucket: &String) -> Result<(),()> {
    Ok(())
}
