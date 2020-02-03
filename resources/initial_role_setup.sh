#!/bin/bash

# This is a quick and dirty script to stand up the initail role that is used to stand up
# the stack
# Better installation methods will come in a later versions

# Check if the aws cli is installed
installed="which aws"
[$installed -eq 0] && echo "aws cli is installed..." || echo "aws cli could not be found or is not installed. Please install the cli and try this script again."

# Execute the cloud formation with the given parameters
aws cloudformation create-stack --stack-name bearcat-sync-role-creation --template-body file:///bearcat_bucket_sync.yml
