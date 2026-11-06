# Variant v003 Insight

## Helped

- hit-rate: At the 16384 size, hit-rate rose on both web_0 and hm_0, and that size is where the hypothesis predicted a gain. Going from a 1% to a 5% window grows the window from about 164 to about 819 entries. Recency bursts now stay in the window long enough to be re-referenced and build up sketch frequency before they face the `cand_freq > victim_freq` contest. Under the 1% window they reached the contest cold and lost. Every window eviction still goes through that contest, so the gain cannot come from skipping admission.

## Hurt

- hit-rate (131072 size): The gain is gone at the large size. web_0 dipped slightly and hm_0 moved by only a hair. A 1% window at 131072 is already about 1300 entries, enough to absorb the bursts. At this size the extra 4% of capacity mainly takes slots from probation and protected (`main_cap = capacity - window_cap`, with `protected_cap` still at 80% of it). Those slots were holding frequency-proven entries.
- per-op-time: Median get, median put and both tails all got slower, even though `get`, `put`, `evict_one`, `on_hit` and the sketch are byte-identical to the parent. The only code change runs once, in `try_new`. So the regression is not new per-op instructions. Either the benchmark is noisy, or the larger window changes which `on_hit` path each hit takes, or both. This change cannot explain a regression this large through added work. Re-run the benchmark before blaming the window size.

## Takeaway

A fixed 5% window, with the frequency contest kept on every window exit, accounts for a static share of the small-cache hit-rate gain without any runtime adaptivity. That answers H03's question in favour of "a larger window" over "skipping the contest". The benefit depends on size: it pays where 1% of capacity is too small to hold a recency burst, and it gives nothing back once 1% is already large. That suggests sizing the window as an absolute entry floor, e.g. max(1% of capacity, about 800 entries), rather than as a single percentage. The per-op-time result should not be read as the cost of this change, because the hot path was not touched.
