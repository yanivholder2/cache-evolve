# Variant v002 Insight

## Helped

*No dimensions improved.*

## Hurt

- per-op-time: Occupancy-responsive boost calculation (division, squaring, floating-point min) adds measurable latency overhead at ghost-hit re-admission, making gets 31% slower and puts 12% slower than the fixed +2 boost in v003.

## Takeaway

The hypothesis claimed occupancy-responsive re-admission would unlock dual-metric gains while maintaining v003's latency wins. Instead, the adaptive boost mechanism—despite being placed in a cold path (ghost hits)—introduces floating-point arithmetic overhead that directly harms latency. This variant reveals that sophistication in re-admission signaling does not come free: the fixed +2 boost strategy in v003 is simpler, cheaper, and delivers better latency without sacrificing hit rate. Sometimes correctness beats adaptivity.
