# Variant v001 Insight

> Note: I could not open the overlay source in this session. Directory listing and search tools were unavailable, and the source and manifest paths I guessed did not exist. The mechanism below comes from H01's stated change: remove the bimodal "1 in `BIP_PERIOD` to MRU" branch and its counter state, so every fresh key or gate-failing ghost hit enters at stash LRU. I did not check the diff itself.

## Helped

- hit-rate: After the change, a key reaches stash MRU only through a recent ghost hit or a demotion from the normal tier. The periodic blind MRU slot no longer hands protected stash space to one-off keys, so the keys that hold it have real reuse evidence. The gain over the parent was larger than BIP removal gave alone on the shared cycle-0006 ancestor. The likely reason is v002's recency gate. Stale ghost hits that fail it are routed into the insertion branch BIP used to control, so the blind MRU slot was taking more stash capacity in this lineage than in the ungated one.

## Hurt

## Takeaway

The recency gate and BIP removal do not overlap; they reinforce each other. The gate pushes more traffic into the ghost-miss insertion path, which makes that path's blind MRU branch more expensive. That is why removing the branch paid more here than on the ungated parent. The open cycle-0007 question is now answered: the gate did not recover the capacity BIP was wasting. Stash MRU should require evidence, either a recent ghost hit or a demotion. This result comes from one LIRS trace at one cache size.
