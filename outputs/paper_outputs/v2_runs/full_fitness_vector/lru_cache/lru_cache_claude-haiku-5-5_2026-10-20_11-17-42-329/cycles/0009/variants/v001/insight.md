# Variant v001 Insight

## Helped

- hit-rate: Raising the protected share from 4/5 to 9/10 of capacity lets rescued keys stay in the protected segment longer before demotion, which helps the reused keys in workload-a at size 2048 survive the probationary scan traffic.

## Hurt

- per-op-time: The `put` tail latency grew. The only code change is the larger `protected_cap()` value, so a fuller protected segment is the likely source: more misses at capacity land on the referenced-probationary-head path, which calls `demote_prot_head` and its spare-and-retry branch more often. This is an inference from the code, not a profiled result.

## Takeaway

In this SLRU, the protected share is the lever that trades probationary room for retained reused keys: moving it from 4/5 to 9/10 kept more reused keys on workload-a, but it also leaves the probationary segment thin, so the `put` miss path does more demotion work. Metadata size and `get` latency did not move, because the change touches only the capacity split and not the node layout or hit path.
