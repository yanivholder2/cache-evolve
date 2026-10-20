# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:44:18+00:00

## timeline

### 14:44:26 thinking



### 14:44:32 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Insertion depth is the only lever with repeated hit-rate gains. New keys placed below the hot head (tail in one form, a midpoint marker in another) produced small gains in several cycles, and the midpoint form is the most consistent.\n- Eviction-neutral layout and placement changes (sentinel indices, head-hit relink skip, pre-sized index, hasher swap) hold hit-rate flat against their parent. This is expected behaviour preservation, not yet a demonstrated benefit.\n- A frequency signal helps only when it is confined to the LRU end. A small bounded victim window gave a marginal gain, while wider windows lost.\n\n## Recurring failures\n\n- Changes that alter eviction order structurally have lowered hit-rate: segmented recency lists, second-chance referenced bits, and second-hit promotion. Strict recency ordering appears to do useful work on this trace.\n- Frequency counters and admission gates have been flat or slightly negative: decaying or capping hit counters, count-min admission gating, and widening the frequency-aware victim window.\n- Tuning the segmentation in the downward direction lost: shrinking the protected share lowered hit-rate, so the larger protected share holds re-referenced keys better.\n- Lengthening ghost-set retention lost hit-rate, so longer ghost memory admits keys that do not recur.\n- Winners have repeatedly been named without a measured gain on the dimension they target, and some were named despite a hit-rate dip. This selection-criterion gap runs from cycle 0001 through cycle 0010.\n- Rejections for infrastructure reasons (session limits, missing insight files) carry no strategy signal and should not be counted as failures of their approach.\n\n## Open questions\n\n- Per-op time and metadata size have not been recorded for any variant in any cycle. Every claim about memory or time cost is untested.\n- Does the midpoint insertion gain hold on a second trace or another cache size, and how does it change at other marker depths?\n- Does the larger protected share hold up above its current setting, or is it already near the best point?\n- Did the tail-insertion variants that conflicted (gain, loss, null across cycles 0005 to 0007) differ mainly in insertion depth? Is that conflict now explained?\n- What does the ghost-set retention behaviour look like on another trace, now that engagement has been confirmed?\n- Does the frequency-aware tail-window gain survive on a second trace, or is it specific to the current workload and size?\n- Hypothesis labels (notably H03) have been mapped to different mechanisms across cycles. Which definition should future lineages cite?\n\n## Per-policy notes\n\n- Plain LRU (strict recency): changes that reorder the recency list have lost hit-rate; placement-only changes that keep order hold it flat. Shallower-than-head insertion of new keys is the main candidate for a gain.\n- Segmented (probation/protected): the larger protected share outperforms the smaller one. Midpoint insertion gives a small gain on this base. The split alone does not beat the baseline.\n- Frequency-aware eviction: a bounded window near the LRU end is the only frequency variant with a marginal gain. Wider scans, counter decay, and capped counters did not help.\n- Ghost/admission filters: confirmed to engage once retention is varied, and longer retention lost. Count-min and correlated-reference gates have not produced a measured gain.\n- Second-chance and second-hit promotion: both lowered hit-rate and are not supported on this trace.\n"
}
```

### 14:44:35 chat

I wrote `cross-cycle-insights.md` to the cycle 0010 cross-cycle output directory, using the four fixed sections in order. I did not read the previous version of the file.

The main signals across cycles 0001–0010:

- **Recurring wins:** Placing new keys below the hot head is the only lever with repeated hit-rate gains, most consistently the midpoint form. Eviction-neutral layout changes hold hit-rate flat, which shows behaviour preservation but not a benefit. A frequency signal helps only near the LRU end.
- **Recurring failures:** Structural changes to eviction order (segmented lists, second-chance, second-hit promotion) lowered hit-rate. Counter and admission variants were flat or negative. Shrinking the protected share and lengthening ghost retention both lost. Winners have repeatedly been named without a measured gain on their target dimension.
- **Open questions:** Per-op time and metadata size have never been recorded, so every memory or time claim is untested. The midpoint gain, protected-share upper range, and frequency-window gain all need a second trace or size. The H03 label has mapped to different mechanisms across cycles.
- **Per-policy notes:** Grouped by LRU, segmented, frequency-aware, ghost/admission, and second-chance.

The file avoids specific numbers beyond the parameters already in the ledger, and it describes magnitudes qualitatively.


---

- finished: 2026-10-20T14:44:35+00:00
- duration: 17.62s
- status: ok
- input_tokens: 4
- output_tokens: 3148
- cache_read_tokens: 41229
- cache_write_tokens: 14455
- cost_usd: 0.004878
