# Variant v003 Insight

## Helped

(none: no hit-rate cell moved by more than rounding noise)

## Hurt

(none: same reason)

## Takeaway

The main prediction did not come true: hm_0 at 16384 was expected to gain most, but it did not move. Possible reasons: the redirect rarely fires, or the stash slot it frees goes to an item that is not reused before it is evicted, while the ghost-return path makes up for the cold resident it drops. When a ghost-hit bypass cascades, it makes no difference to hit-rate whether the displaced counter-0 resident goes to the stash or to the ghost. So stash churn from cascades is not what limits v002 at 16384. That fits v002's conclusion that target-tier capacity, not stash placement, is the limit. The hypothesis also predicted lower per-op-time on the cascade path and asked for per-op-time and metadata-size, but the fitness vector reports only hit-rate. So the expected cost benefit, the only remaining reason to keep this change, has not been measured. Note: this insight is based on the hypothesis and the fitness vectors. The overlay sources could not be listed in this session, so it is not confirmed that the change landed as specified. If it did not land (for example, the flag was never set on the ghost-bypass call site), that would also explain a null result.
