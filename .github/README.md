# Bearcat Bucket Sync

![](https://github.com/ArdusJax/bearcat/workflows/Production%20Build/badge.svg) ![](https://github.com/ArdusJax/bearcat/workflows/Development%20Build/badge.svg) ![](https://github.com/ArdusJax/bearcat/workflows/Cargo%20Security%20Audit/badge.svg) ![](https://github.com/ArdusJax/bearcat/workflows/Testing/badge.svg) 

## Overview

Application that uses locked down IAM users to securely sync AWS bucket content between AWS buckets. This is extremely valuable for syncing content between the commercial and restricted accounts in AWS.

### Restricted Account Examples

- `govcloud`
- `china`

## Usage

```text
USAGE:
    bearcat [OPTIONS] <source> <destination> [--] [ARGS]

ARGS:
    <source>         Name of the AWS bucket to be sync'd from.
    <destination>    Name of the AWS bucket to be sync'd to.
    <region-from>    AWS region for the bucket that is being sync'd from.
    <region-to>      AWS region for the bucket that is being sync'd to.
    <profile>        Commercial AWS profile
    <config>         location of the configuration file (defaults to ~/.bearcat)

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -v <verbose>...        Sets the level of verbosity
```

## Installation

### Script

* Clone the repository
* Run the `setup.sh` located in the `resources` directory

The installation uses a cloud formation template to create an ec2 instance with the `bearcat` configured to sync between the buckets provided.

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
