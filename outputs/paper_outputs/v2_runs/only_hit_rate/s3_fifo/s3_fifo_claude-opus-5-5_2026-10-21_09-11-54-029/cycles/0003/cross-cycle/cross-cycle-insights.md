## Recurring wins

- Protecting keys that have proven they get reused pays off on workload-a. Every Winner so far (shrinking the small queue twice, raising the main frequency cap) gave more room or more retention to keys with demonstrated repeat reuse.
- Shrinking the small queue (to 10%, then to 5%) gave consistent but shrinking hit-rate gains. The gain comes from a larger main queue, not from a larger ghost.
- Main-queue retention (the frequency cap) is a newer axis. It gave a modest gain, and it can be combined with the partition changes.
- Single-change variants with a clear lineage gave attributable results. Variants that bundled two changes (cycle 0001 v002) could not be attributed.

## Recurring failures

- Making re-admission or graduation more generous keeps losing hit-rate. Examples: a lower graduation threshold, a larger or decoupled ghost, a freq credit for ghost re-admits. On this workload, one extra reuse signal is weak evidence, and the main capacity it takes comes out of keys with proven reuse.
- Making re-admission much stricter by inserting at the main tail also lost. Front-of-main, freq-0 re-admission sits between those two and has held up.
- Per-op-time and metadata-size have gone unmeasured for three cycles in a row, even though every Continuation asked for them. The incumbent lineage keeps lengthening its linear scans (a larger ghost, a higher cap), so the unmeasured cost keeps growing.
- Carried-forward actions get dropped. The indexed-structure port took until cycle 3, and combining it with the policy Winner has still not been done.

## Open questions

- How much per-op-time does the behaviour-exact indexed port save, and how much metadata-size does it add? Does it stack cleanly with the current policy baseline (5% split, cap of 7)?
- How long do eviction sweeps get under the higher frequency cap? Does raising the cap above 7, or setting the graduation threshold separately from the cap, add hit-rate?
- Can a re-admission probation window counted in accesses, somewhere between tail and front insertion, beat front insertion? Would that make a larger ghost useful?
- Do the partition and cap gains hold on other traces and cache sizes, or are they specific to workload-a?
- Was cycle 0001 v002's regression caused by the lower threshold or by the frequency reset? This is low priority.

## Per-policy notes

- S3-FIFO: the current baseline is a 5% small queue, a main frequency cap of 7, a ghost sized with main, and front-of-main freq-0 ghost re-admission. The partition curve looks close to flat below 10%, so further shrinking is likely to give diminishing returns. The ghost now does most of the filtering.
- S3-FIFO: the indexed slab/index structure is verified as behaviour-neutral and is ready to adopt. Its cost and benefit are unmeasured.
