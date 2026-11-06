# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-28T20:14:27+00:00

## timeline

### 20:14:43 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Compacting node metadata is the most dependable lever. The `u32`-index layout (c1) became the base for every later cycle and costs nothing on any other axis.\n- Packing policy state into bits the node already has works. A segment flag in a link's high bit and a counter in the padding byte (c2) both kept the node compact. Any new field that misses existing spare bits costs a full word, because the node has no free padding (c1).\n- Changes that only touch representation move one axis cleanly and leave hit rate exactly unchanged (c1, c3 v002). Policy changes move several axes at once, often in opposite directions.\n- SLRU is the strongest hit-rate policy so far. Once it was flag-packed onto `u32` nodes (c2) it kept its gains on every trace at both sizes, and its only remaining cost is on put.\n\n## Recurring failures\n\n- Hash-map speed levers: a cheaper hasher left get p50 flat (c1) and then made it worse (c2). The probe looks bound by memory and sensitive to how well the hash spreads structured block keys. This is inferred; probe lengths have not been measured.\n- List and relink micro-optimisation as a get-latency lever. Removing relinks (c1), compacting nodes (c1) and specialising relinks (c3) all left get p50 within noise. List maintenance does not dominate the cost of a hit.\n- Side structures kept per cache (ghost `HashSet`, SipHash sketch, c3) made get p50 worse even though get never reads them, probably through CPU-cache pressure. Each one also adds about one hash operation per probe to put.\n- History-based admission in its crude forms fails on web_0. The eager form (a full-key ghost that promotes straight to protected and forces demotion) and the strict form (a miss-only, tie-rejecting gate) both hurt it, even though hm_0 gained slightly under each (c3).\n- Policies that buy hit rate keep paying for it on put: SLRU's double relink, GCLOCK's sweep multiplied by decrement-by-one aging (c2), and the hashing added by ghosts and sketches (c3). The put path is bound by hashing and memory, not by list logic.\n\n## Open questions\n\n- web_0 at the large cache size has not improved in any cycle, whether through recency segmentation, small counters, a ghost or a sketch. It reacts to how aggressive admission is in both directions. Candidates: a tiny fingerprint ghost with gentle admission, W-TinyLFU (counts hits too, small window, cheap hash), or an adaptive probation share.\n- SLRU put cost: avoid the second relink on promotion, add a fast path for updating keys that already exist, and avoid cloning the key on insert.\n- get latency: table density, hash quality (a better-mixing finaliser), and map/arena locality are all untested. Speed of the hash itself is ruled out.\n- Benchmark attribution: the GCLOCK get slowdown (c2) and the side-structure get regressions (c3) were never separated from noise caused by interleaving puts and gets. An isolated get benchmark would settle several open threads.\n\n## Per-policy notes\n\n- LRU (baseline): its node layout has no free padding. `u32` indices are a free improvement whatever the policy.\n- SLRU: the default base since c2 (flag-packed, `u32`). It is weak at the small cache size, where the probation share is untuned. Adding a ghost or a TinyLFU gate in front of it has not beaten the plain version.\n- CLOCK: lazy promotion gave a small metadata win, and get barely moved. Its single bit helped hm_0 at both sizes but was mixed on web_0. It has put tail latency from sweeps, and that is still unbounded.\n- GCLOCK: its hit-rate gain was concentrated in hm_0 at the large size, and decrement-by-one aging inflates sweeps. It is only worth revisiting if aging moves off the eviction path (for example bulk halving).\n- TinyLFU / ghost admission: only crude versions have been tried. Any retry has to be tiny, cache-friendly and gentle at admission.\n"
}
```

### 20:14:49 chat

I've written `cross-cycle-insights.md` in the cycle-0003 cross-cycle output dir. It has the four required sections, built only from the three-cycle ledger and with no numbers added.

- **Recurring wins:** shrinking node metadata (the `u32` index layout) is the most dependable improvement and is now the base for everything. Packing policy state into bits the node already has works. Flag-packed SLRU (segmented LRU) gives the best hit rate, and its remaining cost is only on put.
- **Recurring failures:** three get-latency levers have failed so far: a faster hasher, trimming the list and relink code, and adding side structures like a ghost list or frequency sketch, which made get slower. Crude admission filters have not fixed web_0. Every policy that raises hit rate has added cost on the put path.
- **Open questions:** web_0 at the large cache size is the main unsolved case. The other threads are:
  - cutting SLRU's put cost
  - map density and memory locality as untested ways to speed up get
  - a benchmark that measures get on its own, to tell real slowdowns from noise
- **Per-policy notes:** separate entries for baseline LRU, SLRU, CLOCK, GCLOCK and the TinyLFU/ghost admission variants.


---

- finished: 2026-10-28T20:14:49+00:00
- duration: 22.78s
- status: ok
- input_tokens: 4
- output_tokens: 1919
- cache_read_tokens: 36992
- cache_write_tokens: 8559
- cost_usd: 0.114266
