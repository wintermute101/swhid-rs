//! SWHID v1.2 VCS integration for Git repositories
//!
//! This module provides SWHID v1.2 compliant functionality to compute SWHIDs
//! from Git repository objects when the `git` feature is enabled:
//! - Revision SWHIDs (commits) - `swh:1:rev:<digest>`
//! - Release SWHIDs (tags) - `swh:1:rel:<digest>`
//! - Snapshot SWHIDs (repository state) - `swh:1:snp:<digest>`
//!
//! This module implements the SWHID v1.2 specification for VCS objects,
//! using Git as the reference VCS implementation.

use crate::error::SwhidError;
use crate::Swhid;
use std::path::Path;

use git2::{ObjectType as GitObjectType, Repository, Signature};

use crate::release::Release;
use crate::revision::Revision;
use crate::Bytestring;

fn io_error(msg: String) -> SwhidError {
    SwhidError::Io(std::io::Error::other(msg))
}

fn oid_to_array(oid: git2::Oid) -> Result<[u8; 20], SwhidError> {
    oid.as_bytes()
        .try_into()
        .map_err(|e| io_error(format!("Unexpected tree_oid length: {e}")))
}

fn parse_signature(sig: Signature) -> (Bytestring, i64, Bytestring) {
    let name = sig.name_bytes();
    let email = sig.email_bytes();

    let mut full_name = Vec::with_capacity(name.len() + email.len() + 3);
    full_name.extend_from_slice(name);
    full_name.extend_from_slice(b" <");
    full_name.extend_from_slice(email);
    full_name.push(b'>');

    let when = sig.when();
    let offset_minutes = when.offset_minutes();
    let offset_hours = offset_minutes / 60;
    let offset_minutes = offset_minutes % 60;
    let sign = when.sign();
    let offset = format!("{sign}{offset_hours:02}{offset_minutes:02}");

    (full_name.into(), when.seconds(), offset.into_bytes().into())
}

/// Compute a SWHID v1.2 revision identifier froma Git commit
///
/// This implements the SWHID v1.2 revision hashing algorithm for Git commits,
/// creating a `swh:1:rev:<digest>` identifier according to the specification.
pub fn revision_swhid(repo: &Repository, commit_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    let commit = repo
        .find_commit(*commit_oid)
        .map_err(|e| io_error(format!("Failed to find commit: {e}")))?;

    let tree = commit
        .tree()
        .map_err(|e| io_error(format!("Failed to get commit tree: {e}")))?;

    let tree_oid = tree.id();

    let (author, author_timestamp, author_timestamp_offset) = parse_signature(commit.author());
    let (committer, committer_timestamp, committer_timestamp_offset) =
        parse_signature(commit.committer());

    let revision = Revision {
        directory: oid_to_array(tree_oid)?,
        parents: commit
            .parents()
            .map(|parent| oid_to_array(parent.id()))
            .collect::<Result<_, _>>()?,
        author,
        author_timestamp,
        author_timestamp_offset,
        committer,
        committer_timestamp,
        committer_timestamp_offset,
        extra_headers: Vec::new(), // FIXME: does not seem to be exposed by git2
        message: Some(commit.message_bytes().into()),
    };

    Ok(revision.swhid())
}

/// Compute a SWHID v1.2 release identifier from a Git tag
///
/// This implements the SWHID v1.2 release hashing algorithm for Git tags,
/// creating a `swh:1:rel:<digest>` identifier according to the specification.
pub fn release_swhid(repo: &Repository, tag_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    use crate::release::ObjectType;

    let tag = repo
        .find_tag(*tag_oid)
        .map_err(|e| io_error(format!("Failed to find tag: {e}")))?;

    let target = tag
        .target()
        .map_err(|e| io_error(format!("Failed to get tag target: {e}")))?;
    let target_oid = target.id();

    let (author, author_timestamp, author_timestamp_offset) = match tag
        .tagger() {
            Some(tagger) => {
                let (author, author_timestamp, author_timestamp_offset) = parse_signature(tagger);
                (Some(author), Some(author_timestamp), Some(author_timestamp_offset))
            },
            None => (None, None, None)
    };

    let release = Release {
        object: oid_to_array(target_oid)?,
        object_type: match target.kind() {
            Some(GitObjectType::Commit) => ObjectType::Revision,
            Some(GitObjectType::Tree) => ObjectType::Directory,
            Some(GitObjectType::Blob) => ObjectType::Content,
            Some(GitObjectType::Tag) => ObjectType::Release,
            _ => return Err(io_error("Unknown target type".to_string())),
        },
        name: tag.name_bytes().into(),
        author,
        author_timestamp,
        author_timestamp_offset,
        extra_headers: Vec::new(), // FIXME: does not seem to be exposed by git2
        message: tag.message_bytes().map(Into::into),
    };

    Ok(release.swhid())
}

/// Compute a SWHID v1.2 snapshot identifier from a Git repository
///
/// This implements the SWHID v1.2 snapshot hashing algorithm for Git repositories,
/// creating a `swh:1:snp:<digest>` identifier according to the specification.
pub fn snapshot_swhid(repo: &Repository, commit_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    let commit = repo
        .find_commit(*commit_oid)
        .map_err(|e| io_error(format!("Failed to find commit: {e}")))?;

    let _tree = commit
        .tree()
        .map_err(|e| io_error(format!("Failed to get commit tree: {e}")))?;

    // Create snapshot content
    let mut snapshot_content = Vec::new();

    // Add revision SWHID
    let revision = revision_swhid(repo, commit_oid)?;
    snapshot_content.extend_from_slice(b"revision ");
    snapshot_content.extend_from_slice(revision.to_string().as_bytes());
    snapshot_content.push(b'\n');

    // Add directory SWHID
    let dir_swhid =
        crate::directory::DiskDirectoryBuilder::new(repo.path().parent().unwrap_or(Path::new(".")))
            .swhid()?;
    snapshot_content.extend_from_slice(b"directory ");
    snapshot_content.extend_from_slice(dir_swhid.to_string().as_bytes());
    snapshot_content.push(b'\n');

    let digest = crate::hash::hash_swhid_object("snapshot", &snapshot_content);
    Ok(Swhid::new(crate::ObjectType::Snapshot, digest))
}

/// Open a Git repository for SWHID v1.2 computation
///
/// This function opens a Git repository to enable SWHID v1.2 computation
/// for revision, release, and snapshot objects.
pub fn open_repo(path: &Path) -> Result<Repository, SwhidError> {
    Repository::open(path).map_err(|e| io_error(format!("Failed to open repository: {e}")))
}

/// Get the HEAD commit of a Git repository for SWHID v1.2 computation
pub fn get_head_commit(repo: &Repository) -> Result<git2::Oid, SwhidError> {
    let head = repo
        .head()
        .map_err(|e| io_error(format!("Failed to get HEAD: {e}")))?;

    head.target()
        .ok_or_else(|| io_error("HEAD is not a direct reference".to_string()))
}

/// Get all tags in a Git repository for SWHID v1.2 release computation
pub fn get_tags(repo: &Repository) -> Result<Vec<git2::Oid>, SwhidError> {
    let mut tags = Vec::new();
    let tag_names = repo
        .tag_names(None)
        .map_err(|e| io_error(format!("Failed to get tag names: {e}")))?;

    for tag_name in tag_names.iter().flatten() {
        if let Ok(tag_oid) = repo.refname_to_id(&format!("refs/tags/{tag_name}")) {
            tags.push(tag_oid);
        }
    }

    Ok(tags)
}

