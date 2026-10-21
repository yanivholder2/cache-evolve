# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:25:44+00:00

## timeline

### 09:25:58 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Every cycle's Winner reduced noise or cost in the frequency signal. None of them changed policy shape. The winners were fast hashing (c1), sampled victims (c2), a wider sketch (c3) and get-then-put dedup (c4).\n- Lifetime frequency is the right signal for workload-a. Keeping long-run counts has beaten freshness every time it was tested.\n- The only large hit-rate gain in four cycles came from removing collision noise with a wider sketch. Every other win was marginal.\n- Strict `>` with keep-victim-on-tie held up once it was tested directly. Keep it as the contest rule.\n- Isolating one lever per variant has paid off. It showed that the early aging losses were confounded with conservative update and collision noise.\n\n## Recurring failures\n\n- Sketch aging/halving lost or stayed flat in every form tested: short period with conservative update (c1), long period alone (c2), and very long period on a clean sketch (c4). Freshness costs hit-rate on this trace.\n- Window restructuring or adaptation didn't pay off. A 1% fixed window did nothing (c1). A naive hill-climber drifted toward an LRU-like region, which this LIRS-style trace punishes (c2).\n- Tie-admission and wider victim sampling both hurt (c3). They act on estimates that never age, so the lowest estimate marks the youngest resident, not the coldest one.\n- Accuracy levers that fix the same error don't add up. Conservative update on top of a wide sketch was flat (c4).\n- Per-op-time and metadata-size have gone unscored in all four cycles. Speed and memory claims (fast hash, dedup, wide sketch) are all still unverified.\n\n## Open questions\n\n- With clean, honest estimates, can structural levers recover hit-rate? Examples are an SLRU probation/protected split, a small fixed window, or a doorkeeper. So far these were tested only on the noisy sketch.\n- A first-time newcomer still ties a resident accessed once, and the newcomer loses that tie. Is there a structural fix for this case that doesn't reopen tie-admission?\n- How narrow can the sketch go, perhaps narrower plus conservative update, before hit-rate drops? This matters once metadata-size is priced.\n- What do the per-op-time and metadata-size costs of the current lineage actually look like? A scoring run that reports them is a prerequisite.\n\n## Per-policy notes\n\n- **TinyLFU sketch:** Width was the decisive lever, and dedup added a small gain. Conservative update is redundant at high width. Aging is negative for workload-a. The sketch side looks exhausted as a hit-rate lever.\n- **Admission contest:** Keep strict `>` with keep-victim-on-tie. Contest-rule changes keep failing because the estimates they rely on were the real bottleneck.\n- **Victim selection:** Frequency-sampled victims from the LRU head gave a marginal gain. Widening the sample hurt, so treat this lever as exhausted.\n- **W-TinyLFU window / SLRU:** Results so far are neutral to negative: a fixed 1% window did nothing and an adaptive window hurt. This is the main untested lever on clean estimates and is worth one re-test.\n- **Hashing:** A single fast hash with a flat sketch is neutral on policy. Its speed benefit has not been measured.\n"
}
```

### 09:26:03 chat

I wrote `cross-cycle-insights.md` in the cross-cycle output folder. It rebuilds the cross-cycle memory from the four ledger rows only, using the four required sections in order.

**Recurring wins:** Every Winner so far reduced noise or cost in the frequency counts rather than changing the policy itself. The one large hit-rate gain came from the wider sketch removing collision noise. Long-run counts have beaten freshness every time on workload-a. Tested directly, the strict `>` admission rule (current resident wins ties) held up, so it stays.

**Recurring failures:**
- Every form of sketch aging tested either lost hit-rate or made no difference.
- Window restructuring, both the fixed 1% window and the hill-climber, didn't pay off.
- Admitting newcomers on ties and widening the victim sample both hurt.
- Conservative update adds nothing once the sketch is wide.
- Per-op-time and metadata-size have not been scored in any of the four cycles, so every speed and memory claim is still unchecked.

**Open questions:** Now that the estimates are clean, it's worth re-testing structural options such as a protected/probation split, a small fixed window or a doorkeeper. The others are how far the sketch can shrink before hit-rate drops, and getting a run that actually reports per-op-time and metadata-size.

**Per-policy notes:** These are grouped by sketch, admission rule, victim selection, window/protected-split structure and hashing. The sketch side looks used up as a hit-rate lever; the window/protected-split structure is the main lever still untested on clean estimates.


---

- finished: 2026-10-21T09:26:03+00:00
- duration: 19.09s
- status: ok
- input_tokens: 4
- output_tokens: 1795
- cache_read_tokens: 36160
- cache_write_tokens: 8758
- cost_usd: 0.113212
