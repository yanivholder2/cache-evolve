# Variant v002 Insight

## Helped

(No measured dimension improved. H02 predicted that graduating on a single probation hit (`freq >= GRADUATE_MIN_FREQ`, with `GRADUATE_MIN_FREQ = 1`) would raise `hit-rate` on workload-a. It did not.)

## Hurt

- hit-rate: This dropped very slightly, a change small enough that it is effectively flat. In v001, a key hit once in small was ghosted. If it was missed again, it came back into main at freq 0. In v002 it goes straight into main and carries freq 1. Each of those keys costs an `evict_main` call, which pushes out a main resident. Its carried freq of 1 also lets it survive one extra CLOCK pass. Both effects mostly cancel out the misses that v001 paid on ghost re-admission, so admitting earlier gains nothing and costs a little.

## Takeaway

This is the clean, isolated test that cycle 0001's v002 could not give. Only the graduation threshold changed: freq is still not reset, and `MAX_FREQ = 7`, the 5% split, ghost behaviour and the slab/index structure all match v001. Under those conditions, lowering the threshold from `freq > 1` to `freq > 0` does not improve `hit-rate` on workload-a at size 2048. On this trace, the ghost queue already recovers once-hit small keys about as well as promoting them directly does. So the threshold is not a lever worth spending more cycles on at the 5% split. `GRADUATE_MIN_FREQ` stays a named constant, but there is no sign that moving it in either direction would matter more than the ghost path already does.
