# Variant v002 Insight

> Note: this insight is based on the hypothesis (H02), both fitness vectors and the parent's insight. The overlay source and manifest.toml could not be found in this session: there was no directory listing, and the guessed paths under `attempts/01/overlay/` did not exist. So I could not check that the refused-bit and strike-credit path was implemented, or reached, as the hypothesis describes.

## Helped

- hit-rate (web_0, size 131072): This is the cell the hypothesis targeted, and it moved in the predicted direction. But the gain is at noise level. So few keys took the full path the strike credit needs: refused by both gate tiers, sent to the stash, dropped back to the ghost, then returned again while still within ghost reach.
- hit-rate (hm_0, size 131072): Up at noise level. This fits with the strike gate (`min_ctr <= 1`) rarely firing, and with it displacing only cold residents when it does fire.

## Hurt

- hit-rate (hm_0, size 16384): Down at noise level. This is consistent with the hypothesis's claim that the strict first-return gate on small hm_0 is preserved. It does not show that second-returners displacing counter-1 residents cost anything that matters.

## Takeaway

Admitting a returner on its second return barely recovers c0008's web_0 loss at 131072. So the valuable web_0 returners are not mostly keys that get refused, dropped and then come back again within ghost reach. Either they return only once within the ghost's capacity-sized window, or the stash keeps them long enough that they never fall back to the ghost. Per-key repeated-refusal evidence is therefore too rare a signal to separate the two sizes. The next step should first measure how often the `refused == 1` branch fires on web_0 at 131072, before changing the strike credit or threshold.
