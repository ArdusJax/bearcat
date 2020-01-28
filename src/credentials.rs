use regex::{Regex, Captures};
use rusoto_core::{HttpClient, Region, RusotoError};
use rusoto_credential::{AwsCredentials, ProvideAwsCredentials};
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use rusoto_ssm::*;
use std::collections::HashMap;
use std::io;
use std::io::{ErrorKind};
use std::error::Error;
use rusoto_credential::ChainProvider;
use log::{info, warn};

pub enum CredentialProvider {
    AwsSsm { key: String, region: Region },
    AwsSts,
    Vault,
}

impl CredentialProvider {
    pub fn get_credentials(self) -> Result<AwsCredentials, Box<dyn Error>> {
        match self {
            Self::AwsSsm { key, region } => get_creds_from_ssm(key, region),
            Self::AwsSts => unimplemented!("Need to add sts integration"),
            Self::Vault => unimplemented!("Need to add Vault integration"),
        }
    }
}

fn get_creds_from_ssm(key: String, region: Region) -> Result<AwsCredentials, Box<dyn Error>> {
    let client =
        rusoto_ssm::SsmClient::new_with(HttpClient::new().unwrap(), ChainProvider::new(), region);
    let req = GetParameterRequest {
        name: key,
        with_decryption: Some(true), // forcing always on for now
    };
    info!(target: "Credential Provider Events", "Request:\n{:?}", &req);
    let res = client.get_parameter(req).sync()?;
    // need to get rid of these unwraps
    let v = res.parameter.unwrap().value.unwrap();
    let cap = parse_ssm_value(&v)?;
    parse_cred_from_ssm_value(cap)
}

// This assumes the key is stored in the format "aws_id::aws_key"
// Regex pattern (?x)(?P<id>.+)::(?P<secret>.+)
fn parse_cred_from_ssm_value(cap: Captures) -> Result<AwsCredentials, Box<dyn Error>> {
    Ok(AwsCredentials::new(
        cap["id"].to_owned(),
        cap["secret"].to_owned(),
        None,
        None,
    ))
}

// This assumes the key is stored in the format "aws_id::aws_key"
// Regex pattern (?x)(?P<id>.+)::(?P<secret>.+)
// todo: Add arg for the regex expression to be passed in
fn parse_ssm_value(value: &str) -> Result<Captures, Box<dyn Error>> {
    let keys = Regex::new(r"(?x)(?P<id>.{20})::(?P<secret>.{40})")?;
    match keys.captures(value) {
        Some(v) => Ok(v),
        None => Err(Box::new(io::Error::new(
            ErrorKind::NotFound,
            "no ssm credentials found in the ssm text",
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // All of these are static values for tests
    // This will be revisited, just want something here to test basic cases for now.

    // Happy path tests
    const SSM_VALID_TEST_VALUES: [&'static str; 3] = [
        "RKAEIZTBPEI2H7N5VH3B::9VB+PisnRTbW/A+NmVTRf2T3oJ//LqPEQnaXsUsC",
        "AKJDIZTWIEK7W8N5VH3B::AVHVRrsnRTbW/7qWKbAAf2T3DJR4LqPEQnaXs+sZ",
        "BZOAIZTBPEI2H7N5VH3B::NVH2ogsnRTbW/3q+MXTRf2T3oJ94LqR/+RaXsQsC",
    ];

    // Invalid vaules that should produce errors when fed to the parser
    const SSM_INVALID_TEST_VALUES: [&'static str; 7] = [
        "RKAEIZTBPEI2H7N5VH3B9VB+PisnRTbW/A+NmVTRf2T3oJ//LqPEQnaXsUsC",
        "AKJDIZTWIEK7W8N5VH3B//AVHVRrsnRTbW/7qWKbAAf2T3DJR4LqPEQnaXs+sZ",
        "BZOAIZTBPRTbW/3q+MXTRf2T3oJ94LqR/+RaXsQsC",
        "EI2H7N5VH3B: H7N5VH3B9VB+",
        "nRTbW/7qWKbhh//::BZOAIZTBPRTbW/:::qR/+RaXsQsC/7qWKbAAfTAVH3B::",
        "::BZOAIZTBPEI2H7N5VH3BNVH2ogsnRTbW/3q+MXTRf2T3oJ94LqR/+RaXsQsC",
        "BZOAIZTBPEI2H7N5VH3BNVH2ogsnRTbW/3q+MXTRf2T3oJ94LqR/+RaXsQsC::",
    ];

    #[test]
    fn parse_valid_ssm_value_test() {
        for (_, v) in SSM_VALID_TEST_VALUES.iter().enumerate() {
            let t = parse_ssm_value(v);
            assert_eq!(t.is_err(), false);
            if let Ok(cap) = t {
                assert_eq!(cap["id"].len(), 20);
                assert_eq!(cap["secret"].len(), 40)
            }
        }
    }

    #[test]
    fn parse_invalid_ssm_value_test() {
        for (_, v) in SSM_INVALID_TEST_VALUES.iter().enumerate() {
            assert_eq!(parse_ssm_value(v).is_err(), true)
        }
    }
}
