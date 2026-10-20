# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:14:05+00:00

## timeline

### 20:14:21 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Static sizing of admission is what has paid off: a much smaller small queue (~5%) plus a shorter ghost. Together they give the main gain over baseline. Later cycles showed this sizing-only setup matches more complex bundles, so it is the preferred simple base.\n- Ghost length acts as an admission filter: longer hurt, shorter helped. Small-queue shrinking gave the clearest gain of any single lever, but with diminishing returns.\n- Adding information without adding protection or loosening rules (e.g. ghosting main evictees) is safe but near-neutral.\n- Hit-path promotion (small to main when freq reaches 2) is the first new lever in many cycles to show a gain that is real in behaviour terms. Its gain is still confounded with bundled sizing.\n\n## Recurring failures\n\n- Loosening admission (looser promotion gate, longer ghost, pre-protected or boosted ghost re-admits, grace passes) lost every time. It lets weakly reused keys displace useful residents.\n- Ageing and decay changes (faster freq halving, lower freq cap, in-place ageing) lost every time. This workload rewards keeping reuse history. A higher freq cap gave at most a noise-level gain.\n- Rewrites of main-eviction mechanics (lookahead, reinsertion caps) lost. The unbounded second-chance reinsertion scan does real protective work.\n- Filtering ghost hits or ghost content (freq>0 fast path, reuse-only ghosting) lost. Ghost hits are strong evidence and should stay permissive.\n- Eviction-time promotion-gate strictness tied twice. It is either non-binding or a no-op, so it is not worth more cycles.\n- Adaptive or feedback-driven sizing lost to the fixed ratio. Per-key ghost metadata and main insertion-position tweaks gave nothing.\n- Most cycles' best variant was a tie or marginal gain, and many gains may be within noise.\n\n## Open questions\n\n- Is the hit-path promotion gain real once the parent is the sizing-only base? Which threshold and which main insertion position are best?\n- What does hit-path promotion cost in `get` per-op time? Cost dimensions (metadata-size, per-op-time) have never been measured in any cycle.\n- Do any gains hold on a second trace or cache size? All evidence is one trace (LIRS) at one cache size.\n- Do the no-op-suspected variants (promotion gate, insertion position) actually change behaviour? Authors could not read overlay source.\n- Is S3-FIFO at a local optimum for this trace, so that only structural hit-path levers remain?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy so far): the base is baseline plus a small queue of ~5% and a shorter ghost, with simple second-chance main eviction unchanged.\n- Small-to-main promotion at eviction time is insensitive to freq threshold; hit-path promotion is the untested-in-isolation lever.\n- Ghost: shorter is better, content and selection filters do not help, and extra per-key state adds nothing.\n- Main eviction: keep the parent's reinsertion semantics, and avoid ageing.\n- Next cycles should use the sizing-only base as the explicit parent, change one hit-path lever at a time, validate on a second trace, and restore cost measurements.\n"
}
```

### 20:14:26 chat

I wrote `cross-cycle-insights.md` in the cycle 0008 cross-cycle output directory, regenerated from the eight-cycle ledger digest. It has the four required sections in the fixed order.

Its main conclusions:
- **Recurring wins:** Fixed admission sizing is what has paid off: a small queue of about 5% plus a shorter ghost. A sizing-only setup matched the more complex bundles, so it is the preferred base. Hit-path promotion is the one newer lever that looks promising, but its gain is still mixed in with the sizing it was bundled with.
- **Recurring failures:**
  - Loosening admission lost every time it was tried.
  - Ageing and decay changes lost every time.
  - Rewrites of main-queue eviction lost, as did filtering ghost hits or ghost content.
  - Adaptive sizing lost to the fixed ratio.
  - Making the eviction-time promotion gate stricter tied twice, so it may have no effect.
- **Open questions:**
  - Does hit-path promotion still help once the parent is the sizing-only base, and what does it cost per `get`?
  - Do any gains hold on a second trace or cache size? All evidence so far is one trace at one cache size.
  - Do the variants suspected of changing nothing actually change behaviour?
  - Cost (metadata size, per-op time) has never been measured in any cycle.
- **Per-policy notes:** These cover S3-FIFO only, since it is the only policy in the ledger. The next cycles should use the sizing-only base as the explicit parent, change one hit-path lever at a time, validate on a second trace, and restore the cost measurements.

I kept the file qualitative, with no invented numbers, since I only had the digest.


---

- finished: 2026-10-20T20:14:26+00:00
- duration: 21.37s
- status: ok
- input_tokens: 4
- output_tokens: 1828
- cache_read_tokens: 39207
- cache_write_tokens: 11003
- cost_usd: 0.066915
