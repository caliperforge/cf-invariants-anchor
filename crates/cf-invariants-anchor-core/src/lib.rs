// cf-invariants-anchor-core — shared types.
//
// `ContractSurface` is the canonical mid-form between Anchor IDL ingest
// and the invariant suggester. `InvariantCandidate` carries a
// `source` field that distinguishes manual / AI-suggested / heuristic
// candidates — the renderer keys disclosure on this.

use serde::{Deserialize, Serialize};

/// Token-like field on a stored account that participates in
/// balance-conservation reasoning. The Phase-0 suggester ranks these.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BalanceField {
    /// Account type that owns the field (e.g. `Vault`).
    pub account: String,
    /// Field name on the account (e.g. `amount`).
    pub field: String,
    /// Solidity-style scalar type (`u64`, `u128`, `i64`, ...).
    pub ty: String,
}

/// One Anchor program instruction (one entry in the IDL `instructions`
/// array).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Instruction {
    pub name: String,
    /// Argument names, ordered.
    pub args: Vec<String>,
    /// Account names referenced (writable + read-only).
    pub accounts: Vec<String>,
}

/// Canonical mid-form: what cf-invariants-anchor-idl produces and what
/// cf-invariants-anchor-suggest consumes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractSurface {
    pub program_id: String,
    pub program_name: String,
    pub instructions: Vec<Instruction>,
    /// All scalar balance-bearing fields across all account types.
    pub balance_fields: Vec<BalanceField>,
}

impl ContractSurface {
    /// Instructions that name-match the conservation movement pattern
    /// (deposit / withdraw / mint / burn / transfer / claim).
    ///
    /// Phase-0 heuristic — deliberately conservative. The Phase-1 path
    /// is structural (read the IR / account-mut-set), not lexical.
    pub fn movement_instructions(&self) -> Vec<&Instruction> {
        const MARKERS: &[&str] = &[
            "deposit", "withdraw", "mint", "burn", "transfer", "claim",
            "redeem", "stake", "unstake",
        ];
        self.instructions
            .iter()
            .filter(|ix| {
                let n = ix.name.to_ascii_lowercase();
                MARKERS.iter().any(|m| n.contains(m))
            })
            .collect()
    }
}

/// Provenance for an invariant. The scorecard renderer reads this to
/// decide whether to emit the AI-disclosure banner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum InvariantSource {
    /// Hand-authored by the user.
    Manual,
    /// Heuristic suggester (no AI call). Used in offline mode and tests.
    Heuristic {
        suggester_version: String,
    },
    /// AI-suggested. Banner-required on the scorecard.
    AiSuggested {
        model: String,
        prompt_version: String,
        timestamp_utc: String,
    },
}

impl InvariantSource {
    pub fn is_ai_suggested(&self) -> bool {
        matches!(self, InvariantSource::AiSuggested { .. })
    }
}

/// One candidate invariant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvariantCandidate {
    /// Stable identifier — used as the `#[invariant_test]` function name
    /// when emitted.
    pub name: String,
    /// One-line summary.
    pub summary: String,
    /// Class — `balance_conservation` in Phase 0; the ranking framework
    /// is extensible (see suggester ClassRegistry).
    pub class: String,
    /// Phase-0 ranking: higher = stronger candidate. Bounded \[0.0, 1.0\].
    pub rank: f32,
    /// One-paragraph rationale shown to the developer before they accept.
    pub rationale: String,
    /// The Crucible-side assertion plus the fixture-side bookkeeping
    /// required to evaluate it.
    pub emit_hints: EmitHints,
    pub source: InvariantSource,
}

/// Fields the emit crate needs to synthesise a Crucible `#[invariant_test]`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmitHints {
    /// Account type to read with `ctx.read_anchor_account::<T>(&pda)`.
    pub account_type: String,
    /// Field on the account to assert against (e.g. `amount`).
    pub field: String,
    /// Expression to compute the expected value at fixture-side. The
    /// emit crate substitutes this verbatim into the rendered fixture.
    /// For Phase-0 balance-conservation: `fixture.expected_amount`.
    pub expected_expression: String,
    /// Action names whose post-conditions update the expected expression.
    /// The emit crate generates an `action_*` arm per name.
    pub action_names: Vec<String>,
}

/// Scorecard envelope — mirrors cf-invariants (Cairo) shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scorecard {
    pub invariants_total: usize,
    pub invariants_violated: usize,
    pub ai_suggestions_included: usize,
    pub runtime_ms: u64,
    pub crucible_version: String,
    pub counterexamples: Vec<Counterexample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterexample {
    pub invariant_name: String,
    pub seed: String,
    pub failing_sequence: String,
    pub witness: serde_json::Value,
}
