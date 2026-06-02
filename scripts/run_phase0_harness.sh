#!/usr/bin/env bash
# cf-invariants-anchor Phase 0 harness runner.
#
# Reproduces, on a Solana-toolchain-equipped host, the two Crucible runs
# that the CI workflow at `.github/workflows/ci.yml` runs on every push:
# clean variant should report 0 violations; planted variant should
# report at least 1 violation with a short counterexample.
#
# Requires on PATH:
#   - cargo (Rust stable, 1.79+ for our workspace; Crucible builds with
#     whatever the Anza platform-tools below carries)
#   - solana CLI (Anza installer, v2.1+ recommended) for `cargo-build-sbf`
#   - crucible CLI (`cargo install --path crates/crucible-fuzz-cli`
#     from an asymmetric-research/crucible @ v0.2.0 clone)
#
# Layout assumption: the upstream Crucible clone lives at
#   <repo-root>/experiments/crucible/
# (a sibling of cf-invariants-anchor under experiments/). That satisfies
# the `../../../../../crucible/...` path deps in
# `references/vault_ref{,_planted}/fuzz/vault_ref/Cargo.toml`. CI sets
# this up automatically; for local reproduction either clone Crucible to
# that path or vendor `crucible-fuzzer` + `crucible-test-context` and
# edit the fuzz Cargo.tomls.

set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS_VERSION="${SOLANA_TOOLS_VERSION:-v1.52}"
CRUCIBLE_TIMEOUT="${CRUCIBLE_TIMEOUT:-30}"

echo "==> cf-invariants-anchor workspace tests"
cargo test --manifest-path "$HERE/Cargo.toml" --workspace

echo
echo "==> cargo build-sbf vault_ref (clean)"
cargo build-sbf \
    --tools-version "$TOOLS_VERSION" \
    --manifest-path "$HERE/references/vault_ref/programs/vault_ref/Cargo.toml"

echo
echo "==> cargo build-sbf vault_ref (planted)"
cargo build-sbf \
    --tools-version "$TOOLS_VERSION" \
    --manifest-path "$HERE/references/vault_ref_planted/programs/vault_ref/Cargo.toml"

echo
echo "==> re-emit invariant fixture from IDL (sanity)"
cargo run --manifest-path "$HERE/Cargo.toml" \
    --bin cf-invariants-anchor --quiet -- \
    emit \
    "$HERE/references/vault_ref/idls/vault_ref.json" \
    --target crucible \
    --out "$HERE/references/vault_ref/fuzz/vault_ref/src/main.rs"
cp "$HERE/references/vault_ref/fuzz/vault_ref/src/main.rs" \
   "$HERE/references/vault_ref_planted/fuzz/vault_ref/src/main.rs"

echo
echo "==> Crucible run on the CLEAN variant (expect: 0 violations within ${CRUCIBLE_TIMEOUT}s)"
( cd "$HERE/references/vault_ref/fuzz/vault_ref" && \
  crucible run vault_ref invariant_amount_conservation \
      --release --timeout "$CRUCIBLE_TIMEOUT" )

echo
echo "==> Crucible run on the PLANTED variant (expect: >=1 violation)"
( cd "$HERE/references/vault_ref_planted/fuzz/vault_ref" && \
  crucible run vault_ref invariant_amount_conservation \
      --release --timeout "$CRUCIBLE_TIMEOUT" || \
  echo "Crucible reported a violation (expected for the planted variant)." )

echo
echo "==> Done. Compare the two runs to:"
echo "    $HERE/findings/vault_ref_clean/scorecard.expected.md"
echo "    $HERE/findings/vault_ref_planted/scorecard.expected.md"
echo "Real captured scorecards (from the CI run) live alongside, at"
echo "    $HERE/findings/vault_ref_{clean,planted}/scorecard.md"
echo "once CI has produced them (see .github/workflows/ci.yml)."
