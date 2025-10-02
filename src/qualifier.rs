use std::fmt::{self, Display};
use std::str::FromStr;

use crate::core::Swhid;
use crate::error::SwhidError;

/// Fragment sub‑selectors
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LineRange {
    pub start: u64,
    pub end: Option<u64>, // inclusive range like "9-15", or single "9"
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ByteRange {
    pub start: u64,
    pub end: Option<u64>,
}

fn parse_range(s: &str) -> Result<(u64, Option<u64>), SwhidError> {
    if let Some((a, b)) = s.split_once('-') {
        let start: u64 = a.parse().map_err(|_| SwhidError::InvalidQualifierValue{ key: "range".into(), value: s.into() })?;
        let end: u64 = b.parse().map_err(|_| SwhidError::InvalidQualifierValue{ key: "range".into(), value: s.into() })?;
        if end < start { return Err(SwhidError::InvalidQualifierValue{ key: "range".into(), value: s.into() }); }
        Ok((start, Some(end)))
    } else {
        let start: u64 = s.parse().map_err(|_| SwhidError::InvalidQualifierValue{ key: "range".into(), value: s.into() })?;
        Ok((start, None))
    }
}

impl Display for LineRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.end {
            Some(e) => write!(f, "{}-{}", self.start, e),
            None => write!(f, "{}", self.start),
        }
    }
}
impl Display for ByteRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.end {
            Some(e) => write!(f, "{}-{}", self.start, e),
            None => write!(f, "{}", self.start),
        }
    }
}

/// Known qualifier keys (order in output is canonicalized).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnownKey { Origin, Visit, Anchor, Path, Lines, Bytes }

impl KnownKey {
    pub fn as_str(self) -> &'static str {
        match self {
            KnownKey::Origin => "origin",
            KnownKey::Visit  => "visit",
            KnownKey::Anchor => "anchor",
            KnownKey::Path   => "path",
            KnownKey::Lines  => "lines",
            KnownKey::Bytes  => "bytes",
        }
    }
}

/// A qualified SWHID with optional qualifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QualifiedSwhid {
    core: Swhid,
    origin: Option<String>,
    visit: Option<Swhid>,
    anchor: Option<Swhid>,
    path: Option<String>,
    lines: Option<LineRange>,
    bytes: Option<ByteRange>,
    // future‑proof: unknown qualifiers we preserve round‑trip
    others: Vec<(String, String)>,
}

impl QualifiedSwhid {
    pub fn new(core: Swhid) -> Self {
        Self { core, origin: None, visit: None, anchor: None, path: None, lines: None, bytes: None, others: vec![] }
    }
    pub fn core(&self) -> &Swhid { &self.core }

    pub fn with_origin(mut self, url: impl Into<String>) -> Self { self.origin = Some(url.into()); self }
    pub fn with_visit(mut self, id: Swhid) -> Self { self.visit = Some(id); self }
    pub fn with_anchor(mut self, id: Swhid) -> Self { self.anchor = Some(id); self }
    pub fn with_path(mut self, path: impl Into<String>) -> Self { self.path = Some(path.into()); self }
    pub fn with_lines(mut self, lines: LineRange) -> Self { self.lines = Some(lines); self }
    pub fn with_bytes(mut self, bytes: ByteRange) -> Self { self.bytes = Some(bytes); self }

    pub fn push_unknown(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.others.push((key.into(), value.into())); self
    }
}

impl Display for QualifiedSwhid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.core)?;
        let mut sep = ';';
        let mut write_kv = |k: &str, v: String, f: &mut fmt::Formatter<'_>| -> fmt::Result {
            write!(f, "{sep}{k}={v}")?; sep = ';'; Ok(())
        };
        if let Some(o) = &self.origin { write_kv("origin", o.clone(), f)?; }
        if let Some(v) = &self.visit  { write_kv("visit", v.to_string(), f)?; }
        if let Some(a) = &self.anchor { write_kv("anchor", a.to_string(), f)?; }
        if let Some(p) = &self.path   { write_kv("path", p.clone(), f)?; }
        if let Some(l) = &self.lines  { write_kv("lines", l.to_string(), f)?; }
        if let Some(b) = &self.bytes  { write_kv("bytes", b.to_string(), f)?; }
        for (k, v) in &self.others { write_kv(k, v.clone(), f)?; }
        Ok(())
    }
}

impl FromStr for QualifiedSwhid {
    type Err = SwhidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (core_str, qstr) = match s.split_once(';') {
            Some((c, rest)) => (c, Some(rest)),
            None => (s, None),
        };
        let core: Swhid = core_str.parse()?;
        let mut q = QualifiedSwhid::new(core);
        if let Some(qstr) = qstr {
            for item in qstr.split(';') {
                if item.is_empty() { continue; }
                let (k, v) = item.split_once('=').ok_or_else(|| SwhidError::InvalidFormat(item.into()))?;
                match k {
                    "origin" => q.origin = Some(v.to_owned()),
                    "visit"  => q.visit  = Some(v.parse()?),
                    "anchor" => q.anchor = Some(v.parse()?),
                    "path"   => q.path   = Some(v.to_owned()),
                    "lines"  => {
                        let (s, e) = super::qualifier::parse_range(v)?;
                        q.lines = Some(LineRange{ start: s, end: e });
                    }
                    "bytes"  => {
                        let (s, e) = super::qualifier::parse_range(v)?;
                        q.bytes = Some(ByteRange{ start: s, end: e });
                    }
                    other => q.others.push((other.to_owned(), v.to_owned())),
                }
            }
        }
        Ok(q)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn roundtrip_qualified() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684".parse().unwrap();
        let q = QualifiedSwhid::new(core.clone())
            .with_origin("https://example.org/repo.git")
            .with_path("/src/lib.rs")
            .with_lines(LineRange{start: 9, end: Some(15)});
        let s = q.to_string();
        assert!(s.contains("origin=https://example.org/repo.git"));
        assert!(s.contains("path=/src/lib.rs"));
        assert!(s.contains("lines=9-15"));

        let parsed: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(parsed.core(), &core);
    }
}
