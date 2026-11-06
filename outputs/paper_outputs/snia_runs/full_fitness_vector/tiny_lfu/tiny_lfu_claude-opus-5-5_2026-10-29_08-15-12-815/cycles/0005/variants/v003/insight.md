# Variant v003 Insight

## Helped

- hit-rate: Accuracy held up much better than the hypothesis expected. The hypothesis allowed hit-rate at 131072 to fall up to halfway back toward cycle/0003/v001. Instead, web_0 at 131072 is flat and hm_0 is slightly up at both sizes. Only web_0 at 16384 slipped a little, so the summary is essentially unchanged from the parent. Pairwise conservative update still spares the shared counter in each pair. On these traces that seems to be enough to suppress the collision inflation that decides close `cand_freq > victim_freq` contests. The extra bumps it allows still go to rows that are not the key's overall minimum, and `estimate_hashed` mostly ignores those rows. So the exact four-way minimum was not what produced the parent's accuracy gain.

## Hurt

- per-op-time (get p50): The median `get` got clearly slower than the parent's, when the goal was to return to about 35 ns. That outcome contradicts the hypothesis's mechanism. The rewrite takes the dependency chain from three chained `min`s down to one `min` per independent pair, and nothing else in the file changed. A 3-deep integer `min` chain is only a few cycles. Neither its removal nor the one extra compare/flag can account for a regression of this size. The likeliest explanations are a codegen change: `increment_hashed` is only `#[inline]` (a hint), and the slightly larger body may have stopped being inlined into `get`/`put` or been vectorized differently. The other candidate is measurement noise. Without assembly or repeat runs, this data can't tell the two apart.
- per-op-time (get p99, put p50/p99): Every tail and the put median regressed even more than the get median did, with p99s roughly doubling or worse. `increment_hashed` runs once per get and once per put, and `reset` is `#[cold]`. A per-call cost from the pair arithmetic would scale both operations similarly; it would not mainly inflate the tails. The tail-heavy shape points more toward run-to-run or environmental variance (or an out-of-line call) than toward the min logic itself.

## Takeaway

Pairwise conservative update within row pairs (0,1)/(2,3) keeps essentially all of full conservative update's hit-rate benefit. That makes it a valid accuracy-equivalent substitute for the exact four-way min. However, it did not recover latency. Because of that, and because a 3-deep `min` chain is too small to matter, the parent insight's diagnosis looks wrong: the parent's `get` slowdown probably did not come from the serial `min` chain. Before more min-shortening variants are tried, the next step should be to compare the generated assembly and inlining of `increment_hashed` between v001 and v003, and to re-run the latency benchmark for both. That would separate a real codegen cost from noise.
