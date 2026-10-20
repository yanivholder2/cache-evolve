# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:44:09+00:00

## timeline

### 14:44:16 thinking



### 14:44:21 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Behavior-preserving refactors (resident-key index, probe removal, bit-folding, hasher swap, ghost-structure changes) keep hit-rate flat across many cycles. That is a reliable property of the changes, but the cost gains they target have never been reported, so none is a confirmed win.\n- The only measured hit-rate gain is the cold-victim ghost feed (cycle 0009, main-queue evictions at zero frequency routed into the ghost queue). It is the one ghost-side change that has moved hit-rate, and cycle 0010 supports its shape: a capped side ring lost the gain, and a warm-victim feed was flat. It is still a single-trace, marginal result.\n\n## Recurring failures\n\n- Ghost re-admission routing keeps losing or failing to help: widening the ghost window (0003), routing ghost hits through the small queue (0004), and giving re-admissions an extra main-queue pass (0007). A capped side ring for main-fed entries also lost (0010).\n- Frequency levers look inert or negative: raising the main-queue frequency cap (0002), halving the decay step (0005), and carrying frequency 1 into main on re-admission (0007) produced no signal or a loss.\n- Promotion-bar and promotion-timing tweaks do not change residency: relaxing the bar to one hit lost hits (0001), raising it was neutral (0003), and earlier promotion of second-hit entries only moved timing (0008).\n- Adaptive small-queue sizing driven by ghost hits did not beat the fixed split (0005). Ratio tweaks in the first cycle were also neutral.\n- Retaining warm main-queue residents with a tail reinsert lost hits (0006).\n\n## Open questions\n\n- Does the cold-victim ghost feed hold on other traces and at other capacities? Every cycle so far used lirs workload-a at capacity 2048.\n- Does the cold-victim gain survive the unchanged linear ghost scan and `retain` cost, or does the scan cost offset it?\n- Do any of the cost-side changes (resident index, probe removal, bit-folding, hasher swap, ghost refactor, early return) lower per-op-time or metadata-size? Ten cycles in, no run has reported either.\n- Did the intended overlay edits land in the built code? Several variants across cycles could not confirm their edits, and some flat or negative results may reflect no-op or mismatched builds.\n\n## Per-policy notes\n\n- Small queue: sizing (fixed and adaptive) and promotion bar (one-hit, two-hit, three-hit, timing) have shown no reliable hit-rate signal on this trace.\n- Main queue: frequency cap, decay, reinsert position, and warm-victim retention have not helped. Cold-victim eviction is the only main-side path tied to a gain, and it works through the ghost queue.\n- Ghost queue: the cold-victim feed is the lead hit-rate candidate. Window size, small-queue routing, re-admission frequency, and fingerprint or structure changes have not produced gains. Its lookup cost remains unmeasured.\n- Resident index and membership: removing probes, folding membership into state bits, early return, and hasher swaps are cost-side work, all hit-rate neutral, and all unmeasured on cost.\n- Evidence scope: all cycles are S3-FIFO, one trace (lirs workload-a), one capacity (2048).\n"
}
```

### 14:44:24 chat

I wrote `cross-cycle-insights.md` to the cycle 0010 cross-cycle output directory, with the four required sections in order.

The main takeaways:

- **Recurring wins:** No pattern has yet shown a confirmed win. Behavior-preserving refactors stay flat on hit-rate, but none of them has reported per-op-time or metadata-size. The only measured hit-rate gain is the cold-victim ghost feed from cycle 0009, which is marginal and rests on one trace.
- **Recurring failures:** Ghost re-admission routing, frequency tweaks, promotion-bar and promotion-timing changes, adaptive sizing, and warm-main retention have all been flat or negative across cycles.
- **Open questions:** Whether the cold-victim feed holds on other traces and capacities, whether its gain survives the unchanged ghost scan cost, and whether the intended overlay edits actually landed in the builds.
- **Per-policy notes:** Grouped by small queue, main queue, ghost queue, and resident index. Scope is still one trace (lirs workload-a) at capacity 2048.

I did not read the previous `cross-cycle-insights.md`, as the instructions required.


---

- finished: 2026-10-20T14:44:24+00:00
- duration: 15.35s
- status: ok
- input_tokens: 4
- output_tokens: 2722
- cache_read_tokens: 41907
- cache_write_tokens: 14777
- cost_usd: 0.004736
