# Cycle 0009 Insight

## Cohort outcome

All three variants came from cycle 8's v003 (`small` at an eightieth, ghost at thirteen-eightieths). Each ran one of the checks cycle 8 asked for:
- **v001** ported cycle 8's `small`-to-`main` credit carryover together with `MAX_FREQ = 7`. It won, by a sliver that could be noise.
- **v002** added a short ghost fed by `main` evictions. Hit rate was nominally flat.
- **v003** ported the long-pending fingerprint ring for the ghost. Hit rate was unchanged, as intended. Its real target, `metadata-size`, was not scored.

## Patterns

- **Each experiment closed its line of work rather than opening one.** Carryover mostly overlaps with the smaller `small`, as predicted. The `main`-fed ghost found almost nothing to recover. So `main`'s eviction of zero-credit keys already rejects keys nearly correctly, and "feed the ghost differently" is answered.
- **On workload-a at 2048, every admission and retention lever we know of is close to used up.** These include: the `small`/ghost split, credit caps, carryover, scan order, entry credits, and now ghost sources. The remaining gains are within noise on one trace and one size.
- **The fingerprint ring is policy-neutral.** It reproduces the parent's admission decisions and removes per-slot key clones. It is a safe base for all later variants. Whether it wins on `metadata-size` depends on the evaluation, not on more code.
- **The variant insights keep proposing an idea that has already failed.** Both v001 and v002 again point at giving ghost-admitted keys a starting credit. That idea lost in cycle 7 and was dropped in cycle 8. Treat it as rejected.
- **The bottleneck is now the evaluation, not the policy.** The fitness vector still has only `hit-rate`. A second cache size or second trace has been asked for since cycle 4 and has still not appeared.

## Open questions

- Does the fingerprint ring actually lower `metadata-size`? Does any S3-FIFO variant come close to the baseline on it, or on `per-op-time`?
- Do the carryover and split gains survive at a second cache size? Do they survive on a second trace?
- In v001, which part caused the sliver: the cap or the carryover? This only matters if it holds up beyond workload-a at 2048.

## Continuation

Use v001 as the parent and port v003's fingerprint ring onto it, as one cheap combined winner. Stop proposing hit-rate tweaks on workload-a at 2048; that search is saturated.

Drop these lines of work:
- the `main`-fed ghost
- credit work on the `small` path
- ghost-arrival starting credits, which lost when tested in cycle 7 and should not be retried

The next cycle should focus on getting `metadata-size` and `per-op-time` scored, and on running at a second size or trace. If the harness cannot do that, flag it rather than inventing new knobs.
