// cf-invariants-anchor-suggest — ranked invariant suggestion.
//
// Phase 0 ships ONE class — balance_conservation — but the
// `InvariantClass` trait + `ClassRegistry` are extensible: Phase 1+
// will register monotonicity, access-control, and oracle-freshness
// behind the same API.
//
// The suggester is pure heuristic in Phase 0 (no AI call on the
// default path). The cf-invariants Cairo reference handles the AI
// transport — we reuse that shape in Phase 1 by adding an
// `ai_propose` adapter that wraps these candidates with
// `InvariantSource::AiSuggested {…}`. Phase 0 emits
// `InvariantSource::Heuristic {…}` so the disclosure machinery is
// exercised end-to-end without an AI call.

use cf_invariants_anchor_core::{
    ContractSurface, EmitHints, InvariantCandidate, InvariantSource,
};

pub const SUGGESTER_VERSION: &str = "0.1.0";

/// One pluggable invariant class.
pub trait InvariantClass {
    /// Stable identifier — what `InvariantCandidate.class` carries.
    fn class_id(&self) -> &'static str;

    /// Propose ranked candidates for the given surface.
    fn propose(&self, surface: &ContractSurface) -> Vec<InvariantCandidate>;
}

/// Composable list of registered classes.
pub struct ClassRegistry {
    classes: Vec<Box<dyn InvariantClass>>,
}

impl ClassRegistry {
    pub fn empty() -> Self {
        Self { classes: vec![] }
    }

    /// Phase 0 default: balance_conservation only.
    pub fn phase0() -> Self {
        let mut r = Self::empty();
        r.register(Box::new(BalanceConservation));
        r
    }

    pub fn register(&mut self, class: Box<dyn InvariantClass>) {
        self.classes.push(class);
    }

    /// Walk every registered class and concatenate proposals,
    /// re-sorted by `rank` descending.
    pub fn propose_all(&self, surface: &ContractSurface) -> Vec<InvariantCandidate> {
        let mut out = Vec::new();
        for c in &self.classes {
            out.extend(c.propose(surface));
        }
        out.sort_by(|a, b| {
            b.rank
                .partial_cmp(&a.rank)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }
}

/// Phase-0 class: balance_conservation.
///
/// For each scalar uint field on an account referenced by ≥1 movement
/// instruction (deposit/withdraw/mint/burn/transfer/claim/redeem/...),
/// emit a candidate that asserts the on-chain value equals a
/// fixture-side ledger walked through the same actions.
pub struct BalanceConservation;

impl InvariantClass for BalanceConservation {
    fn class_id(&self) -> &'static str {
        "balance_conservation"
    }

    fn propose(&self, surface: &ContractSurface) -> Vec<InvariantCandidate> {
        let movement: Vec<&str> = surface
            .movement_instructions()
            .into_iter()
            .map(|i| i.name.as_str())
            .collect();

        if movement.is_empty() {
            return vec![];
        }

        let mut candidates = Vec::new();
        for f in &surface.balance_fields {
            // Strong-signal field names get rank > 0.85; everything
            // else gets 0.5 (still surfaced, ordered lower).
            let strong = matches!(
                f.field.as_str(),
                "amount" | "balance" | "total_amount" | "total_assets" | "supply"
            );
            let rank = if strong { 0.92 } else { 0.55 };

            let invariant_name = format!(
                "invariant_{}_conservation",
                f.field.to_ascii_lowercase()
            );
            let summary = format!(
                "{}.{} == fixture-tracked sum of deposits − sum of withdrawals",
                f.account, f.field
            );
            let rationale = format!(
                "Detected balance-bearing field `{}.{}: {}` on an account \
                 mutated by movement-class instructions ({}). A correct \
                 implementation keeps this field in lock-step with the \
                 net amount transferred in via these instructions; any \
                 drift is a balance-conservation violation. The fixture \
                 walks `expected_{}` through `action_deposit`/`action_withdraw` \
                 and asserts equality after every action.",
                f.account,
                f.field,
                f.ty,
                movement.join(", "),
                f.field
            );
            let emit_hints = EmitHints {
                account_type: f.account.clone(),
                field: f.field.clone(),
                expected_expression: format!("fixture.expected_{}", f.field),
                action_names: movement.iter().map(|s| s.to_string()).collect(),
            };

            candidates.push(InvariantCandidate {
                name: invariant_name,
                summary,
                class: self.class_id().to_string(),
                rank,
                rationale,
                emit_hints,
                source: InvariantSource::Heuristic {
                    suggester_version: SUGGESTER_VERSION.to_string(),
                },
            });
        }

        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cf_invariants_anchor_core::{BalanceField, ContractSurface, Instruction};

    fn vault_surface() -> ContractSurface {
        ContractSurface {
            program_id: "Va111tRef1111111111111111111111111111111111".into(),
            program_name: "vault_ref".into(),
            instructions: vec![
                Instruction { name: "initialize".into(), args: vec![], accounts: vec![] },
                Instruction { name: "deposit".into(),    args: vec!["amount".into()], accounts: vec![] },
                Instruction { name: "withdraw".into(),   args: vec!["amount".into()], accounts: vec![] },
            ],
            balance_fields: vec![BalanceField {
                account: "Vault".into(),
                field: "amount".into(),
                ty: "u64".into(),
            }],
        }
    }

    #[test]
    fn phase0_proposes_amount_conservation() {
        let surface = vault_surface();
        let registry = ClassRegistry::phase0();
        let candidates = registry.propose_all(&surface);
        assert_eq!(candidates.len(), 1);
        let c = &candidates[0];
        assert_eq!(c.class, "balance_conservation");
        assert_eq!(c.name, "invariant_amount_conservation");
        assert!(c.rank > 0.9);
        assert!(matches!(c.source, InvariantSource::Heuristic { .. }));
        assert_eq!(c.emit_hints.account_type, "Vault");
        assert_eq!(c.emit_hints.field, "amount");
        assert!(c.emit_hints.action_names.contains(&"deposit".to_string()));
        assert!(c.emit_hints.action_names.contains(&"withdraw".to_string()));
    }

    #[test]
    fn no_movement_yields_no_candidates() {
        let mut surface = vault_surface();
        surface.instructions = vec![Instruction {
            name: "initialize".into(),
            args: vec![],
            accounts: vec![],
        }];
        let candidates = ClassRegistry::phase0().propose_all(&surface);
        assert!(candidates.is_empty());
    }

    #[test]
    fn ranking_sorts_strong_signal_first() {
        let mut surface = vault_surface();
        surface.balance_fields.push(BalanceField {
            account: "Vault".into(),
            field: "lifetime_counter".into(),
            ty: "u64".into(),
        });
        let candidates = ClassRegistry::phase0().propose_all(&surface);
        assert_eq!(candidates.len(), 2);
        // Strong-signal "amount" outranks the weak-signal counter.
        assert_eq!(candidates[0].name, "invariant_amount_conservation");
        assert!(candidates[0].rank > candidates[1].rank);
    }
}
