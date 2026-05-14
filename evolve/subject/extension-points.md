# Dash Extension Points

Stable interfaces where Level 3 Variants plug in to the Dash subject cache.
Each extension point is a location where a Variant can provide a new
implementation without modifying files in `src/`.

| Extension point                | Status                   | Where to implement                                                                        |
|--------------------------------|--------------------------|-------------------------------------------------------------------------------------------|
| `Segment` trait                | **Realized** — see below | Outside `src/`; implement `Segment<K, V>` and pass to `Dash::new`                         |
| `EvictionPolicy` enum          | Planned                  | Not yet a trait; adding a new policy requires editing `src/shared/eviction_policy.rs`     |
| Probing function               | Planned                  | Hardcoded as `(target + 1) % n` in `SegmentCore`; not yet parameterised                   |
| Cascade / demotion-chain order | Planned                  | Hardcoded inside promotion strategy impls; not yet a hook                                 |

---

## `Segment<K, V>` — realized

**Trait:** `dash::dash::segments::Segment`
**Source:** `src/dash/segments/mod.rs`

`Dash<K, V, S: Segment<K, V>>` is generic over the segment strategy. To author
a Level 3 Variant:

1. Define a struct outside `src/` (e.g., in the Variant's `impl/` directory).
2. Implement `Segment<K, V>` for it using only the crate's public API.
3. Implement the three required methods: `core`, `core_mut`, and `get`.
   **Level 3 `get` constraint:** `get_index`, `buckets`, and `stash_buckets` are
   `pub(crate)`. Compose existing promotion or combined helpers rather than
   traversing buckets directly; genuinely novel lookup logic requires Level 4.
4. Instantiate `Dash::new(num_segments, || MySegment::new(&settings))`.

**Contract:** the trait's doc comment in `src/dash/segments/mod.rs` is the
authoritative source. Read it before implementing.

See `tests/segment_external_fixture.rs` for a working external implementation
compiled against the public API.

---

## Planned extension points

The three remaining extension points are not yet stable traits. Tracking tickets
will be filed when the framework's research loop reaches the point where it
would benefit from varying these axes independently of the Segment strategy.
