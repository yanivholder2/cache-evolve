# Variant v003 Insight

## Helped

- per-op-time: Deleting `self.record(key_hash(key))` from `get()` leaves a hit as a map lookup plus unlink and push_back, so `get` no longer pays for a sketch hash on every read; its p50 and p99 both moved down against the parent.

## Hurt

- hit-rate: Reads no longer add frequency to the sketch, so keys that are only read lose admission weight against the resident victim, and the admission gate now sees a less complete picture of recent use. The drop is small but it is the trade the hypothesis accepted.
- per-op-time: `put` p50 and p99 both rose slightly, although the put path is textually unchanged; the diff does not explain this, so it is more likely run-to-run noise than a real cost of the change.

## Takeaway

Feeding the admission sketch only from `put()` removes the hash from the read path, which is the cost the parent flagged, at the price of letting read-only popularity go uncounted. The only measured signal is a small hit-rate loss on one trace at one capacity, so the next check is whether that loss holds on other workloads, especially read-heavy ones where reads carry most of the frequency signal.
