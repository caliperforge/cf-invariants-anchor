# AI disclosure — cf-invariants-anchor

cf-invariants-anchor is built with AI assistance. This document is the
in-repo detail on what the AI module does, where the audit log lives,
and how to disable AI involvement.

## What is AI-suggested today (Phase 0)

**Nothing in the runtime path.** Phase 0 ships the *heuristic*
suggester only — pure structural reasoning over the parsed
ContractSurface. Every `InvariantCandidate` emitted by the Phase-0
build carries:

```
source: InvariantSource::Heuristic { suggester_version: "0.1.0" }
```

The scorecard renderer's AI-disclosure banner is therefore dormant
on Phase-0 reference runs. The banner's behaviour is still exercised
by unit tests (`ai_disclosure_banner_when_ai_count_positive`,
`ai_disclosure_omitted_when_no_ai_suggestions`,
`ai_source_renders_ai_disclosure_banner`).

## What will be AI-suggested in Phase 1

Phase 1 will add an Anthropic Claude Sonnet adapter that calls the
Anthropic API with a versioned prompt (`prompts/invariant_suggestion_v1.txt`)
and the parsed ContractSurface, and wraps the returned candidates with:

```
source: InvariantSource::AiSuggested {
    model: "claude-sonnet-4-...",
    prompt_version: "invariant_suggestion_v1",
    timestamp_utc: "<ISO-8601>",
}
```

When that lands:

- Every AI-suggested candidate file carries an
  `[AI-SUGGESTED, UNVERIFIED]` header naming model, prompt version,
  timestamp.
- The scorecard markdown emits an AI-disclosure banner whenever
  `Scorecard.ai_suggestions_included > 0`.
- Each Anthropic API call writes an audit record to
  `.cf-invariants-anchor/ai-log/<timestamp>.json`
  (model, prompt version, token counts, response SHA-256 hash) —
  the directory is `.gitignore`d by default.
- The CI test suite uses `MockTransport`; live calls only fire when
  both `CF_INVARIANTS_ANCHOR_AI_LIVE=1` and `ANTHROPIC_API_KEY` are set.

The shape mirrors the cf-invariants (Cairo) AI path one-for-one so
operators using both have a single mental model.

## AI assistance in **authoring** this codebase

This codebase was authored with AI assistance (Claude). Every file is
reviewed and accepted by the operator. The AI-authoring assistance is
distinct from the AI **runtime suggestion** path described above —
the former is one-time at authorship; the latter is a deliberate,
disclosed, audit-logged runtime feature.

## How to disable AI involvement

- **Default build**: heuristic suggester only. No outbound calls, no API keys required.
- **Phase 1 AI path**: requires explicit opt-in via `CF_INVARIANTS_ANCHOR_AI_LIVE=1` + `ANTHROPIC_API_KEY`. Unset either and the live transport short-circuits to `MockTransport`.

## Cost budget

Phase 1 will inherit cf-invariants's per-call cap (**$0.05** per call) and three-contract demo cap (**$0.25**), asserted by a `cost_budget` regression test in `cf-invariants-anchor-ai`.

## Full policy

[caliperforge.com/ai-disclosure](https://caliperforge.com/ai-disclosure).
