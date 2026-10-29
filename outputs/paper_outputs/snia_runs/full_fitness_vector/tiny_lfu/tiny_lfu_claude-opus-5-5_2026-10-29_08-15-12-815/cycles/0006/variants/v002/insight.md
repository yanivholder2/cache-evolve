# Variant v002 Insight

## Helped

- hit-rate (16384 cells): Both small-size cells went up by a tiny amount. This is about the size of run-to-run noise. It fits the hypothesis's prediction that the small size would stay flat, and is not a real gain from the wider addressing.

## Hurt

- hit-rate (131072 cells): The predicted large-size gain did not happen. hm_0 is unchanged and web_0 slipped a little. The rationale's "~4× lower collision load per counter" does not hold. Each block still has 128 nibbles, and each addition still makes 4 increments. Before, each row put 1 increment into its private 32 nibbles. Now, 4 increments spread over 128 shared nibbles. In both layouts the average load per counter is additions/32. The change only moved where collisions happen. Rows now share counters across keys, and about 6/128 of keys lose an independent row to an intra-key duplicate. Neither of these reduces overestimation in close `cand_freq`/`victim_freq` contests.
- per-op-time (put): Put p50 and especially put p99 got worse; get stayed about flat. The likely cause is that the parent's row `i` could only touch words `2i`/`2i+1`. The four read-modify-writes in `increment_hashed` were therefore provably disjoint and could be scheduled independently. With `word = (b >> 4) & 7`, any two rows may alias the same word. Each row's load must then wait for the previous row's store, a store-to-load dependency through `block[w]`. Put is hit hardest, possibly because it runs this sketch path more often (increment plus the two `estimate_hashed` calls during eviction). That part is not isolated.

metadata-size is unchanged, as designed. The block count and `heap_bytes()` are untouched, and the residency layer and node are byte-identical to the parent.

## Takeaway

Widening per-row addressing inside a fixed 128-counter block cannot lower per-counter load: the load is set by increments ÷ counters per block, not by how many positions each row can reach. So H02's premise was wrong at the arithmetic level, and the flat 131072 hit-rate confirms it. Tying each row to its own word pair turns out to be valuable: it makes the rows independent and gives a no-alias guarantee that keeps put latency low. A lever against large-size overestimation has to change counters per addition (more blocks/width, or conservative update) rather than how rows are laid out within a block.
