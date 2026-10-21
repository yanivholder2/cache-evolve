# Cycle 0007 Insight

## Cohort outcome

Variants moved off insertion tuning and probed other policy paths: a geometric halving jump for hit promotion (v001), skipping reorder on put-of-existing-key with a single-scan restructure (v002), and a stateless hash-parity choice between the last two slots for eviction (v003). All three were Winners, but none shows a real gain in its own insight.

## Patterns

- Winner status is again disconnected from the variants' evidence. v001 and v003 report hit-rate losses, v003 also reports slower put and get, and v002 reports no median put gain and a worse put tail. This is the fifth straight cycle of this pattern.
- Gradual one-step transpose promotion acts as a frequency filter. Faster promotion removes that filter and hurts hit rate on the scan-heavy trace, so the parent's promotion looks already tuned.
- Stateless randomization carries no reuse signal. Hash-bit victim choice only adds noise and an unpredictable branch. Scan-resistance information must come from recency or frequency, not key-hash bits.
- Put-hit reordering is not a meaningful cost, since the workload's puts are mostly misses. Restructuring put-hit control flow adds variance and no saving.
- Get latency still has no real gain, now seven cycles. Small get shifts in v001, v002 and v003 are attributed to noise or layout.
- No variant changed metadata size.

## Open questions

- Are any of the three Winner statuses real, or are they noise in tail latency?
- Is there a middle ground on promotion, such as a jump only after a second hit, that keeps the frequency filter?
- Can any signal that reflects reuse fit in the 40-byte metadata, or is the budget the real limit?
- Where does get time go? Harness or dispatch overhead is untested.
- Do any results hold on a second workload?

## Continuation

Drop geometric or aggressive promotion, stateless hash-based eviction, and put-hit control-flow tweaks. Keep swap-based len/4 insertion with transpose-by-one promotion as the baseline. Aim the next cycle at the put-miss and insertion path or at a small real reuse signal under the metadata budget. Add a noise check and a second workload, and discount Winners whose own insight shows a loss.
