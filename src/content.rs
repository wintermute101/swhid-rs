use crate::core::{ObjectType, Swhid};
use crate::hash::hash_blob;

/// In‑memory file content (minimal helper).
#[derive(Debug, Clone)]
pub struct Content<'a> {
    bytes: Cow<'a, [u8]>,
}

use std::borrow::Cow;

impl<'a> Content<'a> {
    pub fn from_bytes(bytes: impl Into<Cow<'a, [u8]>>) -> Self {
        Self { bytes: bytes.into() }
    }

    #[cfg(feature="serde")]
    pub fn as_bytes(&self) -> &[u8] { &self.bytes }

    pub fn swhid(&self) -> Swhid {
        let digest = hash_blob(&self.bytes);
        Swhid::new(ObjectType::Content, digest)
    }
}
