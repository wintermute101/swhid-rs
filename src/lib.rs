#![doc = include_str!("../README.md")]

pub mod content;
pub mod core;
pub mod directory;
pub mod error;
pub mod git;
pub mod hash;
pub mod qualifier;

pub use crate::content::Content;
pub use crate::core::{ObjectType, Swhid};
pub use crate::directory::{Directory, DiskDirectoryBuilder, WalkOptions};
pub use crate::qualifier::{ByteRange, LineRange, QualifiedSwhid};

#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};

mod readme_test {
    #![doc = include_str!("../README.md")]
}
