# Variant v002 Insight

## Helped

(No dimension improved. The fitness vector reports only `hit-rate`, and it moved slightly the wrong way.)

## Hurt

- hit-rate: Ghost-readmitted keys now enter `main` with `freq = 1`, so `evict_main` decrements them and pushes them back to the head instead of evicting them. Each one gets a full extra lap through the 1844-slot main queue even if it is never touched again. Main stays fixed in size, so every extra lap pushes some other entry out. The second-chance loop also has to rotate past these credited entries before it reaches a `freq == 0` victim. Along the way it decrements the credits of keys that graduated from small. On workload-a, the ghost hits that would have been re-accessed within one main lap don't make up for what it costs to keep the ones that weren't.

## Takeaway

On workload-a, a single ghost hit is weak evidence of reuse. Many re-admitted keys are one-off returns at a long reuse distance and don't recur within the next main lap. Starting them at freq 0, as v001 does (a "prove it again in main" rule), works better than giving them a free second chance, because main capacity spent on that credit comes out of keys that have already shown reuse. This variant therefore suggests that ghost re-admission is not the bottleneck at this 10% split. Future changes to ghost handling should stay at freq 0 or be more selective, for example by crediting only repeat ghost hits.
