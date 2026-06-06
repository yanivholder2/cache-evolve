# Variant v003 Insight

## Helped

- per-op-time: Collapsing `demote_from_normal_bucket`'s foreign-resident branch from a two-`put_at` retry-plus-cascade (re-insert into true target, then cascade that target's LRU victim to stash) down to a single unconditional `put_at(Slot::Stash, ...)` removed the worst-case double-hop demotion that cycle 0007 located as the put-p99 bound, cutting put-p99; the get tail moved down too but within noise, since the demote callers are on the put path.

## Hurt

- hit-rate: Dropping the one-more-chance retry was not perfectly free as predicted — foreign probing residents now fall straight to stash instead of getting re-homed in their true target, and that conflict-mitigation step was salvaging a small number of hits, so hit-rate slipped slightly below the parent rather than holding.

## Takeaway

The one-more-chance retry is overwhelmingly latency overhead under ClassicLRU — removing it buys the predicted put-p99 win — but it is not pure overhead: the foreign-resident re-home was rescuing a handful of accesses, so this is a real (if tiny) latency-vs-hit-rate trade, not the strictly-free trade the conflict-insensitivity insight forecast.
