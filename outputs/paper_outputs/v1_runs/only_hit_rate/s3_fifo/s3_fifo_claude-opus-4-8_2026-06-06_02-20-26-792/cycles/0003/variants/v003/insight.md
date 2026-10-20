# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Tightening the small-to-main promotion gate from `freq > 1` to `freq > 2` was inert on this workload: hit-rate did not move at all. With v001's tiny `capacity/16` (~6%) small queue, entries are evicted FIFO too quickly to ever sit at the tail with `freq == 2` — they reach the `freq == 3` ceiling and promote under both gates, or never get a second hit and ghost under both — so there are no borderline two-hit keys for the stricter gate to filter. This refutes H03's premise that demanding stronger reuse evidence would purge displacing two-hit residents, and shows the `freq > 1` vs `freq > 2` distinction is a no-op once the admission queue is shrunk this far; the load-bearing lever remains partition sizing, not the promotion threshold.
