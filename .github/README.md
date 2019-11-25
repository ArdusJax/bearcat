# Bearcat Bucket Sync

![](https://github.com/ArdusJax/bearcat/workflows/Production/badge.svg) ![](https://github.com/ArdusJax/bearcat/workflows/Development/badge.svg) ![](https://github.com/ArdusJax/bearcat/workflows/Cargo%20Security%20Audit/badge.svg)

## Overview

Application that uses locked down IAM users to securely sync AWS bucket content between AWS buckets. This is extremely valuable for syncing content between the commercial and restricted accounts in AWS.

### Restricted Account Examples

- `govcloud`
- `china`

## Architecture

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

#### Diagram

### EC2 Instance + Large EBS Volume

Used when you need to sync large files or AMI's. The attached EBS volume needs to have sufficient space to store the artifacts while they are being sync'd.

#### Benefits

- Can sync large files
- Control the amount of space that you need for the EBS volume
- Not limited 15 minutes of activity

#### Diagram
