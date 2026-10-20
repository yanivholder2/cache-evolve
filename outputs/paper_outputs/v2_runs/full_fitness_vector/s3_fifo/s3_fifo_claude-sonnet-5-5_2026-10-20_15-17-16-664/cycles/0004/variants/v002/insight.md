# Variant v002 Insight

## Helped

- None. No fitness dimension improved over the parent. `metadata-size` for `Entry` is unchanged, and the extra structure only shows up in the container.

## Hurt

- hit-rate: Re-admitting main-ghost keys straight into main with freq 1 did not pay off on workload-a. Keys evicted from main were mostly not worth protecting, so they took main slots from better residents. Splitting the ghost also meant keys that used to be re-admitted through the shared ghost path no longer get that path.
- per-op-time: `get` and `put` both got slower, and `put` tail latency rose the most. Every `put` miss now has a second linear ghost scan, plus removal and FIFO trimming on the new main ghost, which undoes the scan consolidation from v001.
- metadata-size: The container struct (`S3Fifo`) grew because of the extra main-ghost queue. The per-entry size did not change, so the summary dimension stayed flat.

## Takeaway

A freq head start for main-evicted keys on re-entry is not a free hit-rate gain. On this trace it cost hit rate, and the separate bounded ghost brought back the scan overhead that v001 had removed. Any future attempt to treat main-evicted keys differently needs an O(1) membership structure, or a way to reuse the existing ghost with a tag. It also needs evidence that those keys really are reused sooner than small-queue ghosts.
