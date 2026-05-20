use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream,
    Client,
};

use crate::config::StorageConfig;

pub fn build_client(cfg: &StorageConfig) -> Client {
    let creds = Credentials::new(
        cfg.access_key.clone(),
        cfg.secret_key.clone(),
        None,
        None,
        "qr-api",
    );

    let s3_config = aws_sdk_s3::Config::builder()
        .behavior_version_latest()
        .region(Region::new(cfg.region.clone()))
        .endpoint_url(&cfg.endpoint_url)
        .credentials_provider(creds)
        .force_path_style(true)
        .build();

    Client::from_conf(s3_config)
}

pub async fn upload_bytes(
    client: &Client,
    bucket: &str,
    key: &str,
    bytes: Vec<u8>,
    content_type: &str,
) -> Result<(), anyhow::Error> {
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(bytes))
        .content_type(content_type)
        .send()
        .await?;
    Ok(())
}

pub async fn upload_svg(
    client: &Client,
    bucket: &str,
    key: &str,
    svg: String,
) -> Result<(), anyhow::Error> {
    upload_bytes(client, bucket, key, svg.into_bytes(), "image/svg+xml").await
}

pub fn public_url(public_base_url: &str, key: &str) -> String {
    format!("{}/{}", public_base_url.trim_end_matches('/'), key)
}

pub fn object_key(qr_id: uuid::Uuid, format: &str) -> String {
    format!("{}.{}", qr_id, format)
}
