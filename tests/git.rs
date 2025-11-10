#![cfg(feature = "git")]

use std::path::Path;

use assert_fs::prelude::*;
use git2::Repository;

use swhid::git::*;
use swhid::ObjectType;

#[test]
fn test_revision_swhid() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let repo = git2::Repository::init(tmp.path()).unwrap();

    // Create a simple commit
    let mut index = repo.index().unwrap();
    let file_path = tmp.child("test.txt");
    file_path.write_str("test content").unwrap();

    index
        .add_path(file_path.path().strip_prefix(tmp.path()).unwrap())
        .unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();

    let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
    let commit_oid = repo
        .commit(
            Some("refs/heads/main"),
            &sig,
            &sig,
            "Test commit",
            &tree,
            &[],
        )
        .unwrap();

    let swhid = revision_swhid(&repo, &commit_oid).unwrap();
    assert_eq!(swhid.object_type(), ObjectType::Revision);
}
