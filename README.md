# swhid-rs — SWHID v1.2 Reference Implementation in Rust

A comprehensive implementation of the SWHID v1.2 specification as defined in **ISO/IEC 18670:2025**. This crate provides a complete reference implementation for parsing, generating, and working with SWHID v1.2 identifiers in Rust.

> ℹ️ **SWHID v1.2 Compliant** — This implementation fully adheres to the SWHID v1.2 specification and ISO/IEC 18670:2025 standard, with the exception of using collision-detecting SHA-1 instead of regular SHA1.

## Features

### Core SWHID v1.2 Support
- **Complete SWHID v1.2 parsing and formatting** — Robust validation with comprehensive error handling
- **All 5 SWHID v1.2 object types** — Content (`cnt`), Directory (`dir`), Revision (`rev`), Release (`rel`), Snapshot (`snp`)
- **Qualified SWHIDs** — Full support for all SWHID v1.2 qualifiers (origin, visit, anchor, path, lines, bytes)
- **Case-sensitive validation** — Strict adherence to SWHID v1.2 specification (lowercase hex only)

### SWHID v1.2 Content & Directory Hashing
- **Content SWHIDs** — Compute `swh:1:cnt:<digest>` for any byte data using SWHID v1.2 algorithms
- **Directory SWHIDs** — Compute `swh:1:dir:<digest>` for directory trees using SWHID v1.2 algorithms
- **SHA-1 with collision detection** — Uses collision-detecting SHA-1 for enhanced security
- **Symlink handling** — SWHID v1.2 specification compliant symlink processing
- **File exclusion** — Configurable patterns to exclude files from directory hashing

### VCS Integration (Optional)
- **Git feature flag** — Enable SWHID v1.2 VCS object computation with `--features git`
- **Revision SWHIDs** — Compute `swh:1:rev:<digest>` from Git commits using SWHID v1.2 algorithms
- **Release SWHIDs** — Compute `swh:1:rel:<digest>` from Git tags using SWHID v1.2 algorithms
- **Snapshot SWHIDs** — Compute `swh:1:snp:<digest>` from repository state using SWHID v1.2 algorithms
- **Repository analysis** — List tags, analyze commit history for SWHID v1.2 computation

### CLI Tool
```bash
# Content SWHIDs
swhid content --file README.md
echo "Hello, World!" | swhid content

# Directory SWHIDs
swhid dir .
swhid dir --exclude-suffix .tmp --exclude-suffix .log /path/to/project

# VCS SWHIDs (requires --features git)
swhid git revision --repo /path/to/git/repo
swhid git release --repo /path/to/git/repo --tag v1.0.0
swhid git snapshot --repo /path/to/git/repo
swhid git tags --repo /path/to/git/repo

# Parse and validate SWHIDs
swhid parse 'swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391'
swhid parse 'swh:1:dir:...;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20'

# Verify SWHIDs
swhid verify --file README.md --expected 'swh:1:cnt:...'
```

### Performance & Quality
- **Performance benchmarks** — Criterion-based benchmarking suite
- **Error handling** — Comprehensive error types with detailed messages

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
swhid = "0.2.0"

# Optional features
[dependencies.swhid]
version = "0.2.0"
features = ["serde", "git"]  # Enable serialization and Git support
```

## Usage Examples

### Basic SWHID v1.2 Operations

```rust
use swhid::{Swhid, ObjectType, Content, Directory, QualifiedSwhid};

// Parse a SWHID v1.2 identifier
let swhid: Swhid = "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391".parse()?;
println!("Object type: {:?}", swhid.object_type()); // Content
println!("Digest: {}", swhid.digest_hex());

// Create SWHID v1.2 content identifier
let content = Content::from_bytes(b"Hello, World!");
let swhid = content.swhid();
println!("Content SWHID: {}", swhid);

// Create SWHID v1.2 directory identifier
let dir = Directory::new(Path::new("/path/to/directory"));
let swhid = dir.swhid()?;
println!("Directory SWHID: {}", swhid);
```

### Qualified SWHID v1.2 Identifiers

```rust
use swhid::{Swhid, QualifiedSwhid};

let core: Swhid = "swh:1:cnt:...".parse()?;
let qualified = QualifiedSwhid::new(core)
    .with_origin("https://github.com/user/repo")
    .with_path("/src/main.rs")
    .with_lines(10, Some(20))
    .with_bytes(100, Some(200));

println!("Qualified SWHID: {}", qualified);
// Output: swh:1:cnt:...;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20;bytes=100-200
```

### VCS Integration (Git Feature)

```rust
#[cfg(feature = "git")]
use swhid::git;

#[cfg(feature = "git")]
{
    let repo = git::open_repo("/path/to/git/repo")?;
    
    // Get HEAD commit SWHID v1.2
    let head_commit = git::get_head_commit(&repo)?;
    let revision_swhid = git::revision_swhid(&repo, &head_commit)?;
    
    // Get tag SWHID v1.2
    let tag_oid = repo.refname_to_id("refs/tags/v1.0.0")?;
    let release_swhid = git::release_swhid(&repo, &tag_oid)?;
    
    // Get snapshot SWHID v1.2
    let snapshot_swhid = git::snapshot_swhid(&repo, &head_commit)?;
}
```

## Features

| Feature | Description |
|---------|-------------|
| `serde` | Enable `Serialize`/`Deserialize` for all public types |
| `git` | Enable VCS integration for SWHID v1.2 revision/release/snapshot computation |

## Performance

The implementation is optimized for performance:

- **Zero-copy parsing** where possible
- **Efficient directory traversal** with configurable options
- **Memory-efficient hashing** using streaming APIs
- **Comprehensive benchmarks** covering all major operations

Run benchmarks with:
```bash
cargo bench
```

## Testing

The implementation includes comprehensive tests covering:

- **Content hashing** — Various data types, edge cases, unicode
- **Directory processing** — Nested structures, symlinks, exclusions
- **SWHID parsing** — Valid/invalid formats, all object types
- **Qualified SWHIDs** — All qualifier types, parsing, formatting
- **Git integration** — Repository operations, commit/tag analysis
- **Error handling** — Comprehensive error scenarios

Run tests with:
```bash
cargo test
cargo test --all-features  # Include Git tests
```

## Correctness & Scope

- **SWHID v1.2 compliant** — Full adherence to the SWHID v1.2 specification and ISO/IEC 18670:2025, with the exception of using collision-detecting SHA-1 instead of regular SHA1
- **VCS-compatible hashing** — Uses SWHID v1.2 algorithms compatible with Git for VCS objects
- **Cross-platform** — Works on Unix and non-Unix systems
- **Security-focused** — Uses collision-detecting SHA-1 for enhanced security

### Platform Notes
- **Unix systems** — Respects executable permissions for SWHID v1.2 compliance
- **Non-Unix systems** — Regular files default to 100644 mode per SWHID v1.2 specification
- **Special files** — Sockets, FIFOs, devices are ignored per SWHID v1.2 specification
- **Symlinks** — Handled according to SWHID v1.2 specification

## License

Dual-licensed under **MIT** or **Apache-2.0** at your option.

## References

- **SWHID v1.2 Specification** — [Software Heritage Identifier specification v1.2](https://docs.softwareheritage.org/devel/swh-model/persistent-identifiers.html)
- **ISO/IEC 18670:2025** — International standard for Software Heritage Identifiers
- **Software Heritage** — [softwareheritage.org](https://www.softwareheritage.org/)

## Contributing

Contributions are welcome! Please ensure:

- All tests pass: `cargo test --all-features`
- Code is formatted: `cargo fmt`
- No clippy warnings: `cargo clippy --all-features`
- Benchmarks still pass: `cargo bench`
- SWHID v1.2 compliance is maintained
