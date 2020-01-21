use rusoto_core::{Region, RusotoError};
use rusoto_credential::AwsCredentials;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use rusoto_ssm::*;
use std::io::{Error, ErrorKind};

pub enum CredentialProvider {
    AwsSsm { key: String, region: Region },
}

impl CredentialProvider {
    pub fn get_credentials(self) -> Result<AwsCredentials, Error> {
        match self {
            Self::AwsSsm { key, region } => get_creds_from_ssm(key, region),
        }
    }
}

// Get the credentials from ssm
fn get_creds_from_ssm(key: String, region: Region) -> Result<AwsCredentials, Error> {
    // need real logging not println
    // need to include error chaining
    let client = rusoto_ssm::SsmClient::new(region);
    let req = GetParameterRequest {
        name: key,
        with_decryption: Some(true), // forcing always on for now
    };
    let res = client
        .get_parameter(req)
        .sync()
        .expect("failed to get secret from parameter store");

    let v = res.parameter.unwrap().value.unwrap();
    let cred = parse_cred_from_ssm_value(&v);
    Ok(AwsCredentials::new("something", "secret", None, None))
}

fn parse_cred_from_ssm_value(value: &str) -> Result<AwsCredentials, ()> {
    Ok(AwsCredentials::new("something", "secret", None, None))
}
