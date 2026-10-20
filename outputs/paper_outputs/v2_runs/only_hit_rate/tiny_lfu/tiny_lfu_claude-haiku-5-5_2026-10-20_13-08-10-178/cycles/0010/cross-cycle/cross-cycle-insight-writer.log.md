# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T13:56:31+00:00

## timeline

### 13:56:38 thinking



### 13:56:43 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Choosing probation victims by sketch estimate over a small tail window has been the most consistent gain: two candidates (C0008), then three (C0009), then four (C0010). Each step helped less than the last, so returns are tapering.\n- Reducing estimator collision error is the main admission lever: sketch width (C0004), an extra hash row (C0007), and conservative update (C0005) each helped. Their gains appear to overlap rather than stack independently.\n- Protected-segment sizing near the incumbent ratio is the main-region setting that holds up. A modest share increase helped once (C0006); pushing it further reversed the gain (C0007).\n\n## Recurring failures\n\n- Aging or halving the sketch on a fixed or table-wide trigger keeps losing hit-rate (C0001, C0005, C0009). It erases the frequency history the admission comparison depends on.\n- Gating admission, promotion, or sketch updates on a filter keeps losing hit-rate (C0002 window ahead of LRU, C0006 doorkeeper, C0007 sketch-gated promotion). Frequency counts lag recency, and withheld increments are not recovered by estimate-time correction.\n- Removing estimator depth or read-side increments hurts, since both reduce the popularity signal the admission rule uses (C0005, C0010).\n- Layout-only and storage-narrowing changes hold hit-rate by construction (C0002, C0003, C0009) but their claimed gains (footprint, per-op time) are not measured, so they cannot be credited.\n- Tie-breaking changes that admit equal-estimate newcomers against probation victims hurt (C0008). Queue position was protecting older cold entries the sketch cannot separate.\n- Ghost-history frequency bonuses barely move hit-rate and carry unmeasured metadata and per-op cost (C0008).\n\n## Open questions\n\n- Do the victim-scan, estimator, and protected-share gains hold on other workloads and cache sizes? Every result so far is one trace at one size.\n- Does the victim scan plateau past four candidates, and does its gain depend on the earlier width and conservative-update changes?\n- Is the Winners designation for Cycle 0007 correct? v001 reported a hit-rate loss and only v002 showed a gain; this needs resolving before building on either.\n- What metadata and timing signals would let layout, depth, and conservative-update costs be weighed against their gains?\n- Would per-row or per-key aging with a threshold that actually fires help bound counter growth without losing admission history?\n\n## Per-policy notes\n\n- Admission (frequency sketch): width and depth help, and conservative update adds a small gain on top. Fixed or global halving hurts. Filtering first-touch keys or read-side increments hurts. Counter narrowing is neutral on hit-rate.\n- Main region (probation/protected segments): the incumbent protected share is near the best setting; both a halved cap and a much larger share lost. Demotion to the back of probation is the better placement; push-to-front lost.\n- Victim selection (probation eviction): comparing several LRU-tail candidates by sketch estimate is the strongest lever, with diminishing returns. Breaking ties in favour of the newcomer hurts.\n- Pre-admission structures: a recency window ahead of LRU with admission at exit lost hit-rate; a doorkeeper used as an update filter lost hit-rate. Neither should be retried in its current form.\n- Ghost history: the +1 recently-evicted bonus is marginal and not worth keeping without a more selective signal and measured cost.\n"
}
```

### 13:56:46 chat

I wrote `cross-cycle-insights.md` to the cycle 0010 cross-cycle output directory, using the four fixed sections in the required order. I built it only from the ledger digest in the prompt and did not read any previous version of the file.

The main signals across cycles 0001–0010:

- **Recurring wins:** choosing probation victims by sketch estimate over a small tail window (two, then three, then four candidates) has been the most consistent gain, though each step helps less. Reducing estimator collision error (sketch width, an extra row, conservative update) is the main admission lever, and those gains look like they overlap. The protected share works best near the incumbent ratio.
- **Recurring failures:** fixed or table-wide sketch halving has lost hit-rate in three cycles. Gating admission, promotion, or updates on a filter has also lost each time. Layout-only changes hold hit-rate by construction, but their claimed gains are unmeasured.
- **Flag on the Cycle 0007 Winners:** the designation names v001, but v001's record shows a hit-rate loss. I listed this as an open question rather than treating either variant as confirmed.

Every result comes from one trace at one cache size, so I framed all the gains as provisional.


---

- finished: 2026-10-20T13:56:47+00:00
- duration: 16.12s
- status: ok
- input_tokens: 4
- output_tokens: 2724
- cache_read_tokens: 40442
- cache_write_tokens: 13312
- cost_usd: 0.004429
