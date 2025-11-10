use crate::utils::HeaderWriter;
use crate::{Bytestring, Swhid};

pub enum ObjectType {
    Revision,
    Directory,
    Release,
    Content,
}

pub struct Release {
    pub object: [u8; 20],
    pub object_type: ObjectType,
    pub name: Bytestring,
    pub author: Option<Bytestring>,
    pub author_timestamp: Option<i64>,
    pub author_timestamp_offset: Option<Bytestring>,
    pub extra_headers: Vec<(Bytestring, Bytestring)>,
    pub message: Option<Bytestring>,
}

impl Release {
    /// Compute a SWHID v1.2 revision identifier from a Git commit
    ///
    /// This implements the SWHID v1.2 revision hashing algorithm for Git commits,
    /// creating a `swh:1:rev:<digest>` identifier according to the specification.
    pub fn swhid(&self) -> Swhid {
        let manifest = rel_manifest(self);
        let digest = crate::hash::hash_swhid_object("tag", &manifest);

        Swhid::new(crate::ObjectType::Revision, digest)
    }
}

pub fn rel_manifest(rev: &Release) -> Vec<u8> {
    let Release {
        object,
        object_type,
        name,
        author,
        author_timestamp,
        author_timestamp_offset,
        extra_headers,
        message,
    } = rev;
    let mut writer = HeaderWriter::default();

    writer.push(b"object", object);
    writer.push(
        b"type",
        match object_type {
            ObjectType::Revision => b"commit".as_ref(),
            ObjectType::Directory => b"tree".as_ref(),
            ObjectType::Release => b"release".as_ref(),
            ObjectType::Content => b"blob".as_ref(),
        },
    );
    writer.push(b"tag", name);

    match (author, author_timestamp, author_timestamp_offset) {
        (Some(author), Some(author_timestamp), Some(author_timestamp_offset)) => 
            writer.push_authorship(
                b"author",
                author,
                *author_timestamp,
                author_timestamp_offset,
            ),
        (None, None, None) => (),
        _ => (), // unspecified, see https://github.com/swhid/specification/issues/62
    }

    for (key, value) in extra_headers {
        writer.push(key, value)
    }

    writer.build(message.as_ref())
}
