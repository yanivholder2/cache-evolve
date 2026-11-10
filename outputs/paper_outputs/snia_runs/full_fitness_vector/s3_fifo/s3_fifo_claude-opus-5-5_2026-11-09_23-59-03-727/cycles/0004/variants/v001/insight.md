# Variant v001 Insight

## Helped

- hit-rate: In `evict_main()`, main victims with `reused == false` are now dropped without a ghost entry. These are keys that graduated (or were re-admitted from ghost) and then got no hit while in main. Dropping them saves the ghost slots, which are bounded at `main_size`, for keys that proved useful in main, so ghosts are less often pushed out before their key comes back. The hit rate rose at all four trace/size points and the summary cleared the hypothesis threshold.
- hit-rate: Clearing `freq` and `reused` when a key graduates from small to main means a burst of hits during admission no longer buys extra second chances in main. Main now holds keys by their reuse inside main, and the ghost gate only counts that reuse. The gain is larger on hm_0 than on web_0 at both sizes, so hm_0's working set responds more to this stricter in-main accounting.
- per-op-time: The `put` tail dropped a little. Unreused main victims skip `push_ghost`, so a miss less often pays for the ghost-index insert, the bounded pop-back scan, or the `2 * main_size` `retain` compaction. The summary metric did not move.

## Hurt

(none: metadata-size held because `reused: bool` sits in `Entry`'s existing padding after `freq`, as the new `entry_fits_in_padding` test asserts; the extra `reused = true` store in `get()` writes the same slab entry that the freq update already touches and cost nothing measurable)

## Takeaway

The two cycle-3 filters act on separate mechanisms: the reuse gate controls who gets a ghost slot, and the graduation reset controls how many second chances a key carries into main. Merging them on the Fx-hashed slab gave a strict hit-rate improvement at every measured point, with no change in entry size and no cost to `get`/`put` latency. Clearing `reused` together with `freq` at graduation is what makes them work together: both filters then judge a key only by its reuse inside main. hm_0 benefits most, so further tuning of main-queue retention (for example, the graduation threshold or `MAX_FREQ`) is more likely to pay off there than on web_0.
