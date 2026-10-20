# Variant v003 Insight

## Helped

(none) No dimension improved in a meaningful way. The `put` p99 dropped slightly and the `get` p99 rose slightly. This fits run-to-run noise: the change adds only one `vbit_set` on the ghost-hit branch of `put`, writing to a bitset word that the slot-reuse `vbit_clear` has just touched.

## Hurt

- hit-rate: Setting VISITED on ghost returners (`self.vbit_set(idx)` after `push_back_main`) dropped hit-rate below the parent. That is under the hypothesis's own 0.7314 failure threshold, so the hypothesis is refuted. A returner already gets roughly one full SIEVE lap of protection just by entering at the main tail. The pre-set bit doubles that to about two laps. So every returner that turns out to be one-shot now holds a main slot for an extra pass. While the hand spends its clearing pass on these returners, it evicts something else instead: older main nodes whose bit is clear. These include keys promoted from small, which enter main with VISITED cleared even though they earned a real hit.

## Takeaway

On workload-a, a ghost hit is enough evidence to skip probation, but not enough to also count as a main-queue hit. Giving returners pre-set credit creates an asymmetry. A ghost returner, which has shown one reuse across an eviction, now outranks a promoted small key, which was actually hit while resident and loses its bit on promotion. The hand then sacrifices those promoted keys to protect returners that often do not come back within two laps. Cycle 0007 found that weakening admission (placing returners at the hand) also lost hit-rate. Together with this result, the current admission strength looks like a local optimum: plain tail insertion with VISITED clear. If returners are to be treated differently from promoted keys, it should come from evidence gathered while the key is resident, not from the ghost match alone.
