# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The ghost-list override in `put()` admits any key that the frequency gate rejected but that was bypassed recently, without rechecking `incoming > victim`, so a recurring key with a low counter can displace a resident whose counter is higher. Every bypass also writes to the ghost list, so one-shot keys push out the ghosts that the override depends on.

## Takeaway

On workload-a, a second-miss signal does not improve on the frequency gate it sits behind. The ghost admit path skips the counter comparison, so recurrence alone is allowed to evict frequency-protected residents, and this variant's drop suggests that recurrence within a one-slot-per-capacity window is a weaker predictor of reuse than the counter. A version that keeps the counter check on the ghost path, or keys ghost entries on a higher recurrence threshold, would isolate whether the loss comes from the override itself or from the ghost list's churn.
