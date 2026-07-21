## Recurring wins

- **MRU-end insertion** has been a stable anchor since Cycle 1; every winning lineage carries it forward, and it changed tier pressure dynamics enough to invalidate prior results from the LRU-end base (e.g., stash bypass reversed sign when the base changed).
- **Struct field elimination** (timestamp removal) produces compound dividends — smaller per-item footprint, better cache-line utilization during bucket traversal, and eliminated syscalls — with zero hit-rate cost. Identified in Cycle 1, amplified in Cycle 6 (40 % footprint drop on the clock base).
- **Vacancy-gated frequency promotion (≥2 threshold)** has been the stable promotion-side optimum since Cycle 3. Structural cascade elimination (firing only into open slots) consistently beats threshold-only tuning, and the ≥2 boundary is the confirmed local optimum for workload-a.
- **Stacking orthogonal mechanisms is additive** when the winning variants touch separate subsystems. Confirmed Cycle 2 (insertion + struct), re-confirmed Cycle 5 (eviction + promotion gate), and Cycle 6 (three-way stack). Stacking fails when mechanisms share internal state (see Recurring failures).
- **Frequency-aware eviction improves hit-rate independently of promotion changes.** LFU-minimum (Cycle 5) and two-chance clock (Cycle 6) both showed eviction selection is a self-contained lever, not auxiliary to promotion gating.

## Recurring failures

- **O(n) bucket scans for eviction victim selection** impose a consistent tail-latency regression that recurs across every cycle they appear in (Cycles 5, 7, 9). The two-chance clock (Cycle 6) solved this for the baseline case, but rotation depth in hot buckets is the persistent residual problem.
- **Stash tier interventions do not move hit-rate on workload-a.** Stash bypass (Cycles 2, 3, 8), vacancy-gated stash bypass (Cycle 4), and cold-preferring stash eviction (Cycle 8) all failed to shift aggregate miss curves. The stash tier holds too small a fraction of total capacity, and stash-insert conditions rarely produce high-frequency items on this trace.
- **Overlay coverage gaps silently revert parent struct changes.** Seen in Cycle 4: child overlays that modify only one file can silently un-apply multi-file parent changes (e.g., re-inflating the item struct), making regressions appear from the new hypothesis when the cause is inheritance incompleteness.
- **Counter decay is antagonistic to frequency-gated promotion on stationary workloads.** Cycle 6 showed that post-eviction counter halving directly suppresses items below the promotion threshold and adds a second O(n) pass, doubling median latency with no hit-rate gain on a fixed Zipfian distribution.
- **CLOCK_BIT set on every non-hottest access is too dense.** Cycle 8 found that marking items warm on each ordinary access leaves the clock with no discriminating power — all resident items appear warm, replicating LFU-minimum behavior without the scan. Sparsifying at freq≥3 (Cycle 9) overcorrects and ejects too many working-set items.
- **Probing re-entry vacancy gate is persistently thin on saturated caches.** The joint condition (counter threshold + target vacancy) rarely fires when the cache operates at high occupancy, producing inert results across Cycles 5, 7, and 8 on workload-a.
- **Frequency threshold ≥3 for promotion or clock protection is too strict for workload-a.** Hot set frequency is concentrated at 1–2; a ≥3 boundary cold-classifies a significant fraction of the genuine working set, confirmed across Cycles 4 and 9.

## Open questions

- **What is the right CLOCK_BIT threshold?** freq≥1 is too dense, freq≥3 is too sparse. freq≥2 is the natural next test; it may recover the Cycle 9 hit-rate signal while reducing hot-bucket rotation depth.
- **Can a rotation cap bound clock tail latency?** Aborting `clock_evict_prepare` after k cleared bits and falling back to position-0 could prevent the hot-bucket O(k·n) worst case without sacrificing the cold-tail discrimination that makes the clock valuable.
- **Can the clock chance signal be decoupled from `lfu_counter`?** A separate boolean chance bit — set only when an item is rescued from eviction, not on ordinary accesses — would let the clock coexist with promotion gates without either corrupting the other's signal. This is the most direct path to composing clock eviction with frequency-gated promotion.
- **Is target-path counter increment viable with a gate?** Cycle 7 confirmed the hit-rate signal is real but unconditional firing on the common path triples median latency. A conditional guard (fire only when item is near eviction risk) might capture the gain without the overhead.
- **At what cache occupancy does the probing re-entry gate become effective?** The vacancy condition fails on saturated caches but has never been tested at lower occupancy or larger cache-to-working-set ratios.
- **Do any further struct fields or per-operation overhead remain auditable?** The timestamp audit (Cycle 1) and the 40% footprint drop (Cycle 6) suggest more dead-weight may exist; a systematic audit of item struct fields and hot-path allocations has not been done since Cycle 1.

## Per-policy notes

- **LFU-minimum eviction:** Independently validated hit-rate lever (Cycle 5); the O(n) per-eviction scan is the blocker for production use. Never combined cleanly with the clock base because both compete for the same counter values.
- **Two-chance clock eviction:** Best eviction mechanism found to date — frequency-aware without a full scan, amortized O(1) on cold-tail Zipfian workloads. Introduced in Cycle 6; replaces LFU-minimum as the eviction anchor. Core unresolved problem: CLOCK_BIT sharing `lfu_counter` with promotion gates creates mutual interference.
- **Vacancy-gated frequency promotion (≥2):** Stable since Cycle 3. Raising threshold to ≥3 is confirmed harmful; the ≥2 boundary captures genuine reuse signal without excess cascade. Not yet successfully composed with two-chance clock on a shared-counter base.
- **LRU positional eviction (baseline):** Serves as the reference; confirmed suboptimal vs. any frequency-aware alternative, but incurs no latency overhead. Useful as fallback when frequency-aware variants impose unacceptable tail cost.
- **Counter decay:** Antagonistic to frequency-gated promotion on stationary distributions (Cycle 6). Not worth retesting unless a non-stationary trace (shifting popularity) is available.
- **Stash cold-preferring eviction:** Amortized-safe (fires only at full-bucket insertions), so it is a latency-cheap carry-forward even though it contributes no hit-rate gain on workload-a. Not harmful to include in a stack; just not helpful.
