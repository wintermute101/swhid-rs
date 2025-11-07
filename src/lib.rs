#![doc = include_str!("../README.md")]

pub mod content;
pub mod directory;
pub mod error;
pub mod git;

pub use crate::core::{ObjectType, Swhid};
pub use crate::qualifier::{QualifiedSwhid, LineRange, ByteRange};
pub use crate::content::Content;
pub use crate::directory::{Directory, DiskDirectoryBuilder, WalkOptions};

#[cfg(feature="serde")]
pub use serde::{Serialize, Deserialize};

mod readme_test {
    #![doc = include_str!("../README.md")]
}
