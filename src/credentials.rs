use rusoto_core::Region;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};

enum Provider {
    AWS_STS,
}

fn new_provider(provider_type: Provider) {
    let sts = StsClient::new(Region::UsWest2);
    let provider = StsAssumeRoleSessionCredentialsProvider::new(
        sts,
        "arn:aws:iam::something:role/something".to_owned(),
        "default".to_owned(),
        None,
        None,
        None,
        None,
    );

    let auto_refreshing_provider = rusoto_credential::AutoRefreshingProvider::new(provider);
}

fn new_chain() {}
