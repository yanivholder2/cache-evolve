# Cycle 0005 Insight

## Cohort outcome

All three variants came from cycle 4's v001 (ghost at a quarter of capacity). Each ran one of the experiments cycle 4 asked for:
- **v001** halved the ghost again, to an eighth of capacity. It won, with a smaller gain than the previous halving.
- **v002** swapped the ghost to a fingerprint ring at the same window size, changing only the representation. Hit rate was exactly unchanged.
- **v003** added TinyLFU admission on every route into `main`, compared against the real CLOCK victim. Its gain was only marginal.

## Patterns

- **Shrinking the ghost still pays, but by less each time.** Three halvings in a row have helped, each less than the one before. The ghost window is now about the size of the probation queue, so the next halving would make the ghost smaller than `small`. That probe really asks whether the ghost-into-`main` fast path is worth having at all. Every result so far comes from one trace at one cache size.
- **Frequency admission is near its ceiling on this design.** v003 fixed both flaws found in cycle 4: the leak through `small` and comparing against the wrong entry. It still gained only a hair. The small-queue filter plus a short ghost already turns away most of the low-frequency candidates that TinyLFU is meant to block. So the leak and the wrong target were not what held the sketch back.
- **The representation track finally ran, and as expected it left hit rate alone.** v002 confirms the fingerprint ring is a safe base for later ghost work. The stale-erase fix made no measurable difference, because the case it guards against almost never happens. Whether the ring is a net win isn't known yet: it removes key clones but adds an extra hash pass on each miss.
- **Measurement is still missing, for the fifth cycle running.** This cycle makes the gap costly. One variant's only claimed benefit sits entirely in the unmeasured dimensions. Another adds a per-access sketch whose memory and time cost could outweigh its tiny hit-rate gain. Neither can be judged.

## Open questions

- At a sixteenth of capacity (ghost smaller than `small`), does hit rate finally turn over, or does the ghost path turn out to be dispensable?
- Do the ghost-size gains hold at a second cache size or on another trace, or do they just fit workload-a's reuse distances?
- Does the fingerprint ring actually cut `metadata-size` and `per-op-time`, given its extra hashing? Is the sketch's per-access cost close to zero?

## Continuation

Use this cycle's v001 as the parent. Run the sixteenth-of-capacity probe, and if at all possible run it at a second cache size as well, before trusting further tiny ghost-size gains. Port v002's fingerprint ring onto that parent as a policy-neutral base. Keep its hashing cost in mind: one hash per key, used for both the fingerprint and the lookup, if that can be arranged.

Drop TinyLFU and other frequency-admission gates unless cost measurements show they are nearly free. Look for hit-rate gains outside admission filtering. Measuring `metadata-size` and `per-op-time` is now blocking: without it, the representation and sketch results can't be judged.
