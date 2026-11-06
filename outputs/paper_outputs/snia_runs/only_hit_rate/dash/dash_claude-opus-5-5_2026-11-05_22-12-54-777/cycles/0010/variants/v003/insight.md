# Variant v003 Insight

> Note: this insight is based on H03, both fitness vectors, the parent's insight, and v003's overlay `src/dash/mod.rs` and `src/dash/segments/dash_segment.rs`. I could not find manifest.toml or any eval log, so I do not know the `scan_flagged_inserts` count (it is logged in `Dash::drop`) or the per-op time. Only these files were readable: no directory listing was available.

## Helped

(none: no cell improved)

## Hurt

- hit-rate (web_0, both sizes): The detector fired, so web_0's keys are sequential at the table level. But it fired against the trace. Flagged keys enter their stash bucket at `Front` with counter 0, so the next insert into that bucket evicts them. When they are dropped, `put_into_stash` skips the ghost write. A sequential block that is re-read later therefore gets almost no stash time and also loses its ghost-hit readmission. The ghost-hit readmission is the path every earlier web_0 gain came through. The loss is about the same at both sizes. So this is not stash pressure, which the hypothesis expected to matter most at 16384. Re-referenced scan blocks are losing their second chance at any size.
- hit-rate (hm_0, both sizes): A smaller drop, as you would expect if hm_0 has fewer `+1` runs. It is still a drop, not flat. So with `SCAN_RUN_MIN = 3`, enough hm_0 inserts are flagged to cost hits. Some of them are probably short incidental runs from interleaved streams that the single global `last_key` happens to link together.

## Takeaway

In these MSR traces, sequentially inserted blocks are not one-shot pollution. Enough of them come back that both their stash time and their ghost entry matter. H03 took both away at once, so it removed the very mechanism (ghost readmission) that already handles truly one-hit keys well, since a ghost entry costs no resident slot. If scan resistance is tried again, separate the two levers. Test either `Front` insertion with ghosting kept, or normal `Back` insertion with ghost suppression only. Do not expect a sequential-run filter to be a web_0 lever outside the ghost path: on this evidence, web_0's sequential keys are among the ghost's beneficiaries.
