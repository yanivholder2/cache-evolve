# Variant v001 Insight

## Helped

- hit-rate (128K, `hm_0`): This is the largest gain. It now beats both of the cycle-0005 variants it combines, v001 (Back probing placement alone) and v003 (ghost alone). A key that proves it is hottest in its target now lands at the probing MRU end, so the next probing eviction takes the resident that has gone longest without use. Under the parent, the promoted key took the LRU slot and was the next victim. The ghost-credited fast entry into the target feeds more keys into this path, and placement now keeps them once they get there.
- hit-rate (16K, `hm_0`): The same mechanism works at the small size, and it also lands above both single-lever variants. On `hm_0` the two levers really are additive: the ghost lifts stash→target admission and MRU probing placement lifts target→probing retention.
- hit-rate (16K, `web_0`): Essentially flat, as it was under the parent. Neither lever reaches this cell. The ghost ring's span is too short at 16K, and `web_0` seems to have few target-hottest keys whose survival in probing matters.

## Hurt

- hit-rate (128K, `web_0`): A slight slip from the parent. This cell is where the ghost earned most of its gain, so the stacking is not perfectly clean. With the promotee placed at the MRU end, the probing victim is now always a resident that was already there. That is often a foreign key, and it retries its own target and can cascade a target item into the stash. That means more stash overflows, more ghost churn, and fewer ghost-credited returns surviving. This is a plausible cost of the interaction, not one this run proves.

## Takeaway

MRU placement for target→probing promotees and the stash-eviction ghost mostly stack. On `hm_0` the combined gain beats either change alone, which supports the hypothesis that the two levers work on separate tiers. They are not fully independent, though. Keeping proven keys in probing pushes evictions onto foreign residents, whose cascade feeds the stash and the ghost ring. That shows up as a small `web_0` 128K giveback. If a later change makes these cascades more frequent, check its effect on the ghost.
