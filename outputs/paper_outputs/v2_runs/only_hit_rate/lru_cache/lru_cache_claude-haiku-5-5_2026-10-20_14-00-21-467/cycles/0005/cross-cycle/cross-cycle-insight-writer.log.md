# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:24:37+00:00

## timeline

### 14:24:44 thinking



### 14:24:49 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction-neutral representation and hit-path changes (sentinel index, head-hit relink skip, pre-sized key index, hit-path lookup collapse) have repeatedly held hit-rate exactly equal to their parent. They are safe to carry forward, but no cycle has yet shown a measured size or per-op gain from them.\n- Changing where new keys enter the recency order (inserting at the LRU tail with periodic MRU promotion) is the only hit-rate gain in the ledger so far. It adds no new structure, which makes it the cheapest route to scan resistance seen so far. It has been observed once, on one trace.\n\n## Recurring failures\n\n- Changes that alter eviction order with extra structure have lowered hit-rate on workload-a at size 2048: the probation/protected split and the referenced-bit second-chance approximation both dipped below parent. Strict recency ordering appears to be doing useful work on this trace.\n- Ghost-based admission filtering has produced no measurable effect in two cycles (0002, 0003). It may never have engaged on this trace, so the null result is not yet informative.\n- Winner status is being assigned without a visible basis in the evidence. Variants with hit-rate equal to or below parent, and with no per-op or size data, have been named Winners in cycles 0003, 0004, and 0005. This selection-criterion problem recurs and undermines lineage decisions.\n- Size and per-op goals have never been scored. Every cycle's changes were motivated by metadata or time, yet none has recorded those dimensions. This gap spans all five cycles.\n- Several cycles lost variants to infrastructure failures (session limit, missing insight file). Those variants' strategies are untested, not refuted.\n\n## Open questions\n\n- Do the size and per-op costs or gains of any eviction-neutral change show up once measured? Until they do, the layout and hit-path lineages have no evidence on their stated goals.\n- Does the LRU-tail insertion gain hold on other traces and cache sizes, or is it specific to workload-a at size 2048? Its benefit should concentrate on scan-heavy traffic, which has not been tested.\n- Is the segmented split dead as written, or does its protected-cap and demotion rule carry headroom? The cycle 0004 hit-rate direction for that lineage is still unreported.\n- Which parent do the reported hit-rate deltas refer to? Lineage has been ambiguous in cycles 0002 and 0004, which blocks clean comparison.\n- Did the ghost admission path ever run on the trace? If not, which workload would exercise it?\n\n## Per-policy notes\n\n- Strict LRU (move-to-front): the strongest signal so far is that its ordering should be kept. Neutral layout and hit-path changes preserve it, while second-chance and segmented variants that loosen it lowered hit-rate.\n- LRU with insertion-position change: the only hit-rate gain seen. Treat as the lead candidate, pending generalisation checks.\n- Segmented (probation/protected) recency: lowered hit-rate in its one reported cycle. Its extra list pointer and demotion logic likely add per-op and size cost that has not been measured.\n- Second-chance / referenced-bit: lowered hit-rate and was still listed as a Winner. Do not extend without re-checking that selection.\n- Ghost-based admission filter: no measured effect so far; engagement on the trace is unverified. Keep only as a hit-rate bet if it can be shown to fire.\n"
}
```

### 14:24:52 chat

I wrote `cross-cycle-insights.md` to the cycle 0005 cross-cycle output directory, using the four required sections in order. I built it only from the inlined ledger digest and did not read the previous file.

The main points:

- **Recurring wins:** Eviction-neutral changes (sentinel index, head-hit relink skip, pre-sized index, hit-path lookup collapse) have held hit-rate equal to their parent. None has shown a measured size or per-op gain. The only hit-rate gain in the ledger is inserting new keys at the LRU tail, which I noted was seen once on one trace.
- **Recurring failures:** Extra structure that changes eviction order (probation/protected split, second-chance) lowered hit-rate. Ghost-based admission has shown no effect in two cycles and may never have run. Winners have been named without a visible basis in the evidence in cycles 0003–0005, and per-op time and metadata size have never been recorded.
- **Open questions:** Whether the LRU-tail gain generalises beyond workload-a at size 2048, whether the segmented split has headroom in its protected-cap or demotion parameters, which parent the reported deltas refer to, and whether the ghost filter ever engaged.
- **Per-policy notes:** Grouped by strict LRU, LRU with insertion-position change, segmented, second-chance, and ghost admission.

The "Recurring wins" section is thin. Only the neutral-change pattern recurs across cycles; the insertion-position gain has appeared once, so I placed it there with a caveat rather than calling it a recurring win.


---

- finished: 2026-10-20T14:24:53+00:00
- duration: 15.89s
- status: ok
- input_tokens: 4
- output_tokens: 2864
- cache_read_tokens: 37198
- cache_write_tokens: 10083
- cost_usd: 0.003821
