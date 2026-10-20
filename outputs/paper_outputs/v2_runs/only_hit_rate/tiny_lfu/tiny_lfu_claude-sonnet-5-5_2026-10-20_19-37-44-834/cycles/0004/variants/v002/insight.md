# Variant v002 Insight

## Helped

- hit-rate: No movement versus the parent. The hypothesis grew the protected segment to about 90% of main, but the fitness vector is identical to the parent's, so the larger protected share gave no measurable benefit.

## Hurt

- No fitness dimension regressed. Only hit-rate is reported, so any cost from a larger protected segment (for example more demotions into a smaller probation segment) is unmeasured. I could not read the overlay source in this session, so this comparison rests on the hypothesis text and the fitness vectors.

## Takeaway

Moving the protected/probation split from about 80% to 90% of main did not change hit-rate on workload-a at size 2048. The split is therefore not the bottleneck on this trace at this size, or the extra protected capacity is not being used. The parent's guess that segment sizing is a likelier source of gains than the admission duel was not supported. The next candidates are the sketch (width, aging, or reset period) or the window size, which has not been re-checked.
