# Variant v002 Insight

## Helped

- hit-rate: Deferring a stash-hit promotion until the key's second stash hit keeps one-touch stash entries out of their target buckets, so they no longer displace existing target residents on first use; on workload-a this gave a small net gain.

## Hurt

- per-op-time: Get tail latency worsened, because the first-hit bit check and its update now run on every stash hit, adding work to the stash branch instead of removing the promotion write as the hypothesis predicted.
- per-op-time: Put tail latency worsened, plausibly because entries stay in the stash longer under deferral, so insert paths that fall through to the stash see fuller slots; this is an inference from the design and was not isolated by measurement.

## Takeaway

The hypothesis's latency claim is not supported here: the aggregate get median is flat, and the stash-heavy trace the experiment called for is absent from the vector, so the only clear signal is a small hit-rate gain alongside a worse tail. The lever is keeping single-touch stash entries out of target buckets, and the tail regressions suggest the extra first-hit state on the stash path costs more than the deferred write saved, so the change should be re-measured on a stash-dominant trace before it is credited.
