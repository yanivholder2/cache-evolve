# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:57:53+00:00

## timeline

### 11:57:59 thinking



### 11:58:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Retention signals that separate re-referenced keys from one-shot inserts gained hit-rate on workload-a. This ran through frequency counters, the second-chance reference bit, and the multi-level 2-bit re-reference value (Cycles 0002–0004).\n- Storage-layout changes that keep retention intact can save metadata. The per-bucket reference mask kept hit-rate, but at a time cost (Cycle 0004).\n- Behavior-preserving rewrites of an existing mechanism can recover timing. A word-level scan over the per-bucket mask recovered put-side median time lost to the bit-by-bit clock walk (Cycle 0005).\n\n## Recurring failures\n\n- Shortcuts to the victim search keep failing. A rotating window, a cached minimum, a cached two-candidate pair, and a narrowed scan did not fix the put tail, and several cost hit-rate. The exact minimum search is doing real work (Cycles 0003, 0004, 0006, 0007).\n- The put-side tail has not moved. Aging rewrites, early exits, and scan narrowing were each tried as its fix, and none helped (Cycles 0004–0008).\n- Moving items within a bucket, through midpoint promotion or midpoint reinsertion, hurt hit-rate and added shifting work on the hot path (Cycle 0001).\n- Ghost memory did not lift hit-rate, whether as a short bucket-local ring or a longer one (Cycles 0001, 0002).\n- Stacking extra decay on the periodic halving erased frequency protection too quickly (Cycle 0003).\n- Fingerprint prefilters added get and put cost with no hit-rate change (Cycle 0002).\n- Doorkeeper admission gates are not a dependable lever. They gained hit-rate in one cycle, lost it in another, and barely moved it in a third, while adding put-side and metadata cost (Cycles 0006–0008).\n- Hash placement changes did not move hit-rate. Bucket spread is not the miss driver, and any timing effect is unconfirmed (Cycles 0007, 0008).\n- Moving the access counters into a separate array regressed get tail and put, without a retention gain (Cycle 0008).\n\n## Open questions\n\n- Where does the put tail actually come from on full-bucket puts? Shortcuts to both the scan and the aging have failed, so the cause is still unmeasured.\n- Do the frequency and re-reference gains hold on traces other than workload-a and at other cache sizes?\n- Are the timing movements real? Many get and put shifts across cycles are single-run, and some are unexplained by diffs. Repeated runs are needed before any timing Winner is kept.\n- Is the get-tail gain from skipping the saturated-counter store reproducible with the counter kept in the item (Cycles 0006, 0008)?\n- Can the exact global minimum be kept cheaply, for example by refreshing a candidate on counter updates?\n- The metadata-size probe did not register new per-bucket or per-slot state (Cycles 0001, 0002). Its readings should not be used to judge storage cost until it is fixed.\n\n## Per-policy notes\n\n- Frequency-based victim selection (access counters with periodic halving): the main hit-rate source on workload-a. It is costly on put because each eviction scans the full bucket. Do not add extra decay or per-eviction halving (Cycles 0002, 0003).\n- Second-chance reference bit: gained hit-rate and reduced put cost in one cycle, but its per-item metadata grew and the get path gained a write per hit. Attribution is not yet confirmed. Next steps are to cut state toward one bit and cut the get-side write (Cycle 0003).\n- Multi-level 2-bit re-reference value: gained hit-rate, but its aging rounds carry the put-tail cost. The aging rewrite did not reduce that cost (Cycles 0004, 0005).\n- Per-bucket reference mask: keeps hit-rate and saves metadata, but its get and put time cost has to be recovered. The word-level scan is the current put-side base (Cycles 0004, 0005).\n- Rotating-window or narrowed victim scan: reduces put latency but weakens frequency protection. Dropped as a put-side fix (Cycles 0004, 0006).\n- Doorkeeper admission gate: inconsistent across cycles. It needs reconciling before it is retried, and a looser second-sighting rule is the only variant still worth a test (Cycles 0006–0008).\n- Bucket-local ghost memory and midpoint placement: no hit-rate gain. Treat as closed unless a different mechanism is proposed (Cycles 0001, 0002).\n- Overlay code was unreadable in several cycles (0003–0008), so mechanism attributions across this ledger are inferred from diffs and variant insights and should be treated as provisional.\n"
}
```

### 11:58:08 chat

I wrote `cross-cycle-insights.md` in the output directory with the four required sections, built only from the ledger digest (Cycles 0001–0008).

The main points:

- **Recurring wins:** Retention signals that separate re-referenced keys from one-shot inserts (frequency counters, the second-chance bit, the 2-bit re-reference value) have gained hit-rate on workload-a. The word-level scan over the per-bucket mask recovered some put median time.
- **Recurring failures:** Shortcuts to the victim search have not fixed the put tail, and several cost hit-rate. Midpoint placement, ghost memory, fingerprint prefilters, and hash changes have not paid off. The doorkeeper gate has given inconsistent results across cycles.
- **Open questions:** The source of the put tail is still unmeasured. Timing effects are mostly single-run. The metadata-size probe does not count new per-bucket or per-slot state, so its readings should not be used for storage cost.
- **Per-policy notes:** Each policy has its status and next step. The file also flags that overlay code was unreadable in several cycles, so the mechanism attributions are provisional.

I did not read the previous version of the file, as the instructions require.


---

- finished: 2026-10-20T11:58:09+00:00
- duration: 15.62s
- status: ok
- input_tokens: 4
- output_tokens: 3230
- cache_read_tokens: 40745
- cache_write_tokens: 14117
- cost_usd: 0.004846
