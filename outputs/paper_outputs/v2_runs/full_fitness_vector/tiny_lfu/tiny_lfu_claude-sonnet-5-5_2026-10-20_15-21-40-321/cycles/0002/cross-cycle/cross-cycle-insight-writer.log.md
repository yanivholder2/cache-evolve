# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:28:47+00:00

## timeline

### 15:28:57 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Making the frequency sketch cheaper (fewer hashes, cheaper index derivation, smaller counters) is the only direction that has paid off. Put-tail latency improves most.\n- Changes that leave counters, conservative update and admission semantics untouched keep hit-rate flat while cutting cost. Single-hash plus conservative update (cycle 1) and the single-mix bit-sliced hash (cycle 2) are examples.\n- Hashing and index derivation dominate per-op cost on workload-a. Cutting them is the most reliable lever.\n\n## Recurring failures\n\n- Aging or decay of any kind failed in both cycles, including a period 10x longer. workload-a has stable frequencies, so decay discards useful history. Without a drifting trace it has no case.\n- Adding mechanism to the admission path (small window, trial period) gained no hit-rate and cost time and metadata.\n- Aggressive precision cuts (4-bit counters, one row, tight sizing) buy speed and footprint but visibly lose hit-rate.\n- Extra state or bookkeeping on the hot path or in the cache struct adds latency and grows the struct without a matching benefit.\n\n## Open questions\n\n- How much of the nibble layout's hit-rate loss comes from the 4-bit cap, the single row, or the tight sizing? What is the cheapest fix (second hash or larger table)?\n- Does the cached-hash speedup combine with the compact layout, or does its extra state offset the footprint gain?\n- Which parts of the cycle-1 sketch rewrite (single hash, u8 counters, conservative update) account for its gains?\n- Do the u8 cap and conservative update hold on traces with larger frequency gaps or drift?\n- Is there any trace property that would justify aging, a larger window or a doorkeeper?\n- How much of the cached-hash attribution is real? It was inferred without source access.\n\n## Per-policy notes\n\n- TinyLFU (sketch plus strict-greater admission): the baseline admission rule and never-decaying counters suit workload-a. Improvement room is in sketch cost and accuracy, not in admission mechanics.\n- The headline metadata number tracks Node size, not sketch heap. Sketch savings are invisible there, and added struct fields show up as small growth.\n- Preferred parent for speed: the cycle-2 single-mix hash variant, which has no hit-rate cost. Next step is to combine it with the compact layout plus minimal extra accuracy.\n"
}
```

### 15:29:01 chat

I wrote `cross-cycle-insights.md` to the cycle 0002 cross-cycle output directory. It has the four fixed sections in order, built only from the two ledger cycles.

- **Recurring wins:** Making the frequency sketch cheaper (fewer hashes, cheaper index derivation, smaller counters) is the only direction that has paid off. Put-tail latency improved most. Changes that leave counters, conservative update and admission untouched keep hit-rate flat.
- **Recurring failures:** Aging failed in both cycles, even with a period 10x longer. Extra admission mechanism (the small window) and extra hot-path state add cost without benefit. Aggressive precision cuts (4-bit counters, one row, tight sizing) lose hit-rate.
- **Open questions:**
  - Which part of the nibble layout causes its hit-rate loss, and what is the cheapest fix?
  - Does the cached-hash speedup combine with the compact layout?
  - Which parts of the cycle 1 sketch rewrite account for its gains?
  - Do the u8 cap and conservative update hold on traces with drift?
  - Is there any trace that justifies aging, a larger window or a doorkeeper?
  - How reliable is the cached-hash attribution? It was inferred without source access.
- **Per-policy notes:** For TinyLFU, the baseline admission rule and never-decaying counters suit workload-a, so the room to improve is in sketch cost and accuracy. The headline metadata number tracks Node size, not sketch heap. The cycle 2 single-mix hash variant is the preferred speed parent, and the suggested next step is to combine it with the compact layout.

I used no numbers beyond those the ledger already states qualitatively.


---

- finished: 2026-10-20T15:29:02+00:00
- duration: 14.58s
- status: ok
- input_tokens: 4
- output_tokens: 1536
- cache_read_tokens: 35221
- cache_write_tokens: 6725
- cost_usd: 0.046081
