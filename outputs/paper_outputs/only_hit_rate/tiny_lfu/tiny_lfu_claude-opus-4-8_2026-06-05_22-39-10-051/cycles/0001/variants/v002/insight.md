# Variant v002 Insight

## Helped

- (none): no fitness dimension improved over the parent.

## Hurt

- hit-rate: the `capacity / 100` window (only ~20 slots at size 2048) is too
  small to shelter bursty newcomers long enough to accrue sketch frequency, so
  the unconditional window admission plus deferred main-region contest ended up
  marginally worse than the parent's direct sketch gate on workload-a.

## Takeaway

A 1%-of-capacity admission window does not move hit-rate on this workload:
the window evicts or contests candidates faster than they can build the
frequency the main-region gate demands, so W-TinyLFU's burst protection
collapses back to roughly the baseline's behavior. The window-sizing fraction,
not the window mechanism itself, is the lever that needs tuning.
