# Bearcat Bucket Sync

![](https://github.com/ArdusJax/bearcat/workflows/Production%20Build/badge.svg) ![](https://github.com/ArdusJax/bearcat/workflows/Development%20Build/badge.svg) ![](https://github.com/ArdusJax/bearcat/workflows/Cargo%20Security%20Audit/badge.svg) ![](https://github.com/ArdusJax/bearcat/workflows/Testing/badge.svg) 

## Overview

Application that should use locked down IAM users to securely sync AWS bucket content between AWS buckets. This is extremely valuable for syncing content between the commercial and restricted accounts in AWS.

### Restricted Account Examples

- `govcloud`
- `china`

## Usage

```text
USAGE:
    bearcat [OPTIONS] <source> <destination> <region_source> <region_destination> [--] [ARGS]

ARGS:
    <source>                Name of the AWS bucket to be sync'd from.
    <destination>           Name of the AWS bucket to be sync'd to.
    <region_source>         AWS region for the bucket that is being sync'd from.
    <region_destination>    AWS region for the bucket that is being sync'd to.
    <ssm_key>               SSM key that has credentials for syncing across accounts
    <profile>               Commercial AWS profile
    <config>                location of the configuration file (defaults to ~/.bearcat)

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -v <verbose>...        Sets the level of verbosity
```

## Installation

### Script

The installation uses a cloud formation template to create an ec2 instance with the `bearcat` configured to sync between the buckets provided.

* Clone the repository
* Run the `setup.sh` located in the `resources` directory

### Manual EC2

If the cloud formation standup doesn't work for you in some way, you can download the binary on an ec2 instance and configure it as a service. This option sets up the binary as a linux service that is invoked on a schedule via a cron job.

* Create an ec2 instance with S3 access to both accounts
* Clone the `bearcat` project and build the binary
* Copy the binary to the `/usr/bin` directory on your ec2 instance
* Copy the service `bearcat.service` file from the resources directory to the `"/etc/systemd/system/bearcat.service"` directory on your ec2 instance
* Replace the values for the `source_bucket`, `destination_bucket`, `source_aws_region`, `destination_aws_region`
* Change the mode to `700`
* Change the owner to a restricted user on the ec2 instance
* After the binary and the service file are in place enable the service using `systemctl enable bearcat`

## Architecture

Bearcat downloads all of the objects in your source bucket, and then uploads them to the destination bucket.

This application can be used in a few different architectures, depending on the use case.

| Use case | Architecture|
|----------|-------------|
|Syncing small files (<500MB)|Lambda
|Syncing large files (>500MB)|EC2 + Large EBS volume

### Lambda

For smaller files that need to be sync'd between two buckets we can use an AWS Lambda to run the application.

#### Benefits

- Don't have to provision an EC2 instance
- Only running when something needs to be sync'd
- Limited attack surface
- Only pay for the runtime of the Lambda + transfer cost

### EC2 Instance + Large EBS Volume

Used when you need to sync large files or AMI's. The attached EBS volume needs to have sufficient space to store the artifacts while they are being sync'd.

#### Benefits

- Can sync large files
- Control the amount of space that you need for the EBS volume
- Not limited 15 minutes of activity
