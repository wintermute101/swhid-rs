use sha1collisiondetection::{Digest, Sha1CD};

/// Build a Git object header bytes: `<type> <len>\0`
pub fn git_object_header(typ: &str, len: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(typ.len() + 1 + 20 + 1);
    v.extend_from_slice(typ.as_bytes());
    v.push(b' ');
    v.extend_from_slice(len.to_string().as_bytes());
    v.push(0);
    v
}

/// Hash "blob" content like Git does.
pub fn hash_blob(data: &[u8]) -> [u8; 20] {
    let header = git_object_header("blob", data.len());
    let mut hasher = Sha1CD::new();
    hasher.update(&header);
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash arbitrary Git object given its type and payload bytes.
pub fn hash_object(typ: &str, payload: &[u8]) -> [u8; 20] {
    let header = git_object_header(typ, payload.len());
    let mut hasher = Sha1CD::new();
    hasher.update(&header);
    hasher.update(payload);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_blob_is_git_known_value() {
        let h = hash_blob(&[]);
        // e69de29bb2d1d6434b8b29ae775ad8c2e48c5391
        assert_eq!(hex::encode(h), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    }

    #[test]
    fn hello_world_blob() {
        let h = hash_blob(b"Hello, World!");
        assert_eq!(hex::encode(h), "b45ef6fec89518d314f546fd6c3025367b721684");
    }
}
