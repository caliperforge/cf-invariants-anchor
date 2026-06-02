# cf-invariants-anchor scorecard

## Summary

- Invariants total: **1**
- Invariants violated: **1**
- AI-suggested invariants in this run: **0**
- Runtime: **2100 ms**
- Crucible version: `0.2.0`

## Counterexamples

### `invariant_amount_conservation`

- Seed: `0x76617546`
- Failing sequence (not shrunk; shrinking is v2):

```
INVARIANT VIOLATED invariant_amount_conservation
on_chain: 500
expected: 499
delta: +1
sequence: [action_deposit(500), action_withdraw(1)]
```

---

cf-invariants-anchor — Apache-2.0. Operated by Michael Moffett — michael@caliperforge.com — team@caliperforge.com.
