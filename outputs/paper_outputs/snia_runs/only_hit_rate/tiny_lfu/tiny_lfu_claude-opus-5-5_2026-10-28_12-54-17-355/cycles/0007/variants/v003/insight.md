# Variant v003 Insight

## Helped

- hit-rate: no meaningful gain. The one cell that rose (hm_0 at the small size) moved by a negligible amount. That fits the hypothesis's framing: the change was aimed at latency, not at admission quality. The policy code (climbers, `sampled_main_victim`, the strict `cand_freq > victim_freq` contest, double counting on get+put) is byte-for-byte the parent's. Only the sketch's hashing and memory layout changed.

## Hurt

- hit-rate: the summary slipped just outside the hypothesis's ±0.001 tolerance. Almost all of the loss is in one cell: web_0 at the large size. The likely mechanism is Kirsch–Mitzenmacher double hashing. The four row columns are `h1 + i·h2` (mod width), all derived from one 64-bit hash, so two keys that agree on `h1` and `h2` modulo the width collide in *every* row. With the parent's independent per-row SipHash seeds, a collision in one row said nothing about the others. Correlated full-row collisions inflate some estimates as a group. Since there is no aging and no conservative update, those inflated counts never wash out. They then flip close admission contests, and web_0 is the most contest-driven workload in this lineage (see the parent's insight). The other three cells barely moved, which suggests the effect is small and probably partly re-randomised contest outcomes rather than a systematic bias.
- per-op-time: not measured. The fitness vector contains only hit-rate, so this variant cannot confirm the hypothesis's central claim: that hashing once per sketch op (instead of 4× SipHash) and flattening `Vec<Vec<u32>>` into a single `Vec<u32>` lowers latency. The structural saving is real in the code: one `DefaultHasher` per `increment`/`estimate`, and no per-row pointer chase. But its payoff has no evidence here.

## Takeaway

Replacing four independently seeded SipHash rows with one fmix64-finalised hash plus double hashing is *nearly* hit-rate-neutral, but not free. It cost a small, concentrated loss on web_0's large-cache cell, where the admission contest is decided by narrow sketch-estimate margins and correlated cross-row collisions can tip them. Because the evaluator only scores hit-rate, this lineage cannot reward a pure latency change. The fix is therefore only worth keeping if a per-op-time dimension is added. Otherwise, the main lesson is that the sketch's row independence carries a little real admission accuracy on frequency-skewed traces.
