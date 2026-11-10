# Variant v001 Insight

## Helped

- hit-rate (hm_0, both sizes): one reuse is now enough to graduate, so keys that come back a third time are already in main. In the baseline they took a detour through ghost and missed once more first, and hm_0 has enough of these short-reuse, few-access keys to come out ahead.
- hit-rate (summary): the gain on hm_0 is slightly bigger than the loss on web_0, so the aggregate inches up. The change is close to neutral overall, not a clear win.

## Hurt

- hit-rate (web_0, both sizes): many web_0 keys are hit exactly once while in small and never used again. Under the new rule they get promoted into main instead of being dropped to ghost. Each one forces an extra `evict_main` pass, which wears down the second-chance credit of entries that are actually hot. Resetting the graduate's freq to 0 doesn't stop this, because the cost is paid when the key is admitted, not when it is evicted later.

## Takeaway

Lowering the graduation threshold from 2 to 1 trades one saved miss per "accessed three or more times" key for main-queue pollution from "accessed exactly twice" keys. Which side wins depends on the trace: hm_0 gains and web_0 loses. Resetting freq on graduation doesn't neutralise the pollution. A follow-up should filter what gets into main (for example, use a threshold of 1 only when main has spare room, or scale it with the small queue's reuse rate) rather than change how graduates are treated after they arrive. Only hit-rate was measured for this variant. Metadata-size should be unchanged since the `Entry` layout didn't change, but per-op-time wasn't measured.
