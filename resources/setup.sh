#!/bin/bash

# This is a quick and dirty script to stand up the stack for the sync tool.
# Better installation methods will come in a later version
DESTINATION_BUCKET=$1
SOURCE_BUCKET=$2
KEY_NAME=$3

# Check if the aws cli is installed
installed="which aws"
[$installed -eq 0] && echo "aws cli is installed..." || echo "aws cli could not be found or is not installed. Please install the cli and try this script again."

# Execute the cloud formation with the given parameters
aws cloudformation create-stack --stack-name bearcat-sync --template-body file:///resources/bearcat.yaml --parameters ParameterKey=SourceBucketName,ParameterValue=$SOURCE_BUCKET,ParameterKey=DestinationBucketName,ParameterValue=$DESTINATION_BUCKET,ParameterKey=KeyName,ParameterValue=$KEY_NAME
