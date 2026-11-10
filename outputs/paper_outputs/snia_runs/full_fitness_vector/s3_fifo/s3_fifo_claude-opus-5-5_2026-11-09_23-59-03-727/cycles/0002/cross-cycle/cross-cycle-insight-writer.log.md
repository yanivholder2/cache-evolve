# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-11-10T04:04:05+00:00

## timeline

### 04:04:22 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Structural changes that keep the policy fixed and improve complexity (slab plus hash indexes instead of linear scans) are the only per-op-time gains the diffs actually explain. They also leave hit rate exactly unchanged, so the policy can be evaluated separately from the implementation.\n- On hm_0, giving reused keys more main residency or history pays off reliably: main-victim ghosting, restoring the demotee ghost, and promoting after a single hit all helped there in both cycles.\n\n## Recurring failures\n\n- No single-axis capacity or history tweak has beaten baseline yet. Fixed small-queue ratios, ghosting main victims, adaptive small/main splits, split ghost regions and a looser graduation threshold each moved the trade-off between hm_0 and web_0 (especially web_0 at the large size) without a net gain.\n- Resizing or partitioning the ghost has not fixed web_0 at the large size. Restoring the demotee ghost in full did not rescue it, which rules out ghost-slot competition as the cause there.\n- Bundling two mechanisms into one variant (for example the threshold change together with the freq reset on promotion) leaves the result impossible to attribute. This has repeatedly blocked progress on finding web_0's mechanism.\n- Per-op-time changes that the diff can't explain have appeared in both cycles. They are environmental noise and deserve no timing credit.\n- The ARC-style adaptive split, as implemented, regressed. Its two signals were unbalanced and its ghost was not tied to main's live size.\n\n## Open questions\n\n- What actually drives web_0's loss at the large size? The current leading suspects are freq-0 main-victim ghost hits re-admitted straight into main, and the freq reset on promotion. Test each in isolation.\n- Does restricting main-victim ghosting to victims with a nonzero frequency, or re-admitting ghost hits with only partial credit, keep hm_0's gain without hurting web_0?\n- Can an adaptive split work with normalized signals, a ghost that tracks main's live size, and a smaller step at large capacity?\n- The metadata-per-entry metric misses heap growth from ghost keys and hash maps. Should heap bytes per resident key be tracked alongside it?\n- Timing comparisons need a baseline re-run on the same machine before anyone trusts them.\n\n## Per-policy notes\n\n- S3-FIFO (the only policy explored so far): the hm_0 and web_0 trade-off has been stable for two cycles, so treat it as the main thing to resolve rather than as noise. Ghost capacity behaves as a coupled, zero-sum variable. Future variants should build on the slab-plus-hash base and change one mechanism at a time.\n"
}
```

### 04:04:28 thinking



### 04:04:28 chat

I wrote `cross-cycle-insights.md` from the two-cycle ledger, using the four required sections in order.

- **Recurring wins:** Only structural speedups (slab plus hash indexes) explain per-op-time gains, and they leave hit rate unchanged. On hm_0, giving reused keys more main-queue residency or history has helped in both cycles.
- **Recurring failures:** No single-axis capacity or history tweak has beaten baseline. Each one shifts the hm_0 vs. web_0 trade-off (especially web_0 at the large cache size) without a net gain.
  - Resizing or splitting the ghost queue doesn't fix web_0 at the large size.
  - Testing two changes in one variant has made results impossible to attribute.
  - Unexplained timing changes are environmental noise.
  - The adaptive small/main split regressed.
- **Open questions:**
  - What causes web_0's loss: freq-0 keys re-admitted from the main-victim ghost straight into main, or the frequency reset on promotion? Each needs testing alone.
  - Should only main victims with a nonzero frequency be ghosted?
  - Can a rebalanced adaptive split work?
  - Should heap bytes per key be tracked, since the per-entry metadata metric misses them?
  - Timing needs a baseline re-run on the same machine.
- **Per-policy notes:** S3-FIFO is the only policy tried so far. Its hm_0 vs. web_0 trade-off has held for two cycles, so future variants should treat it as the main problem, build on the slab-plus-hash base, and change one mechanism at a time.


---

- finished: 2026-11-10T04:04:28+00:00
- duration: 23.17s
- status: ok
- input_tokens: 4
- output_tokens: 1589
- cache_read_tokens: 35440
- cache_write_tokens: 7781
- cost_usd: 0.101457
