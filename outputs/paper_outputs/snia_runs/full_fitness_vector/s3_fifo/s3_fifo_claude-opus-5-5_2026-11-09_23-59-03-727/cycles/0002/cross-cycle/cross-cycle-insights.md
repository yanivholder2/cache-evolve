## Recurring wins

- Structural changes that keep the policy fixed and improve complexity (slab plus hash indexes instead of linear scans) are the only per-op-time gains the diffs actually explain. They also leave hit rate exactly unchanged, so the policy can be evaluated separately from the implementation.
- On hm_0, giving reused keys more main residency or history pays off reliably: main-victim ghosting, restoring the demotee ghost, and promoting after a single hit all helped there in both cycles.

## Recurring failures

- No single-axis capacity or history tweak has beaten baseline yet. Fixed small-queue ratios, ghosting main victims, adaptive small/main splits, split ghost regions and a looser graduation threshold each moved the trade-off between hm_0 and web_0 (especially web_0 at the large size) without a net gain.
- Resizing or partitioning the ghost has not fixed web_0 at the large size. Restoring the demotee ghost in full did not rescue it, which rules out ghost-slot competition as the cause there.
- Bundling two mechanisms into one variant (for example the threshold change together with the freq reset on promotion) leaves the result impossible to attribute. This has repeatedly blocked progress on finding web_0's mechanism.
- Per-op-time changes that the diff can't explain have appeared in both cycles. They are environmental noise and deserve no timing credit.
- The ARC-style adaptive split, as implemented, regressed. Its two signals were unbalanced and its ghost was not tied to main's live size.

## Open questions

- What actually drives web_0's loss at the large size? The current leading suspects are freq-0 main-victim ghost hits re-admitted straight into main, and the freq reset on promotion. Test each in isolation.
- Does restricting main-victim ghosting to victims with a nonzero frequency, or re-admitting ghost hits with only partial credit, keep hm_0's gain without hurting web_0?
- Can an adaptive split work with normalized signals, a ghost that tracks main's live size, and a smaller step at large capacity?
- The metadata-per-entry metric misses heap growth from ghost keys and hash maps. Should heap bytes per resident key be tracked alongside it?
- Timing comparisons need a baseline re-run on the same machine before anyone trusts them.

## Per-policy notes

- S3-FIFO (the only policy explored so far): the hm_0 and web_0 trade-off has been stable for two cycles, so treat it as the main thing to resolve rather than as noise. Ghost capacity behaves as a coupled, zero-sum variable. Future variants should build on the slab-plus-hash base and change one mechanism at a time.
