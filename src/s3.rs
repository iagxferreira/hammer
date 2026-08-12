use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_sdk_s3::{primitives::ByteStream, Client};
use chrono::Utc;
use std::path::Path;

pub async fn upload_file(path: impl AsRef<Path>, bucket: &str) -> Result<String> {
    let path = path.as_ref();
    let region =
        aws_config::meta::region::RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region)
        .load()
        .await;
    let client = Client::new(&config);

    let key = format!("{}.zip", Utc::now().format("%Y%m%dT%H%M%SZ"));
    let body = ByteStream::from_path(path)
        .await
        .with_context(|| format!("failed to read {}", path.display()))?;

    let response = client
        .put_object()
        .bucket(bucket)
        .key(&key)
        .body(body)
        .send()
        .await
        .with_context(|| format!("failed to upload {} to s3://{bucket}/{key}", path.display()))?;

    Ok(response
        .version_id()
        .map(|version| format!("s3://{bucket}/{key} (version {version})"))
        .unwrap_or_else(|| format!("s3://{bucket}/{key}")))
}
