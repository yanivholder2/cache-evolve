# Variant v002 Insight

## Helped

- hit-rate: The Cold/Hot tag plus the two-pass sweep shields re-referenced slots — once a `get` promotes a Cold slot to Hot, pass 1 skips it entirely, so the working set survives intervening cold scans that flat CLOCK would have cleared on its single pass.
- hit-rate: The ghost ring of recently-evicted Cold keys recovers the "evicted just before re-reference" tail that the parent had no memory of; a ghost-hit `put` lands the key as Hot, so the very next sweep already protects it instead of treating it as a fresh cold scan victim.

## Hurt

## Takeaway

Layering an evicted-key memory and a second-touch promotion onto the parent's flat CLOCK ring is enough to claw back recency information that plain CLOCK throws away on lirs:multi1 — exactly the cap the parent's insight named. The win is mechanical, not statistical: pass 1's Cold-only scope makes Hot a real protection class, and the ghost ring closes the loop so a one-step-too-late re-reference is no longer punished as a brand-new cold key.
