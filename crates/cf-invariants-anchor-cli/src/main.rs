// cf-invariants-anchor CLI.
//
// Phase 0 surface:
//   cf-invariants-anchor version
//   cf-invariants-anchor ingest <idl.json> [--out -]
//   cf-invariants-anchor suggest <idl.json> [--out -]
//   cf-invariants-anchor emit <idl.json> [--target crucible|trident] [--out path]
//
// `run` (drives `crucible run`) is staged for Phase 1 — Phase 0 ships
// the scorecard renderer separately for the deliberately-seeded
// reference run captured in findings/.

use anyhow::Result;
use cf_invariants_anchor_emit::Target;
use cf_invariants_anchor_suggest::ClassRegistry;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "cf-invariants-anchor",
    version,
    about = "AI invariant-author for Crucible / Trident on Anchor programs."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print cf-invariants-anchor version + target Crucible version.
    Version,

    /// Parse an Anchor IDL into a typed contract surface (JSON to stdout).
    Ingest {
        idl: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },

    /// Propose ranked candidate invariants (Phase 0: balance_conservation).
    Suggest {
        idl: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },

    /// Emit a Crucible-compatible (or Trident-stub) fuzz fixture file.
    Emit {
        idl: PathBuf,
        #[arg(long, default_value = "crucible")]
        target: String,
        #[arg(long, default_value_t = 0)]
        candidate_index: usize,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

const CRUCIBLE_TARGET_VERSION: &str = "0.2.0";

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Version => {
            println!(
                "cf-invariants-anchor {} (target: Crucible v{})",
                env!("CARGO_PKG_VERSION"),
                CRUCIBLE_TARGET_VERSION
            );
        }
        Command::Ingest { idl, out } => {
            let surface = cf_invariants_anchor_idl::ingest_path(&idl)?;
            let text = serde_json::to_string_pretty(&surface)?;
            write_out(out, &text)?;
        }
        Command::Suggest { idl, out } => {
            let surface = cf_invariants_anchor_idl::ingest_path(&idl)?;
            let registry = ClassRegistry::phase0();
            let candidates = registry.propose_all(&surface);
            let text = serde_json::to_string_pretty(&candidates)?;
            write_out(out, &text)?;
        }
        Command::Emit {
            idl,
            target,
            candidate_index,
            out,
        } => {
            let surface = cf_invariants_anchor_idl::ingest_path(&idl)?;
            let registry = ClassRegistry::phase0();
            let candidates = registry.propose_all(&surface);
            let candidate = candidates
                .get(candidate_index)
                .ok_or_else(|| anyhow::anyhow!("no candidate at index {candidate_index}"))?;
            let target = parse_target(&target)?;
            let rendered = cf_invariants_anchor_emit::render(&surface, candidate, target);
            write_out(out, &rendered)?;
        }
    }
    Ok(())
}

fn parse_target(s: &str) -> Result<Target> {
    match s.to_ascii_lowercase().as_str() {
        "crucible" => Ok(Target::Crucible),
        "trident" => Ok(Target::Trident),
        other => anyhow::bail!("unknown target `{other}` (expected `crucible` | `trident`)"),
    }
}

fn write_out(out: Option<PathBuf>, text: &str) -> Result<()> {
    match out {
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(path, text)?;
        }
        None => {
            print!("{text}");
            if !text.ends_with('\n') {
                println!();
            }
        }
    }
    Ok(())
}
