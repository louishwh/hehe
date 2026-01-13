use bytes::Bytes;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::Result;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResourceRef {
    #[serde(skip)]
    Inline(Bytes),
    Base64 { data: String },
    File { path: Utf8PathBuf },
    Url { url: Url },
    ContentAddress { hash: String },
}

impl ResourceRef {
    pub fn inline(data: impl Into<Bytes>) -> Self {
        Self::Inline(data.into())
    }

    pub fn base64(data: impl Into<String>) -> Self {
        Self::Base64 { data: data.into() }
    }

    pub fn file(path: impl Into<Utf8PathBuf>) -> Self {
        Self::File { path: path.into() }
    }

    pub fn url(url: Url) -> Self {
        Self::Url { url }
    }

    pub fn content_address(hash: impl Into<String>) -> Self {
        Self::ContentAddress { hash: hash.into() }
    }

    pub fn is_inline(&self) -> bool {
        matches!(self, Self::Inline(_) | Self::Base64 { .. })
    }

    pub fn is_remote(&self) -> bool {
        matches!(self, Self::Url { .. })
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Self::File { .. })
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResourceMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

impl ResourceMeta {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_media_type(mut self, media_type: impl Into<String>) -> Self {
        self.media_type = Some(media_type.into());
        self
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn with_checksum(mut self, checksum: impl Into<String>) -> Self {
        self.checksum = Some(checksum.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct Resource {
    pub reference: ResourceRef,
    pub meta: ResourceMeta,
}

impl Resource {
    pub fn new(reference: ResourceRef) -> Self {
        Self {
            reference,
            meta: ResourceMeta::default(),
        }
    }

    pub fn with_meta(mut self, meta: ResourceMeta) -> Self {
        self.meta = meta;
        self
    }

    pub fn inline(data: impl Into<Bytes>) -> Self {
        Self::new(ResourceRef::inline(data))
    }

    pub fn from_base64(data: impl Into<String>) -> Self {
        Self::new(ResourceRef::base64(data))
    }

    pub fn from_file(path: impl Into<Utf8PathBuf>) -> Self {
        Self::new(ResourceRef::file(path))
    }

    pub fn from_url(url: Url) -> Self {
        Self::new(ResourceRef::url(url))
    }
}

#[async_trait::async_trait]
pub trait ResourceResolver: Send + Sync {
    async fn resolve(&self, resource: &ResourceRef) -> Result<Bytes>;
    async fn resolve_base64(&self, resource: &ResourceRef) -> Result<String>;
    async fn metadata(&self, resource: &ResourceRef) -> Result<ResourceMeta>;
}

#[async_trait::async_trait]
pub trait ResourceStore: Send + Sync {
    async fn store(&self, data: Bytes, meta: ResourceMeta) -> Result<String>;
    async fn get(&self, content_address: &str) -> Result<Option<Bytes>>;
    async fn exists(&self, content_address: &str) -> Result<bool>;
    async fn delete(&self, content_address: &str) -> Result<bool>;
}
