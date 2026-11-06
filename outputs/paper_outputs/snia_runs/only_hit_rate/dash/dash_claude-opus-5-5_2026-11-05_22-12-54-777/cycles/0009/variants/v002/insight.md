# Variant v002 Insight

> Note: this insight is based on the hypothesis (H02) and on both fitness vectors. The overlay source and manifest.toml could not be read in this session (no directory listing was available, and the expected file paths did not exist). So the diff itself was not checked against the hypothesis's Changes list.

## Helped

- hit-rate (web_0, size 131072): This is the cell the hypothesis targeted, and it moved in the predicted direction. But the gain is at noise level, the same size as the moves in cells the change was not aimed at. So sending returners to the probing bucket's cold tail slot rescues very few of the ghost-hit returners that the target gate refuses. Most of them still go to the stash.
- hit-rate (web_0 16384, hm_0 16384, hm_0 131072): All three moved up slightly, at noise level. This matches the prediction that a fallback gated on cold slots displaces no warm resident. The new tier did no harm, but it also had almost no effect.

## Hurt

(none: no cell regressed)

## Takeaway

When the target's 8-slot tail window is all warm, the probing neighbour's window is almost always all warm too. Both buckets share the same segment and the same load, so a second cold-slot check finds almost nothing extra. Spreading placement to the neighbour does not undo c0008's loss on web_0 at 131072. That loss comes from the gate refusing the returner, not from a lack of cold slots nearby. To recover it, the admission rule itself has to change, for example by comparing the returner's credit with the coldest counter in the window. Adding more placement targets under the same "counter-0 only" rule will not help.
