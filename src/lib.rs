//! SWHID minimal reference implementation
//!
//! This crate provides a **clean, small, dependency‑light** implementation
//! of the SWHID (SoftWare Hash IDentifier) format defined in
//! **ISO/IEC 18670:2025** and detailed in the public specification v1.2.
//!
//! Covered here:
//! - Core identifier representation and parsing/printing (`swh:1:<tag>:<id>`)
//! - Known object tags: contents (`cnt`), directories (`dir`), revisions (`rev`),
//!   releases (`rel`), snapshots (`snp`)
//! - Qualified identifiers (origin, visit, anchor, path, lines, bytes)
//! - Minimal hash computation for **content** (Git blob) and **directory** (Git tree)
//!
//! Not covered here (by design, to stay minimal):
//! - Computing `rev`, `rel`, `snp` intrinsic IDs (they depend on VCS metadata)
//! - Archive traversal or fetching – only local file/dir hashing is implemented
//!
//! The hashing logic follows Git’s object hashing (blob/tree) using SHA‑1.
//! A `sha1dc` feature is exposed to enable collision detection via
//! the `sha1collisiondetection` crate if you want defense‑in‑depth.
//!
//! ## Example
//!
//! ```no_run
//! use swhid::{Content, Directory, ObjectType, Swhid, QualifiedSwhid};
//!
//! // Content hash
//! let blob = Content::from_bytes(b"Hello, world!");
//! let s = blob.swhid();
//! assert_eq!(s.to_string(), "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684");
//!
//! // Parse & format
//! let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684".parse().unwrap();
//! assert_eq!(core.object_type(), ObjectType::Content);
//!
//! // Qualified identifiers
//! let q = QualifiedSwhid::new(core).with_path("/src/lib.rs");
//! assert!(q.to_string().contains(";path=/src/lib.rs"));
//! ```
//!
pub mod core;
pub mod qualifier;
pub mod hash;
pub mod content;
pub mod directory;
pub mod error;
pub mod git;

pub use crate::core::{ObjectType, Swhid};
pub use crate::qualifier::{QualifiedSwhid, LineRange, ByteRange};
pub use crate::content::Content;
pub use crate::directory::{Directory, WalkOptions};

#[cfg(feature="serde")]
pub use serde::{Serialize, Deserialize};
