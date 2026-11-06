# Cycle 0008 Insight

## Cohort outcome

All three variants built on cycle 0007's v002 (ghost hits go straight to target Front with counter 1). Each one targeted the small-size capacity limit from a different side. v001 gave ghost-admitted keys a one-time exemption from the tail-scan victim choice (protecting credited keys). v002 allowed the bypass only when the target tail window had a free or counter-0 slot (gating on contention). v003 sent counter-0 cascade victims to the ghost instead of the stash (where displaced residents go). v002 was the Winner. It gave the largest small-size hm_0 gain, which nobody predicted, and a small web_0 gain at 16384. It gave back part of the parent's large-size web_0 gain.

## Patterns

- The small-size limit comes from who gets displaced, not from how long credited keys survive. Protecting the newcomer (v001) only moved eviction onto proven residents and hurt every cell, the large size most. Refusing the bypass when every candidate is warm (v002) protected those residents and helped. Fast eviction of un-re-hit ghost arrivals is a useful filter, not a bug.
- Where the cascade victim goes does not matter. Sending it to the stash or to the ghost gave identical hit-rates (v003). Stash churn from cascades is not a lever.
- "No cold slot in the window" is a rough admission signal. It separates warm residents from one-time returners well in a contended small target. In a large web_0 target, tail windows can be all warm, so the gate wrongly blocks returners and sends them back to the stash, where they are lost again, as in c0006. The bypass/stash trade-off now depends on cache size and is not settled by any one rule.
- Two variants' insights could not check their overlay diffs, so their mechanisms are inferred. In particular, v001's broad regression might partly come from a reduced counter cap rather than from the exemption itself.
- Per-op-time and metadata-size are still unmeasured, now for eight cycles. v003's only remaining argument was a cost saving, and it cannot be judged.

## Open questions

- Would a relative gate keep v002's small-size protection without blocking valuable large-size returners? One option: bypass when the returner's credit is at least the window's minimum counter, not only when a counter-0 slot exists.
- On a refused bypass, does keeping the ghost entry (letting a second return retry) actually work, or does it just delay the loss?
- Is large-size hm_0 now flat for structural reasons, with no remaining lever in the ghost/bypass family?

## Continuation

Make cycle 0008 v002 (contention-gated ghost bypass) the new parent. Next, refine the gate condition, for example by comparing the returner's credit with the coldest window counter, to win back large-size web_0 while keeping the small-size gains. Drop protecting credited keys inside the victim scan, and drop changes to where cascade victims are sent. Variant insights should confirm the overlay landed as specified, and per-op-time and metadata-size still need to be measured.
