# cf-invariants-anchor — live validation status

**Updated 2026-06-01 (post-fix dispatch):** pin repinned, CI authored. Awaiting
the first green CI run; the artifact is publishable once
[![ci](https://github.com/caliperforge/cf-invariants-anchor/actions/workflows/ci.yml/badge.svg)](https://github.com/caliperforge/cf-invariants-anchor/actions/workflows/ci.yml)
is green on `main`.

## What changed in this dispatch

1. **Version pin fixed (empirically verified against the upstream tag).**
   Pulled `Cargo.toml` from `asymmetric-research/crucible @ v0.2.0` directly
   (not via code-shape inspection):

   ```
   [workspace.dependencies]
   anchor-lang = "1.0.1"
   anchor-spl  = "1.0.1"
   litesvm     = "0.9.0"
   solana-*    = "3.0"
   ```

   Repinned the four affected `Cargo.toml`s in this repo from
   `anchor-lang = "0.30.1"` to `anchor-lang = "1.0.1"`:

   - `references/vault_ref/programs/vault_ref/Cargo.toml`
   - `references/vault_ref/fuzz/vault_ref/Cargo.toml`
   - `references/vault_ref_planted/programs/vault_ref/Cargo.toml`
   - `references/vault_ref_planted/fuzz/vault_ref/Cargo.toml`

   Also added the upstream-standard `idl-build = ["anchor-lang/idl-build"]`
   feature on the program crates to match the v0.2.0 escrow example. No
   source-level porting was needed — the vault programs already use
   Anchor 0.30+/1.0 shared idioms (`#[program]`, `Context<T>`,
   `#[derive(Accounts)]`, `Account<'info, T>`, `system_program::Transfer`,
   `#[account] #[derive(InitSpace)]`, `#[error_code]`) which the v0.2.0
   escrow example also uses verbatim.

2. **CI authored.** `.github/workflows/ci.yml` installs Rust + Anza Solana
   CLI v2.1.21 (with `--tools-version v1.52` platform-tools, required
   by Crucible's edition2024 deps), clones Crucible v0.2.0, installs
   `crucible-fuzz-cli`, runs `cargo test --workspace`, builds both
   vault programs via `cargo build-sbf`, re-emits the fixture, runs
   the Crucible harness on both variants, and asserts:
   - **clean**: no `FUZZ_FINDING` or `INVARIANT VIOLATED` marker in output
   - **planted**: at least one such marker OR a non-zero exit code

   Captured stdout from each Crucible run is written to
   `findings/vault_ref_{clean,planted}/scorecard.md` (real captures;
   distinct from the existing `.expected.{json,md}` authored references)
   and uploaded as the `crucible-scorecards` artifact.

3. **Honesty pass on the repo.** The README's status section now points
   at the CI badge as the source of truth; the `docs/architecture.md`
   "Crucible API verification record" was softened — what was
   confirmed is the upstream surface (verified by fetching `Cargo.toml`
   and `src/` from the v0.2.0 tag), what is *proven* (the harness
   actually builds and behaves) is exclusively the CI run.

## What is still pending

- The first green CI run, on a private repo. The repo lives at
  `experiments/cf-invariants-anchor/` in the operator's monorepo;
  Director will run the staged `gh repo create --private` + push commands
  (see the dispatch's done log). CI executes on push.
- Once green: the `findings/vault_ref_{clean,planted}/scorecard.md`
  files that CI commits/uploads become the captured reference. We
  keep the `scorecard.expected.{json,md}` authored siblings for
  comparison.
- If the real captured runs diverge materially from the authored
  expectations, a follow-up dispatch tunes either the planted bug or
  the invariant before flipping the repo public.

## Standing rule encoded in this repo

> **"Verified" = actually built and run (in CI or locally), never code-shape
> eyeballing. No artifact is publishable until CI is green.**

This is the rule that broke on the previous "verified against v0.2.0"
claim (the prior verification looked at function/macro names without
pulling the tag's `Cargo.toml`, missing the pinned-version drift). The
fix is structural: CI runs the harness end-to-end on every push, so any
future drift fails the badge immediately and blocks publication.

## Local reproduction is optional

Per the dispatch's "do not burden the Mac with a heavy toolchain"
constraint, this artifact has NOT been built or run on the operator's
host (Rust 1.96, macOS 26.5 arm64). The cloud is the runner. To
reproduce locally on a Solana-equipped host:

```bash
# 1. Clone Crucible at the v0.2.0 tag, as a sibling of cf-invariants-anchor.
git clone --branch v0.2.0 https://github.com/asymmetric-research/crucible.git ../crucible

# 2. Install crucible CLI.
cargo install --path ../crucible/crates/crucible-fuzz-cli --locked

# 3. Drive the harness.
./scripts/run_phase0_harness.sh
```
