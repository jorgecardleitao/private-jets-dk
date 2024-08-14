"""
Script that runs an SQL statement and writes 
"""
import sys
import os

import duckdb
import boto3.session


with open(sys.argv[1]) as f:
    sql = f.read()

os.makedirs("results", exist_ok=True)
files = list(map(lambda x: x[0], duckdb.sql(sql).fetchall()))

session = boto3.session.Session()
s3_client = session.client(
    service_name="s3",
    aws_access_key_id=os.environ["AWS_ACCESS_KEY_ID"],
    aws_secret_access_key=os.environ["AWS_SECRET_ACCESS_KEY"],
    region_name="fra1",
    endpoint_url="https://fra1.digitaloceanspaces.com",
)

for file in files:
    s3_client.upload_file(f"results/{file}", "private-jets", f"analysis/v1/{file}", ExtraArgs={'ACL':'public-read'})
