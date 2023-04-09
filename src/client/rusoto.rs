#[derive(Clone)]
pub struct Client {
    pub s3: rusoto_s3::S3Client
}

impl Client{
    pub fn new() -> Self {
     return Self { 
            s3: rusoto_s3::S3Client::new_with(
            rusoto_core::request::HttpClient::new().unwrap(),
            rusoto_credential::StaticProvider::from(
                rusoto_credential::AwsCredentials::new(
                std::env::var("BUCKET_KEY").expect("BUCKET_KEY must be defined"), 
                std::env::var("BUCKET_SECRET").expect("BUCKET_SECRET must be defined"), 
                None, 
                None)
            ),
            rusoto_core::Region::Custom {
                name: "sgp1".to_owned(),
                endpoint: "https://sgp1.digitaloceanspaces.com".to_owned(),
            },
            )
        }
    }
}