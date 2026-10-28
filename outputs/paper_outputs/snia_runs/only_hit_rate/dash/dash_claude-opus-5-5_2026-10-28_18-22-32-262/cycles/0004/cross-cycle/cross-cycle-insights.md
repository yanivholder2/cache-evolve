# Cross-Cycle Insights (through cycle 0004)

## Recurring wins

- **Recency-protecting placement beats admission tuning.** The changes that held up are about *where* items land: MRU-end stash demotion (0001) and `Back` (MRU) placement on promotion into the target (0003). Each was a clean win and later cycles kept building on it. In both cases the old placement was turning a protected item into the next eviction victim.
- **The two-hit stash exit gate is a net positive in tight caches.** It gave consistent gains at 16K in 0001–0003. It earns its keep only when target slots are contested.
- **Occupancy-conditioned gate bypass recovers `web_0` 128K.** Skipping the gate when the target has room fixed the one cell the gate hurt (0003 v002). It became safe only with a strict room threshold of two free slots (0004), which confirmed that the 16K losses came from a last-free-slot cascade.
- **Plain LRU is the right rule for stash eviction and stash placement.** Every departure lost hit-rate: LRU-end insertion of fresh puts (0002) and counter-based victim choice (0004).
- **Lineage:** v001 (0001) → gate (0002) → `Back` promotion (0003) → strict free-slot bypass (0004). Each step added one non-regressing lever.

## Recurring failures

- **Frequency/counter signals don't bite in their current form.** Counters stay near zero because they are halved or reset on moves (0002). Without decay they pin items permanently (0004 v003). Target-hit counts barely filter anything (0004 v002). The `lfu_counter` lifecycle matters more than any threshold placed on top of it.
- **Making entry into the target easier hurts small caches.** One-hit return for demotees (0003) and a last-free-slot shortcut (0003) both cost 16K. When space is tight, a "free" slot is not actually free.
- **Extra filtering on `web_0` 128K always costs hit-rate.** The gate, the frequency comparison, LRU-end insertion and delayed probing (0004) all regressed or stayed flat there. When targets have room, first-hit promotion is close to optimal.
- **Explanations ruled out for `web_0` 128K:** the counter reset on demotion (0003), one-hit churn into probing (0004), and stash flushing of half-gated items (0004).
- **Cost dimensions have never been measured.** Per-op-time and metadata-size were missing in all four cycles, so every cost claim (including the 0001 lookup reorder) is still unverified.

## Open questions

- Can `web_0` 128K be lifted by letting promotees reach probing *more easily*, for example on any target hit? Delaying probing made it worse.
- Is there a frequency signal with explicit decay or aging that can separate hot items from one-hit items without pinning them?
- Can the small `hm_0` gains (delayed probing, protecting residents) be kept with a trace-adaptive or occupancy-conditioned trigger that leaves `web_0` alone?
- Is there any stash protection, such as one that expires or covers only the newest region, that beats plain LRU at 16K?
- Does the target-first lookup reorder (hit-rate neutral) actually lower per-op-time? This needs an evaluation that reports cost.
- With 2-slot buckets the free-slot bypass is nearly used up. Do the remaining levers have to come from probing or the target tier instead of the stash?

## Per-policy notes

- **Stash tier:** keep LRU eviction and MRU-end demotion. The two-hit exit gate stays, with a bypass when the target has two or more free slots. Changing which items the gate applies to, or resetting counters on demotion, is close to neutral, so stop tuning there.
- **Target tier:** promotees must land at the MRU end. LRU-end landing makes the gate defeat itself. The trade-off between protecting residents and admitting new items depends on the trace (`hm_0` favours protecting residents, `web_0` favours admitting).
- **Probing tier:** largely unexplored. The single experiment (delaying entry) suggests `web_0` wants faster target→probing movement.
- **Trace/size behaviour:** 16K rewards stricter admission. `web_0` 128K rewards less filtering. `hm_0` 128K rewards placement and protection. Results usually split across cells and rarely move together.
