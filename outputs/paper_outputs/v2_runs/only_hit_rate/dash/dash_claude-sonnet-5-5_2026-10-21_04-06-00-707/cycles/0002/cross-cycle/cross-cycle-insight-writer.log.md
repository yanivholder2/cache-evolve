# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:16:50+00:00

## timeline

### 04:16:57 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changing where a promoted item lands (MRU placement on hit promotion, cycle 1) is the only lever that has improved hit-rate so far. It beat variants that moved more items.\n- Stacking further placement or eligibility tweaks on that winner has not hurt it. Cycle 2 variants matched the parent exactly, so the MRU baseline is stable.\n\n## Recurring failures\n\n- Approaches that add item movement, such as bidirectional swaps, looser promotion eligibility, or extra promotions, give marginal or negative returns. The likely cause is demotion cascades toward stash and eviction.\n- The placement-position family (insert, demotion, eligibility) produced no hit-rate change in cycle 2. On workload-a at this cache size it looks exhausted, or its overlays were dead edits.\n- Cold-end demotion, justified by the cascade-eviction theory, got no support.\n\n## Open questions\n\n- Do the overlays change the executed code path at all? Equal hit-rate across all cycle 2 variants could mean no-ops.\n- Is workload-a or the cache size insensitive to placement? Other traces or sizes might separate the variants.\n- Do the variants differ on per-op-time or metadata size? Only hit-rate has been measured, so there are no cost claims and ties cannot be broken.\n- Which untested levers matter: eviction victim selection, bucket and probing structure, or admission?\n\n## Per-policy notes\n\n- DashSegment (promotion in `get` and `promotion_get`): MRU insertion on hit promotion is the incumbent, and the lineage branches from cycle/0001/v001. Eligibility, insert-path and demotion-path variants add nothing on top of it on workload-a.\n- Add per-op-time and metadata-size to the fitness vector, and test on a second workload, before trusting any further placement result.\n"
}
```

### 04:17:01 chat

I wrote `cross-cycle-insights.md` in the output dir, with the four fixed sections in order. It is built only from the two cycles in the ledger digest.

- **Recurring wins:** Changing where a promoted item lands (MRU placement on hit promotion, cycle 1) is the only change that has improved hit-rate. Cycle 2 variants only matched it, so it is stable.
- **Recurring failures:** Changes that move more items gave little or lost, probably through demotion cascades. The placement family (insert, demotion, eligibility) added nothing in cycle 2, and cold-end demotion got no support.
- **Open questions:**
  - Do the cycle 2 overlays change the executed code at all? They may be no-ops.
  - Is workload-a at this cache size insensitive to placement?
  - Do the variants differ on per-op-time or metadata size, which have never been measured?
  - Do eviction victim selection, bucket and probing structure, or admission matter?
- **Per-policy notes:** The only entry is DashSegment, where MRU on hit promotion is the incumbent and later work branches from cycle/0001/v001. I also noted that per-op-time and metadata size should be added to the fitness vector, and a second workload tested.

I used no numbers, only qualitative statements.


---

- finished: 2026-10-21T04:17:01+00:00
- duration: 10.96s
- status: ok
- input_tokens: 4
- output_tokens: 1166
- cache_read_tokens: 34984
- cache_write_tokens: 6268
- cost_usd: 0.040507
