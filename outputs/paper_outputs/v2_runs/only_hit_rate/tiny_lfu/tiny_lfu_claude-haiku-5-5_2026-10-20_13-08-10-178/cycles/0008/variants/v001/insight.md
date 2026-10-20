# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Changing the probation-victim tie rule from strict `>` to `>=` made every equal-estimate newcomer displace the probation head, which is the oldest cold entry, so under a conservative sketch the probation segment churns in near-FIFO order on ties and loses entries that might have been re-referenced.

## Takeaway

Admitting ties against probation victims is a net negative on workload-a at size 2048: the strict comparison was keeping older cold entries alive when the sketch cannot separate them, and H01's assumption that tied keys are interchangeable ignored that the tie is broken purely by queue position. The effect is small and comes from a single trace and size, so it is weak evidence against the tie rule in general, not a settled result.
