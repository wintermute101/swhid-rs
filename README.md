# swhid — minimal SWHID (ISO/IEC 18670) implementation in Rust

This crate offers a **clean**, **minimal**, and **well‑documented** implementation
of the SWHID core format and key qualifiers. It aims to be a good **reference**
for implementers who need a small, dependency‑light codebase to:
- parse / pretty‑print SWHIDs,
- compute **content** (`cnt`) and **directory** (`dir`) intrinsic identifiers locally,
- assemble **qualified** identifiers (origin/visit/anchor/path/lines/bytes).

> ℹ️ For the full normative specification see the public spec v1.2 and the ISO/IEC 18670:2025 standard.

## Features
- `core` — `Swhid` type with `Display`/`FromStr`, robust validation.
- `qualifier` — `QualifiedSwhid` with known qualifiers; preserves unknown key/values.
- `hash` — Git‑compatible object hashing (`blob`, `tree`) using collision-detecting SHA‑1.
- `content` — compute `swh:1:cnt:<digest>` for in‑memory bytes.
- `directory` — compute `swh:1:dir:<digest>` for a local directory tree.
- `serde` (feature) — opt‑in `Serialize`/`Deserialize` for public types.

## CLI
```
# content from file
swhid content --file README.md

# content from stdin
cat README.md | swhid content

# directory recursively
swhid dir .

# parse (qualified) SWHID and print in canonical order
swhid parse 'swh:1:cnt:...;path=/src/lib.rs;lines=9-15'
```

## Notes on correctness & scope
- `cnt` and `dir` algorithms follow Git object hashing.
- On Unix, executable bit is respected; on non‑Unix, regular files default to 100644.
- Special files (sockets, fifos, devices) are ignored.
- Computing `rev`/`rel`/`snp` requires VCS metadata and is intentionally out of scope.

## License
Dual‑licensed under **MIT** or **Apache‑2.0** at your option.
