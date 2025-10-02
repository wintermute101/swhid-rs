use std::fs;
use std::io;
#[cfg(unix)] use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::core::{ObjectType, Swhid};
use crate::hash::{hash_blob, hash_object};

/// Options for directory walking / hashing.
#[derive(Debug, Clone, Default)]
pub struct WalkOptions {
    /// Whether to follow symlinks (note: not recommended; SWH uses link targets)
    pub follow_symlinks: bool,
    /// Exclude glob patterns (very minimal: literal suffix match)
    pub exclude_suffixes: Vec<String>,
}

#[derive(Debug, Clone)]
struct Entry {
    name: Vec<u8>, // raw bytes (no encoding assumptions)
    mode: u32,     // git tree mode
    id: [u8; 20],  // object id
}

fn is_excluded(name: &[u8], opts: &WalkOptions) -> bool {
    if opts.exclude_suffixes.is_empty() { return false; }
    let s = String::from_utf8_lossy(name);
    opts.exclude_suffixes.iter().any(|suf| s.ends_with(suf))
}

/// Compute the Git tree payload (concatenation of entries).
fn dir_payload(mut children: Vec<Entry>) -> Vec<u8> {
    // Git sorts by raw bytes of filename (no path separators here).
    children.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = Vec::new();
    for e in children {
        // "<mode> <name>\0<id-bytes>"
        let mut mode = format!("{:06o}", e.mode).into_bytes();
        out.append(&mut mode);
        out.push(b' ');
        out.extend_from_slice(&e.name);
        out.push(0);
        out.extend_from_slice(&e.id);
    }
    out
}

fn path_file_mode(meta: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        let m = meta.permissions().mode();
        let exec = (m & 0o111) != 0;
        if meta.is_dir()       { 0o040000 }
        else if meta.is_file() { return if exec { 0o100755 } else { 0o100644 }; }
        else { return 0o100644; }
    }
    #[cfg(not(unix))]
    {
        if meta.is_dir()       { 0o040000 } else { 0o100644 }
    }
}

fn symlink_mode() -> u32 { 0o120000 }

fn hash_dir_inner(path: &Path, opts: &WalkOptions) -> io::Result<[u8; 20]> {
    let mut children: Vec<Entry> = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_bytes = file_name.as_os_str().as_encoded_bytes().to_vec();

        if is_excluded(&name_bytes, opts) { continue; }

        let md = if opts.follow_symlinks { fs::metadata(entry.path())? } else { fs::symlink_metadata(entry.path())? };
        let ft = md.file_type();

        if ft.is_dir() {
            let id = hash_dir_inner(&entry.path(), opts)?;
            children.push(Entry{ name: name_bytes, mode: 0o040000, id });
        } else if ft.is_symlink() {
            // The blob is the link target bytes
            let target = fs::read_link(entry.path())?;
            let bytes = target.as_os_str().as_encoded_bytes();
            let id = hash_blob(bytes);
            children.push(Entry{ name: name_bytes, mode: symlink_mode(), id });
        } else if ft.is_file() {
            let bytes = fs::read(entry.path())?;
            let id = hash_blob(&bytes);
            let mode = path_file_mode(&md);
            children.push(Entry{ name: name_bytes, mode, id });
        } else {
            // ignore special files
            continue;
        }
    }
    let payload = dir_payload(children);
    Ok(hash_object("tree", &payload))
}

/// Helper wrapper to compute directory SWHID
#[derive(Debug, Clone)]
pub struct Directory<'a> {
    root: &'a Path,
    opts: WalkOptions,
}

impl<'a> Directory<'a> {
    pub fn new(root: &'a Path) -> Self { Self { root, opts: WalkOptions::default() } }
    pub fn with_options(mut self, opts: WalkOptions) -> Self { self.opts = opts; self }

    pub fn swhid(&self) -> io::Result<Swhid> {
        let id = hash_dir_inner(self.root, &self.opts)?;
        Ok(Swhid::new(ObjectType::Directory, id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    

    #[test]
    fn simple_dir_hash() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("a.txt").write_str("A").unwrap();
        tmp.child("b.txt").write_str("B").unwrap();

        let dir = Directory::new(tmp.path());
        let id = dir.swhid().unwrap();
        // Just sanity: should be 40 hex chars
        assert_eq!(id.to_string().len(), "swh:1:dir:".len() + 40);
    }
}
