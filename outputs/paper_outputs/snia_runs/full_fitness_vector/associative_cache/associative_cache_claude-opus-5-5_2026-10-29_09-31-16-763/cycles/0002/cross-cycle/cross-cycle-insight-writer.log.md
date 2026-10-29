# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-29T09:51:17+00:00

## timeline

### 09:51:34 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Scan-resistant admission is the only lever that has won on hit rate twice. Both Winners (cycle 0001 LIP and CLOCK, cycle 0002 BIP-on-LIP) make new items earn protection before they are shielded from eviction. The gains are largest on the wide buckets.\n- Bounding worst-case work helps the put tail every time. Examples so far are the 16-slot sweep cap, clear-on-insert and the dense absence scan.\n- Metadata-size has stayed flat. Reusing existing per-item fields such as `lfu_counter` and changing only the insertion position cost nothing on that axis. Note that this metric cannot see side arrays held inside the bucket.\n\n## Recurring failures\n\n- Every change to the eviction or insert mechanism has made the median put slower, in both cycles. Mid-bucket memmoves, per-item sweeps, window scans and writes to two allocations all cost more per op than they save. \"A memmove is a memmove\" did not hold.\n- Changing the hit path does not move get p50. It barely moved even when the hit path became a single store. The linear key lookup across the wide bucket dominates the get cost.\n- Splitting hot data across allocations makes get p50 worse. The separate dense key Vec added a second memory stream to every hit.\n- Trimming policy dispatch (the per-op `name()` string compare) gave no visible speed gain. The mechanism cost hides it.\n- Returns from tweaking the replacement rule are shrinking. After cycle 0001 the gains are small, and each one trades hit rate between workloads (BIP gave back scan resistance on web_0).\n\n## Open questions\n\n- Does the hit-rate harness run the overlaid bucket `evict_item` / ClassicLRU paths? The frequency-window variant matched the baseline exactly and has not been re-checked.\n- Can inline per-slot fingerprints or tags in the same cache line, or SIMD-friendly blocks, cut get p50 where a separate key Vec did not?\n- Is there a cheaper insert-at-depth for LIP-style policies that avoids the mid-bucket memmove?\n- Should the insertion depth or BIP period scale with bucket width, or use a reuse-aware admission signal, to keep the small-bucket gains without losing web_0?\n- Do LIP-style depth insertion and a cleared CLOCK/GCLOCK reference bit add up, or do they give the same scan protection twice?\n- Can any change to bound the tail avoid adding cost at p50? So far, tail and median have moved in opposite directions.\n\n## Per-policy notes\n\n- LIP / quarter-depth insert: a Winner in cycle 0001 and the base of the cycle 0002 Winner. Its hit rate is strong, but its put latency is consistently poor because of the mid-bucket memmove.\n- BIP (bimodal on LIP): only a marginal Winner, carried by one small-bucket cell. It loses on web_0 and does not fix put latency.\n- CLOCK / GCLOCK: a Winner in cycle 0001. Going to 2 bits added only a little hit rate, though it did so at every point. With the bounded 16-slot sweep and clear-on-insert it is the preferred base mechanism for the put tail, although it makes the typical put slower. Unbounded sweeps hurt the tail.\n- Frequency-aware tail window: has no measurable effect yet, and the harness coverage is unconfirmed. Do not judge it until that is checked.\n- Lookup layout (dense key mirror): improved the put tail but hurt get p50 through poor locality. Drop the separate-Vec approach.\n"
}
```

### 09:51:41 chat

I wrote `cross-cycle-insights.md` from the two-cycle ledger, using the four required sections in order. The main patterns across both cycles:

- **What keeps working:**
  - **Hit rate:** making new items earn their place before they're protected from eviction (LIP, CLOCK, BIP) is the only change that has won twice. The gains are biggest with the wide buckets.
  - **Put tail:** capping worst-case work (the 16-slot sweep cap, clear-on-insert) helps every time.
- **What keeps failing:**
  - **Put median:** every change to the insert or eviction mechanism has made the median put slower.
  - **Get median:** it hasn't moved, because the linear key lookup dominates the cost. Keeping keys in a separate array made it worse.
  - **Smaller tweaks:** removing the per-op policy name check gave no visible gain, and each new replacement-rule tweak now gains less and trades hit rate between workloads.
- **Unresolved:**
  - Whether the hit-rate harness actually runs the overlaid eviction path; the frequency-window variant matched the baseline exactly.
  - Whether keys and hit-time data stored together in one cache line can cut get latency.
  - Whether there's a cheaper way to insert mid-bucket for LIP.
  - Whether insertion depth should scale with bucket width.
  - Whether LIP-style insertion and a cleared reference bit add up or give the same scan protection twice.
  - Whether any fix for the put tail can avoid slowing the typical put.
- **Per-policy notes:** these cover LIP, BIP, CLOCK/GCLOCK, the frequency-aware window and the separate key array. The CLOCK variant with the 16-slot sweep cap and clear-on-insert is marked as the preferred base for the put tail.


---

- finished: 2026-10-29T09:51:42+00:00
- duration: 24.99s
- status: ok
- input_tokens: 4
- output_tokens: 1947
- cache_read_tokens: 36117
- cache_write_tokens: 7526
- cost_usd: 0.106387
