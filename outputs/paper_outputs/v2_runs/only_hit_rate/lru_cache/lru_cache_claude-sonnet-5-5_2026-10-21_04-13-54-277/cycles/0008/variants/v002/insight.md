# Variant v002 Insight

## Helped

- (none): the single fitness dimension did not improve over the parent.

## Hurt

- hit-rate: Deferring the admission check through a ~1% window gave a slightly lower hit rate than v003's entry-time sketch gating. The window is too small to hold bursty keys long enough to be reused. It also takes about 1% of protected capacity.

## Takeaway

On workload-a at size 2048, moving the sketch admission decision from entry to window eviction did not recover any keys that v003 wrongly rejected. The window is so short that most keys are evicted from it before a second hit, so the window plus candidate-vs-head comparison adds complexity for a marginal loss. Entry-time gating looks close to sufficient for this trace. A larger window, or a different fix for the weakness the hypothesis targeted, would be needed to test the "deferred decision" idea fairly.

I could only read the hypothesis and the fitness vectors. The tools for listing and reading the overlay source files were unavailable, so this interpretation comes from the hypothesis text, not from a diff of the code.
