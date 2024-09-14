use std::env;

use s3::{creds::Credentials, Bucket};

pub fn get_bucket() -> Bucket {
    let bucket_name = env::var("AWS_S3_BUCKET").expect("AWS_S3_BUCKET must be set");
    let access_key = env::var("AWS_ACCESS_KEY").expect("AWS_ACCESS_KEY must be set");
    let secret_key = env::var("AWS_SECRET_KEY").expect("AWS_SECRET_KEY must be set");
    let region = env::var("AWS_REGION").expect("AWS_REGION must be set");

    let credentials =
        Credentials::new(Some(&access_key), Some(&secret_key), None, None, None).unwrap();

    *Bucket::new(&bucket_name, region.parse().unwrap(), credentials).unwrap()
}
