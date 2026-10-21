# Variant v003 Insight

## Helped

(None. hit-rate, the only recorded dimension, did not rise.)

## Hurt

- hit-rate: The ghost ring now records only evicted keys whose `lfu_counter` is still 0. As a result, keys that were hit, promoted to MRU and then aged down to index 0 can no longer come back at index 3. Before this change they could. The change does not seem to have bought enough to make up for that: the ring keeps 4 slots and already overwrites its oldest entry, so filling those slots only with never-hit newcomers did not give more useful ghost hits. The drop is very small, and one trace at one size cannot separate it from noise, so the honest reading is "no gain" rather than "clear loss."

## Takeaway

H03 is refuted by its own criterion. On the 1/8-depth LRU line, previously-hit keys that age out of the bucket are worth remembering in the ghost ring at least as much as never-hit newcomers. Filtering them out removed a real source of ghost hits and did not sharpen the signal. Cycle 0005 found the opposite on the CLOCK line, where a long-reuse population diluted the ring. That lesson does not carry over here: on LRU, a hit key that reaches index 0 has a reuse distance just past the bucket, which is exactly what the index-3 reward is designed to rescue. Next steps on this line should leave the ring's admission rule alone and try a different lever, such as ring length or ghost-hit depth. One implementation detail: the "was hit" mark is set in `get_from_position`, which also runs on value updates to an existing key, so updates count as hits for filtering purposes.
