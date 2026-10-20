# Cycle 0007 Insight

## Cohort outcome

v001 and v002 branched from cycle-6 v002 (the counter-gated lineage). v003 branched from cycle-6 v001 (the rescue lineage). v001 merged rescue with the `borrowed` gate and shared one hash between them. It matched the best hit-rate exactly, but its put tail claim was refuted and its get tail grew. v002 added an exact `foreign` counter per normal bucket to gate the probing scan. It cleanly confirmed: get p50 and p99 both dropped, and hit-rate held. v003 replaced the `s0+1` stride with an independently hashed `s1`. It gained a little hit-rate, but the extra hash made every per-op metric worse.

## Patterns

- Exact occupancy counters only pay off when every mutation already knows how to classify the item. `foreign` cost nothing on put because the slot type and the demotion's `true_target` already carry that information. `borrowed` still rehashes in its removal path and shows a put cost. A counter is free when its inputs are free.
- Lossless gates stack. v002 runs the `s1` and probing gates together, so a full miss can now skip both neighbour probes. With no change to cache state, this is still the only reliable way to cut get latency.
- Rescue's cost is structural, not hashing. Sharing one hash fixed bookkeeping, but rescue still does extra normal-bucket work on every stash overflow. It also leaves normal buckets fuller, so promotions inside `get` evict and cascade more often. That pushes up both get and put tails. More hash trimming won't recover this.
- Stash overflow is barely clustered. Independent second choice adds little hit-rate, so the idle-capacity lever is close to exhausted on the stash side. Any extra full hash on a hot path costs more than it gains.

## Open questions

- Is v002's sharp put-p99 drop real? Nothing in the change explains it, so it needs re-measurement before anyone builds on it.
- Do the `foreign` gate's get savings cancel rescue's fuller-bucket scan cost when combined? Does `foreign` stay mostly zero once rescue writes into probing slots?
- Can rescue be trimmed (target-only, or rate-limited) to keep most of its hit-rate while dropping the cascade and tail cost?

## Continuation

Make v002 (both gates) the latency parent. Next, stack rescue onto it in a cheaper form: target-only, or only when the target slot is free without a scan. Also feed `borrowed` from caller-known `s0`/native flags so it never rehashes. Re-measure put p99 before trusting it. Drop independent-hash `s1` unless it is derived from spare primary-hash bits and computed only after `s0` is full. Drop further hash-trimming as a fix for rescue.
