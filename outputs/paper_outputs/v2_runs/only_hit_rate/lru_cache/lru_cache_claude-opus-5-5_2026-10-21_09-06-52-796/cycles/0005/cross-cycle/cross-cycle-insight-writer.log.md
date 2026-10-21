# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:36:09+00:00

## timeline

### 09:36:27 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Admission filtering is the lever that works.** S3-FIFO's gain over LRU (cycle 1) came from controlling which keys enter `main`, not from how recency is tracked. Every later winner refined admission further.\n- **Shrinking the ghost window keeps paying, with smaller gains each time.** Each of four halvings in a row (cycles 3–5, from near capacity down to an eighth) improved hit rate, and each gained less than the one before. The ghost is now about the same size as `small`.\n- **A short ghost makes ghost hits a trustworthy reuse signal.** With a long window, ghost hits misled admission on long loops. Once the window was short, the direct ghost-hit path into `main` became reliable, and adding filters to it gained little or lost hit rate.\n- **The fixed ~10% probation split holds up.** Moving away from it, whether adaptively (cycle 2) or by enlarging LIRS's probation buffer (cycle 3), cost hit rate.\n- **Representation-only changes leave hit rate alone, as expected.** The CLOCK-style links (cycle 1) and the fingerprint ring at the same window size (cycle 5) both kept hit rate level. They are safe bases for later policy work, though their cost benefits are still unverified.\n\n## Recurring failures\n\n- **Measurement of `metadata-size` and `per-op-time` has been missing for five cycles running.** Every memory or speed claim is still untested: the narrow links, the hasher, the fingerprint ring, the smaller ghosts, and the costs of the sketch and LIRS. Fixing this now blocks judging most hypotheses.\n- **Loosening admission costs hit rate.** Longer ghost windows, ARC-style adaptive `small` sizing, and other fast paths that let keys into `main` without proven reuse all regressed (cycle 2).\n- **Over-strict admission also fails.** Two-hit promotion bars and forcing ghost hits back through `small` were flat or regressed (cycles 3–4). Keys whose reuse gap is a little longer than the probation queue get evicted.\n- **Gates on only one admission route leak.** A stricter `small` bar is bypassed by the ghost path, and ghost-only gates are bypassed via `small`. Gates that compared against `main.front()` instead of the real eviction victim were also flawed.\n- **Frequency sketches gain almost nothing on this design.** A count-min gate on the ghost path, and then full TinyLFU against the real victim with the leak fixed (cycles 4–5), each gained only marginally. The short ghost plus the small-queue filter already rejects most cold keys, and the sketch's cost is unmeasured.\n- **Changing recency tracking alone doesn't raise hit rate.** CLOCK roughly matched LRU (cycle 1). Exact LRU order buys nothing here.\n\n## Open questions\n\n- Does the ghost-size curve turn over below `small`'s size (for example a sixteenth of capacity)? Or is the ghost-into-`main` fast path dispensable altogether?\n- Do the ghost-size gains hold at another cache size or on another trace? So far all evidence comes from workload-a at a single size.\n- Do the fingerprint ring, Fx hasher, and `u32` indices actually reduce `metadata-size` and `per-op-time`? Can one hash per key serve both the fingerprint and the lookup?\n- Was cycle 4 v002's regression caused by the routing change or by the two-hit bar? The two were never tested separately.\n- Where can further hit-rate gains come from now that admission filtering looks close to saturated?\n- Does CLOCK's victim sweep cause tail-latency spikes on `put`?\n\n## Per-policy notes\n\n- **S3-FIFO:** the line that keeps winning. Current best: fixed ~10% `small`, one-hit promotion, ghost at an eighth of capacity with a direct ghost-hit path into `main`. Keep the sequence-number ghost fix as a correctness change, even though its effect on hit rate is negligible.\n- **CLOCK / LRU approximations:** hit rate about the same as LRU. Worth pursuing only if measurement shows a memory or latency win.\n- **Strict LRU + representation tweaks (Fx hash, `u32` links, MRU fast path):** hit rate unchanged. Candidate for grafting onto S3-FIFO once cost can be measured. Not yet attempted.\n- **LIRS:** beats LRU but trails S3-FIFO, with a heavier `Node` and a linear ghost-bound walk. Enlarging its HIR queue hurt. Set aside until costs can be measured and the walk is made O(1).\n- **ARC-style adaptivity:** drifted away from the good split and lost hit rate. Dropped.\n- **TinyLFU / count-min admission:** marginal gains at best, with unmeasured cost. Dropped unless measurement shows it is nearly free.\n"
}
```

### 09:36:32 thinking



### 09:36:32 chat

I've written `cross-cycle-insights.md` using the four required sections, built only from the ledger for cycles 1–5.

**What has held up across cycles:**
- **Admission filtering is the main lever.** The S3-FIFO line has won every cycle.
- **Shrinking the ghost keeps helping, by less each time.** Four halvings in a row raised hit rate, and the ghost is now about the same size as the probation queue.
- **Keep the probation queue at a fixed ~10%.** Moving away from that split cost hit rate.

**What keeps failing:**
- **Loosening admission costs hit rate,** and so does making it too strict.
- **Gates on only one route into the main queue leak** through the other route.
- **Frequency sketches add almost nothing** on top of the current design.

**The biggest gap:** for five cycles running, memory size and per-operation time have never been measured. So every memory or speed claim made so far is still untested.

**Open threads:** whether hit rate finally turns over once the ghost is smaller than the probation queue, and whether the ghost-size gains hold at another cache size or on another trace.

The per-policy notes cover S3-FIFO, CLOCK, strict LRU with the representation tweaks, LIRS, ARC-style adaptive sizing and TinyLFU.


---

- finished: 2026-10-21T09:36:32+00:00
- duration: 23.35s
- status: ok
- input_tokens: 4
- output_tokens: 2224
- cache_read_tokens: 37949
- cache_write_tokens: 11068
- cost_usd: 0.14063
