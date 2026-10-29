# Variant v003 Insight

## Helped

- per-op-time (get p50): The small get p50 gain is most likely noise. H03 left every hit path alone (target, probing and stash hits, plus the swap and `PROBE_WINDOW` arm), and get p99 did not move. So nothing in the change should lower get latency.

## Hurt

- hit-rate: Hit rate dropped in all four trace/size cells. The two 16384 cells dropped most, and web_0-131072, the cell H03 was meant to lift, also fell. This meets the hypothesis's refutation condition. Preferring a `lfu_counter == 0` victim mostly evicts the newest stash arrivals, because they have not had time to collect a hit yet. Meanwhile, any entry with a single hit is shielded until every slot reaches ≥1. One-hit entries pile up and fill the stash, which pushes out fresh items before they can reach the gate. Scan resistance is lost, the very failure the hypothesis warned about.
- per-op-time (put p50/p99): Put p50 and p99 both got slightly worse. This fits the new bounded scan over the full stash bucket's `lfu_counter`s, which now runs on every stash-full put/displacement. The parent took the LRU-position victim directly with no scan.

## Takeaway

Protecting stash entries with at least one hit from eviction does not help them pass the reuse gate. It turns the stash into a store of one-hit keys that rarely get reused, and it starves new arrivals of the time they need to show reuse. Small caches feel this most. In this design the stash's LRU-position eviction is already the right scan-resistant policy, and stash victim choice should not prefer evicting zero-hit entries. A later attempt would need a recency or aging bound on the protection, so that one old hit cannot shield an entry indefinitely. Caveat: I could not list or open the overlay source in this session. The code-level reasons assume H03 was implemented as written, using `lfu_counter` as the stash-hit signal.
