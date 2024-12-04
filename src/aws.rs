use aws_config::BehaviorVersion;
use aws_sdk_s3::{config::Region, Client};

pub struct S3 {
    client: Client,
}

impl S3 {
    pub async fn init(
        access_key_id: String,
        secret_access_key: String,
    ) -> Result<Self, aws_sdk_s3::Error> {
        
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
                                                        

        let client = Client::new(&config);

        Ok(Self { client })
    }
}
