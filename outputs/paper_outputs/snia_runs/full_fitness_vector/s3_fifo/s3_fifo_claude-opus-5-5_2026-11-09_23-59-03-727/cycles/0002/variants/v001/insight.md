# Variant v001 Insight

## Helped

- hit-rate: hm_0 at the small size, one of v002's two regressions, recovered and now beats the parent. This fits the hypothesis: `ghost_small` again holds demotees at the full `main_size` bound, so a reused small-queue key no longer loses its ghost slot to main victims and can go straight back into main.

## Hurt

- hit-rate: web_0 at the large size got worse instead of recovering, and that was the hypothesis's own test case. So the shared-ghost coupling does not explain web_0's loss there, and per the hypothesis's own test, ghosting main victims at all looks like the problem on that trace. Each extra ghost hit brings back a freq-0 key straight into main, which forces another `evict_main`.
- hit-rate: hm_0 at the large size and web_0 at the small size slipped a little. In v002 a main victim could take up to `main_size` shared ghost slots. `ghost_main` caps main-victim history at `main_size / 2`, so the longer-distance main-victim reuse behind v002's gain on hm_0 at the large size is partly forgotten.
- metadata-size: The summary did not move because `Entry` is still 24 bytes. The `S3Fifo` struct grew by one `VecDeque` header and one `usize`, though, and total ghost key storage can now reach 1.5× `main_size` instead of 1×.

## Takeaway

Splitting the ghost moved the hit-rate gains between workloads instead of adding to them. Restoring the demotee ghost repaired hm_0 at the small size, but web_0 at the large size got worse even with demotee history fully restored. That rules out ghost-slot competition as the cause of that loss and points at main-victim ghosting itself, at least for web_0. Halving main-victim history also gave back part of hm_0's large-size gain. A next step is to ghost only main victims that once had a nonzero frequency, not to resize the regions again. per-op-time improved even though `put` misses now scan up to 1.5× as many ghost keys and `get` is unchanged. The code can't explain that, so treat it as noise from the measurement environment.
