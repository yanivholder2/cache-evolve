# Variant v002 Insight

## Helped

- None measured. The hypothesis targeted metadata-size by shrinking sketch row width to about 75%, but the fitness vector reports only hit-rate, so any memory saving is not visible here.

## Hurt

- hit-rate: Slightly lower than the parent. Narrower sketch rows mean more hash collisions, so frequency estimates inflate and the tie-admit duel makes somewhat worse admit/reject calls. The drop is small, but it is in the direction the 2x-shrink attempt in the rationale predicted. I could not read the overlay source in this session, so this rests on the hypothesis text and the fitness vectors.

## Takeaway

Even a gentle 25% cut in sketch width costs some hit-rate on workload-a at size 2048, so the sketch is already close to the width where collisions matter. Any metadata saving from this change has to be weighed against that loss. The next step is to add a metadata-size dimension so the trade-off can be seen directly. Without it, the benefit this change was built for goes unmeasured.
