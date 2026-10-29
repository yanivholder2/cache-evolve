# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle 0007 v003, the window-throttled swap lineage. v001 replaced the swap with a single outright eviction of the probing resident. v002 doubled `PROBE_WINDOW`. v003 changed stash victim choice to prefer never-hit entries. All three are listed as Winners, but only v001 brought a useful result: a large-cache hm_0 hit-rate gain, with its latency verdict lost to another uniform shift. v002's hit-rate gain was negligible, and it was bought with a real get-latency cost. v003 lost hit-rate in every cell.

## Patterns

- The eviction-instead-of-swap result now holds on two lineages. Dropping the probing evictee cost nothing under the cascade in cycle 0007, and under the window throttle it cost nothing and slightly helped large-cache hm_0. Keeping the demoted resident is worthless. Freeing a slot is at least neutral and possibly helpful.
- `PROBE_WINDOW` is a latency throttle, not a hit-rate lever. Widening it let many more hottest hits into the costly swap, and the median rose as well as the tail, so the swap is not a rare event once admitted. The parent's window already sits near the useful end of that trade-off. The hm_0-small dip has some other cause.
- The stash's LRU-position eviction is already the right scan-resistant policy. Protecting one-hit stash entries filled the stash with keys that are rarely reused and starved fresh arrivals before the gate could judge them. Small caches were hurt most, and web_0-large still did not move. This was the first test of "who leaves the stash", and it shows the lever cuts the wrong way unless the protection ages out.
- Any added scan or check shows up exactly where it runs. v003's put cost appeared on the stash-full put path, and v002's cost appeared on the get path. This is consistent with the recurring "work on a path shows on that path" lesson.
- Noise is still unmeasured. v001's near-uniform slowdown, including the median of an untouched put, has the same signature as cycle 0007 v002. This is the fourth cycle running with an unexplained uniform shift, and still no identical-baseline re-run has been done. As before, no variant could read its overlay source. v002 also flags a possible implementation side effect, a per-get bound computation, that could inflate its median.

## Open questions

- Does v001 actually improve latency once it is timed against v003 on the same host in the same session?
- What is holding back hm_0-small on the swap lineage, if the window is not the cause?
- Would an aging bound on stash protection, where one old hit does not shield an entry for long, recover any of the web_0-large gain, or is the stash-exit lever dead?
- Does narrowing the window on v001's eviction lineage cut latency further without costing hit-rate?

## Continuation

Adopt v001 (window throttle plus single eviction) as the baseline on hit-rate grounds. Before trusting any latency verdict, run v001, v003 from cycle 0007, and an identical copy of the parent together on one host. This request has gone unmet for seven cycles, and it has to be a hypothesis of its own in the next cohort. Drop window widening. Drop unbounded stash-victim protection. Look for hm_0-small and web_0-large hit-rate in the arm's mechanics or in admission, not in throttle tuning.
