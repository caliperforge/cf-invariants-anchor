// cf-invariants-anchor-report — scorecard markdown + JSON renderers.
//
// Disclosure rule (enforced at the renderer): the AI-disclosure
// banner is emitted whenever `Scorecard.ai_suggestions_included > 0`.
// This mirrors the cf-invariants (Cairo) shape so the two scorecards
// are visually interchangeable across our build-to-win surface.

use cf_invariants_anchor_core::Scorecard;

pub fn render_markdown(s: &Scorecard) -> String {
    let mut out = String::new();
    out.push_str("# cf-invariants-anchor scorecard\n\n");

    if s.ai_suggestions_included > 0 {
        out.push_str(
"> **AI involvement, disclosed at the point of use.**\n\
> AI-suggested invariants are part of this run. They are UNVERIFIED until reviewed by the contract author. See docs/ai-disclosure.md.\n\n",
        );
    }

    out.push_str("## Summary\n\n");
    out.push_str(&format!("- Invariants total: **{}**\n", s.invariants_total));
    out.push_str(&format!(
        "- Invariants violated: **{}**\n",
        s.invariants_violated
    ));
    out.push_str(&format!(
        "- AI-suggested invariants in this run: **{}**\n",
        s.ai_suggestions_included
    ));
    out.push_str(&format!("- Runtime: **{} ms**\n", s.runtime_ms));
    out.push_str(&format!(
        "- Crucible version: `{}`\n",
        s.crucible_version
    ));
    out.push('\n');

    if !s.counterexamples.is_empty() {
        out.push_str("## Counterexamples\n\n");
        for cx in &s.counterexamples {
            out.push_str(&format!("### `{}`\n\n", cx.invariant_name));
            out.push_str(&format!("- Seed: `{}`\n", cx.seed));
            out.push_str("- Failing sequence (not shrunk; shrinking is v2):\n\n");
            out.push_str("```\n");
            out.push_str(&cx.failing_sequence);
            if !cx.failing_sequence.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```\n\n");
        }
    }

    out.push_str(
        "---\n\ncf-invariants-anchor — Apache-2.0. Operated by Michael Moffett — \
         michael@caliperforge.com — team@caliperforge.com.\n",
    );

    out
}

pub fn render_json(s: &Scorecard) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cf_invariants_anchor_core::{Counterexample, Scorecard};

    fn sc(ai: usize, violated: usize) -> Scorecard {
        Scorecard {
            invariants_total: 1,
            invariants_violated: violated,
            ai_suggestions_included: ai,
            runtime_ms: 1234,
            crucible_version: "0.2.0".into(),
            counterexamples: if violated == 0 {
                vec![]
            } else {
                vec![Counterexample {
                    invariant_name: "invariant_amount_conservation".into(),
                    seed: "0xdeadbeef".into(),
                    failing_sequence:
                        "INVARIANT VIOLATED invariant_amount_conservation\n\
                         on_chain: 5\nexpected: 7\ndelta: -2\n\
                         sequence: [action_deposit(100), action_withdraw(50), action_withdraw(45)]"
                            .into(),
                    witness: serde_json::json!({"delta": -2}),
                }]
            },
        }
    }

    #[test]
    fn ai_disclosure_banner_when_ai_count_positive() {
        let s = sc(1, 1);
        let md = render_markdown(&s);
        assert!(md.contains("AI involvement, disclosed at the point of use."));
    }

    #[test]
    fn ai_disclosure_omitted_when_no_ai_suggestions() {
        let s = sc(0, 1);
        let md = render_markdown(&s);
        assert!(!md.contains("AI involvement, disclosed at the point of use."));
    }

    #[test]
    fn counterexample_block_renders() {
        let s = sc(1, 1);
        let md = render_markdown(&s);
        assert!(md.contains("invariant_amount_conservation"));
        assert!(md.contains("INVARIANT VIOLATED"));
    }
}
