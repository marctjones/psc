use worker::*;

/// R2 storage operations for media blobs

/// Get the R2 bucket binding
pub fn get_bucket(env: &Env) -> Result<Bucket> {
    env.bucket("BUCKET")
}

/// Store a blob
pub async fn put_blob(
    bucket: &Bucket,
    key: &str,
    data: Vec<u8>,
    _content_type: &str,
) -> Result<()> {
    // Note: content_type setting requires HttpMetadata in worker 0.4+
    bucket
        .put(key, data)
        .execute()
        .await?;

    Ok(())
}

/// Get a blob
pub async fn get_blob(bucket: &Bucket, key: &str) -> Result<Option<Vec<u8>>> {
    let object = bucket.get(key).execute().await?;

    match object {
        Some(obj) => {
            let body = obj.body()
                .ok_or_else(|| Error::RustError("No body".to_string()))?;
            let bytes = body.bytes().await?;
            Ok(Some(bytes))
        }
        None => Ok(None),
    }
}

/// Delete a blob
pub async fn delete_blob(bucket: &Bucket, key: &str) -> Result<()> {
    bucket.delete(key).await?;
    Ok(())
}

/// Check if a blob exists
pub async fn blob_exists(bucket: &Bucket, key: &str) -> Result<bool> {
    let object = bucket.head(key).await?;
    Ok(object.is_some())
}

/// List blobs with prefix
pub async fn list_blobs(
    bucket: &Bucket,
    prefix: Option<&str>,
    limit: u32,
) -> Result<Vec<String>> {
    let mut list_opts = bucket.list();

    if let Some(p) = prefix {
        list_opts = list_opts.prefix(p);
    }

    let result = list_opts.limit(limit).execute().await?;

    let keys: Vec<String> = result
        .objects()
        .iter()
        .map(|obj| obj.key().to_string())
        .collect();

    Ok(keys)
}

/// Calculate CID for content (simplified)
/// In production, use proper IPFS CID calculation
pub fn calculate_cid(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as BASE64};

    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    // Simplified CID format (not actual CIDv1)
    format!("bafyreig{}", BASE64.encode(&hash[..20]).to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_cid() {
        let data = b"Hello, World!";
        let cid = calculate_cid(data);

        assert!(cid.starts_with("bafyreig"));
        assert!(!cid.is_empty());

        // Same input should give same CID
        let cid2 = calculate_cid(data);
        assert_eq!(cid, cid2);
    }
}
