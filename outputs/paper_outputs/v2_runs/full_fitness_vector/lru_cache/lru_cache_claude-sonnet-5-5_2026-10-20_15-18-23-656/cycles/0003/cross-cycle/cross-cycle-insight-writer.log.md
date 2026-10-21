# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:39:26+00:00

## timeline

### 15:39:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Mechanics changes on the plain LRU (u32 links, lighter/folded Fx hasher, head-touch early return, move-based eviction instead of key clone) gave speed and/or size without touching hit-rate. Hit-rate came only from policy; speed and size came only from mechanics. The axes are independent.\n- Segmented LRU is the one proven hit-rate lever (scan resistance). It is established; no need to re-prove it.\n- Combining independent winners (segmentation + fast hasher + cheap touch + clone removal) worked in cycle 0002, with put gaining most.\n- Packing the segment bit into an existing link field keeps per-node size flat.\n\n## Recurring failures\n\n- Once the policy is segmented, link-width and bit-packing changes stop bringing speed. They shrink size at best, and add widening/NIL/mask work to every link access.\n- Segmentation's real cost is promotion/demotion relinking on the hot path. Moving it between get and put (lazy demotion) or packing flags only relocates the cost. Median gains have often come with worse tails (put p99 from relinking, get p99 from weak hashing).\n- Bundling several changes (fold + layout, hasher + touch + clone) makes attribution guesswork, since authors cannot read diffs. This blocked conclusions in cycles 0002 and 0003.\n- Per-node size metrics hide container growth (extra head/tail pair).\n- Infrastructure failures (build rate limit) wasted a slot in cycle 0001. They say nothing about the idea's merit.\n\n## Open questions\n\n- Is the hash fold or the u32 conversion responsible for the slowdown seen on segmented u32 variants? Test them in isolation.\n- Can promotion be made cheaper by skipping the relink for nodes near the head, or by sampling or batching it?\n- Can the segment be stored implicitly (list membership) so no masking is needed?\n- Does the hit-rate gain justify the latency cost on traces other than the scan-heavy one?\n- Does the fold fix get/put tails on a segmented policy? This is unconfirmed.\n\n## Per-policy notes\n\n- Plain LRU: the mechanics wins are mostly exhausted. u32 links, the folded Fx hasher and move-based eviction were speed and size wins.\n- Segmented LRU: good hit-rate on scans, but it costs per-op time through promotion/demotion. The cycle-0002 merged candidate (segmented, u32 links, folded Fx, move-based eviction) is the baseline. New variants should recover speed without losing hit-rate.\n- Segmented plus u32 links: size gain only, with no speed gain observed so far.\n"
}
```

### 15:39:41 chat

I wrote `cross-cycle-insights.md` in the cycle's `cross-cycle` output directory, with the four fixed sections in order. I built it only from the three-cycle ledger digest and did not read any code or an earlier insights file.

The main conclusions it records:
- **Recurring wins:** Mechanics changes on the plain LRU (u32 links, the folded hasher, a head-touch early return, move-based eviction) gave speed or size without changing hit-rate. Segmentation is the only proven hit-rate lever.
- **Recurring failures:** Once the policy is segmented, link-width and bit-packing changes stop bringing speed. The cost is the promotion/demotion relinking, and moving it between `get` and `put` only relocates it.
- **Open questions:** Whether the hash fold or the u32 conversion causes the slowdown on segmented variants, and whether promotion can be made cheaper.
- **Per-policy notes:** The cycle-0002 merged candidate (segmented, u32 links, folded Fx, move-based eviction) is the baseline for new variants.

I kept the numbers qualitative, as instructed.


---

- finished: 2026-10-20T15:39:42+00:00
- duration: 15.93s
- status: ok
- input_tokens: 4
- output_tokens: 1394
- cache_read_tokens: 36303
- cache_write_tokens: 7868
- cost_usd: 0.049452
