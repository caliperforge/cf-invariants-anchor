// Internal helper: render a Scorecard JSON file to markdown using the
// same renderer the CLI's `run` subcommand will use in Phase 1.
//
//   cargo run --bin render_scorecard -- <scorecard.json> > scorecard.md

use cf_invariants_anchor_core::Scorecard;
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: render_scorecard <scorecard.json>");
            return ExitCode::from(2);
        }
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("read {path}: {e}");
            return ExitCode::from(1);
        }
    };
    let sc: Scorecard = match serde_json::from_slice(&bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("parse {path}: {e}");
            return ExitCode::from(1);
        }
    };
    print!("{}", cf_invariants_anchor_report::render_markdown(&sc));
    ExitCode::SUCCESS
}
