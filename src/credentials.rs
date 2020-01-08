use rusoto_core::Region;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};

enum Provider {
    AWS_STS,
}

// Assume the role used for syncing a bucket on the commercial side
fn new_provider(provider_type: Provider, role_arn: &str, profile: &str, region: Region) {
    let sts = StsClient::new(region);
    let provider = StsAssumeRoleSessionCredentialsProvider::new(
        sts,
        role_arn.to_owned(),
        profile.to_owned(),
        None,
        None,
        None,
        None,
    );

    let auto_refreshing_provider = rusoto_credential::AutoRefreshingProvider::new(provider);
}

fn new_chain() {}
