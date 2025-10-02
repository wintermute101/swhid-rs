use clap::{Parser, Subcommand};
use std::path::PathBuf;

use swhid::{Content, Directory, WalkOptions};
use swhid::{Swhid, QualifiedSwhid};

/// Small CLI for the SWHID reference implementation
#[derive(Parser, Debug)]
#[command(name = "swhid")]
#[command(about = "Compute and parse SWHIDs (ISO/IEC 18670)")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Compute a content SWHID from stdin or a file
    Content {
        /// Path to file (if omitted, read stdin)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Compute a directory SWHID recursively
    Dir {
        /// Directory root
        path: PathBuf,
        /// Follow symlinks (not recommended)
        #[arg(long)]
        follow_symlinks: bool,
        /// Exclude files matching these suffixes (e.g., .tmp, .log)
        #[arg(long, value_name = "SUFFIX")]
        exclude: Vec<String>,
    },
    /// Parse/pretty-print a (qualified) SWHID
    Parse {
        /// The SWHID string
        swhid: String,
    },
    /// Verify that a file or directory matches a given SWHID
    Verify {
        /// Path to file or directory
        path: PathBuf,
        /// Expected SWHID
        swhid: String,
        /// Follow symlinks (not recommended)
        #[arg(long)]
        follow_symlinks: bool,
        /// Exclude files matching these suffixes (e.g., .tmp, .log)
        #[arg(long, value_name = "SUFFIX")]
        exclude: Vec<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Content{ file } => {
            let bytes = if let Some(p) = file {
                std::fs::read(p)?
            } else {
                use std::io::Read;
                let mut buf = Vec::new();
                std::io::stdin().read_to_end(&mut buf)?;
                buf
            };
            let s = Content::from_bytes(bytes).swhid();
            println!("{s}");
        }
        Command::Dir{ path, follow_symlinks, exclude } => {
            let mut opts = WalkOptions { follow_symlinks, ..Default::default() };
            opts.exclude_suffixes = exclude;
            let s = Directory::new(&path).with_options(opts).swhid()?;
            println!("{s}");
        }
        Command::Parse{ swhid } => {
            // Try qualified first, fallback to core
            match swhid.parse::<QualifiedSwhid>() {
                Ok(q) => println!("{q}"),
                Err(_) => {
                    let core: Swhid = swhid.parse()?;
                    println!("{core}");
                }
            }
        }
        Command::Verify{ path, swhid, follow_symlinks, exclude } => {
            let expected: Swhid = swhid.parse()?;
            let actual = if path.is_file() {
                let bytes = std::fs::read(&path)?;
                Content::from_bytes(bytes).swhid()
            } else if path.is_dir() {
                let mut opts = WalkOptions { follow_symlinks, ..Default::default() };
                opts.exclude_suffixes = exclude;
                Directory::new(&path).with_options(opts).swhid()?
            } else {
                eprintln!("Error: {} is neither a file nor a directory", path.display());
                std::process::exit(1);
            };
            
            if actual == expected {
                println!("✓ Verification successful: {} matches {}", path.display(), expected);
                std::process::exit(0);
            } else {
                println!("✗ Verification failed: {} does not match {}", path.display(), expected);
                println!("  Expected: {}", expected);
                println!("  Actual:   {}", actual);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
