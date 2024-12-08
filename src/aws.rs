// use aws_config::BehaviorVersion;
// use aws_sdk_s3::{
//     config::{Credentials, Region, SharedCredentialsProvider},
//     presigning::{PresignedRequest, PresigningConfig},
//     Client,
// };
// use std::time::Duration;

// pub struct S3 {
//     pub client: Client,
//     pub bucket: String,
// }

// impl S3 {
//     pub async fn init(
//         access_key_id: String,
//         secret_access_key: String,
//         region: String,
//         bucket: String,
//         provider_name: &'static str,
//     ) -> Result<Self, aws_sdk_s3::Error> {
//         let cred = Credentials::new(access_key_id, secret_access_key, None, None, provider_name);

//         let config = aws_config::SdkConfig::builder()
//             .credentials_provider(SharedCredentialsProvider::new(cred))
//             .region(Region::new(region))
//             .behavior_version(BehaviorVersion::latest())
//             .build();

//         let client = Client::new(&config);

//         Ok(Self { client, bucket })
//     }

//     pub async fn presigned_url(&self, key: String) -> Result<PresignedRequest, Box< dyn std::error::Error>> {
//         let pre = PresigningConfig::builder()
//             .expires_in(Duration::new(3600, 0))
//             .build()?;

//         let url = self.client
//             .put_object()
//             .bucket(&self.bucket)
//             .key(key)
//             .presigned(pre).await?;

//         Ok(url)
//     }

// }
