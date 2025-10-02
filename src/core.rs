use std::fmt::{self, Display};
use std::str::FromStr;

use crate::error::SwhidError;

/// Known SWH object kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ObjectType {
    /// file contents (Git blob)
    Content,   // "cnt"
    /// directory (Git tree)
    Directory, // "dir"
    /// VCS commit / changeset
    Revision,  // "rev"
    /// VCS annotated tag / release
    Release,   // "rel"
    /// Snapshot of repository refs
    Snapshot,  // "snp"
}

impl ObjectType {
    pub fn as_tag(self) -> &'static str {
        match self {
            ObjectType::Content => "cnt",
            ObjectType::Directory => "dir",
            ObjectType::Revision => "rev",
            ObjectType::Release => "rel",
            ObjectType::Snapshot => "snp",
        }
    }
    pub fn from_tag(tag: &str) -> Result<Self, SwhidError> {
        match tag {
            "cnt" => Ok(Self::Content),
            "dir" => Ok(Self::Directory),
            "rev" => Ok(Self::Revision),
            "rel" => Ok(Self::Release),
            "snp" => Ok(Self::Snapshot),
            other => Err(SwhidError::InvalidObjectType(other.to_owned())),
        }
    }
}

/// A core SWHID: `swh:1:<tag>:<hex-digest>`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Swhid {
    object_type: ObjectType,
    /// Lowercase hex sha1 digest (20 bytes -> 40 hex chars)
    digest: [u8; 20],
}

impl Swhid {
    pub const VERSION: &'static str = "1";

    pub fn new(object_type: ObjectType, digest: [u8; 20]) -> Self {
        Self { object_type, digest }
    }
    pub fn object_type(&self) -> ObjectType { self.object_type }
    pub fn digest_bytes(&self) -> &[u8; 20] { &self.digest }

    pub fn digest_hex(&self) -> String {
        hex::encode(self.digest)
    }
}

impl Display for Swhid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "swh:{}:{}:{}", Self::VERSION, self.object_type.as_tag(), self.digest_hex())
    }
}

impl FromStr for Swhid {
    type Err = SwhidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Expect: swh:1:<tag>:<digest-hex>
        let mut it = s.split(':');
        let scheme = it.next().ok_or_else(|| SwhidError::InvalidFormat(s.to_owned()))?;
        if scheme != "swh" {
            return Err(SwhidError::InvalidScheme(scheme.to_owned()));
        }
        let ver = it.next().ok_or_else(|| SwhidError::InvalidFormat(s.to_owned()))?;
        if ver != Self::VERSION {
            return Err(SwhidError::InvalidVersion(ver.to_owned()));
        }
        let tag = it.next().ok_or_else(|| SwhidError::InvalidFormat(s.to_owned()))?;
        let object_type = ObjectType::from_tag(tag)?;
        let digest_hex = it.next().ok_or_else(|| SwhidError::InvalidFormat(s.to_owned()))?;

        if it.next().is_some() {
            // too many parts
            return Err(SwhidError::InvalidFormat(s.to_owned()));
        }
        if digest_hex.len() != 40 || !digest_hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(SwhidError::InvalidDigest(digest_hex.to_owned()));
        }
        let mut raw = [0u8; 20];
        hex::decode_to_slice(digest_hex, &mut raw).map_err(|_| SwhidError::InvalidDigest(digest_hex.to_owned()))?;
        Ok(Swhid::new(object_type, raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_core() {
        let id: Swhid = "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391".parse().unwrap();
        assert_eq!(id.object_type(), ObjectType::Content);
        assert_eq!(id.to_string(), "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    }
}
