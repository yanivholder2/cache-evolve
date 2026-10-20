# Variant v002 Insight

## Helped

- per-op-time: Deleting the unconditional `target_heat.clear` from `put` lowered put-p50 — the median insert no longer pays a side-table touch, confirming the hypothesis's premise that the per-insert probe was real work on the hot path.

## Hurt

- per-op-time: The summary metric (get-p50) ticked up and both tails rose (put-p99, get-p99). The clear didn't disappear — it moved onto `demote_from_normal_bucket`'s target→stash and cascade branches, and that relocated work plus added branching shows up as tail variance rather than a clean win.
- hit-rate: Effectively flat (down only at the noise floor), as designed — a stash key still carries heat 0 in both versions, so gate decisions stayed unchanged.

## Takeaway

Moving the heat clear from every `put` to the rarer demotion site does shave median put latency, but only by trading it for wider tails: the byte-identical gate behavior held, yet the relocated clear and its new branches made the per-op-time summary regress. Removing a probe from the median path is not free when the path it lands on adds branching the tail then exposes.
