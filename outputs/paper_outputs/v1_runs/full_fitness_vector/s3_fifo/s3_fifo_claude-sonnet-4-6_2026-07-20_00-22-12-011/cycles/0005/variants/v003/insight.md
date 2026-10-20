# Variant v003 Insight

## Helped

- metadata-size: Moving the value out of `Entry` and into `resident_map` reduced `Entry` to a key-only struct (plus zero-sized `PhantomData`), halving its per-entry footprint compared to the parent where `Entry` held both key and value.
- per-op-time: Replacing the O(n) `position()` scans in `get()` and `put()` with a single `HashMap::get_mut` call eliminated the dominant latency cost in the parent — particularly on the put update path, which previously scanned both `small` and `main` deques to locate a matching key before updating its value.
- hit-rate: The marginal gain comes from removing `SCAN_CAP=16`, restoring the unbounded second-chance scan that the cycle 4 insight confirmed terminates quickly in practice on Zipfian workloads and avoids forced eviction of still-hot entries.

## Hurt

(none)

## Takeaway

Consolidating value and frequency into a single `resident_map: HashMap<K, (V, u8)>` simultaneously eliminated O(n) deque scans as the latency bottleneck for both `get()` and `put()`, allowed `Entry` to shed its value field (halving its size), and removed the two parallel freq `VecDeque`s from the struct. The `put()` update path saw the largest improvement: where the parent scanned both `small` and `main` for a key before overwriting the value, `resident_map` reduces that to a single hashmap probe. The metadata-size reduction is a structural consequence of the same refactor rather than a separate optimization.
