# Variant v002 Insight

## Helped

- hit-rate: All four trace/size cells improved, with the larger gains at the 16384-entry size. The new sketch adds TinyLFU aging: a `#[cold] reset` halves every 4-bit counter after `SAMPLE_FACTOR * capacity` additions, and counters saturate at 15. Old popularity now decays, so admission tracks the recent working set. The residency layer did not change, so the whole gain comes from the sketch swap.
- metadata-size (sketch heap component): The sketch heap at cap 1024 dropped to an eighth of its previous size. Counters are now 4-bit nibbles packed into 64-byte `Block`s, where before each counter took a much wider slot. The headline node-size figure (`Node<u64,u64>`) is still 24 bytes, so the summary value did not move.

## Hurt

- per-op-time: Essentially flat. The sketch went from four scattered row accesses to one cache line per key, and that did not recover any of v002's latency gap. Get p99 got slightly worse, likely from the extra saturation check and the sample counter on every `increment_hashed`, but that change is within plausible noise.

## Takeaway

Moving v001's cache-line-blocked aging sketch onto the 24-byte u32-link layout worked. The hit-rate and sketch-memory gains carry over without enlarging the node. The latency result settles H02's open question: v002's regression did not go away when it got v001's sketch, so it does not come from the old sketch's scattered memory access. It stays with the u32-link residency layer, so the next fix to try is codegen in `unlink`/`push_back`/`on_hit` (bounds checks, mask/widen sequences), not the sketch. One more check: hit-rate should equal v001's exactly if the copied sketch behaves identically, and it lands close to that figure but not on it. A same-run comparison should confirm the copy matches v001 exactly.
