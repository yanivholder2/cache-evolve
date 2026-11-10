# Variant v002 Insight

## Helped

- hit-rate (web_0, 16384): at the small size, cycle 0001 found web_0 suffering from main pollution. Here the split can move, so the controller can shift space to whichever side its ghost hits point to. Graduation also no longer forces its own `evict_main` pass. A graduate now just moves into main, and `make_room` evicts from whichever queue is over its target. Together these recover a small part of what web_0 lost in v001.
- hit-rate (hm_0, both sizes): hm_0 gains a little at 16384 and is flat at 131072. The fixed 1/4 split seems to already suit hm_0, so adapting finds almost nothing to improve.
- hit-rate (summary): the aggregate moves up by a negligible amount. Overall the change is neutral, not a win.

## Hurt

- hit-rate (web_0, 131072): the large web_0 cache gives back a little. The controller has a likely bias toward growing small. `ghost` holds up to `main_size` keys and takes in every one-hit key that leaves small. `main_ghost` holds only `capacity / 2` keys, and both kinds of ghost hit move the target by the same step. A larger small queue gives keys that are reused only once more time to get their hit and graduate, which is the same pollution v001 already suffered from.
- per-op-time (not measured): every miss now also linearly scans `main_ghost`, up to `capacity / 2` keys, on top of `ghost`. Because lookups are linear scans, this is likely a real throughput cost at 131072 that the hit-rate numbers don't show.

## Takeaway

On these traces, making the small/main split adaptive with unweighted ±`small_size/16` steps barely moves hit-rate. In v001, the split size was not what held back web_0 or hm_0. The problem was which keys got into main. A follow-up should either:

- weight the adaptation step by the ratio of the two ghost sizes, as ARC does, and log where `small_size` actually settles on each trace, to check whether the controller moves at all; or
- drop split adaptation and go back to filtering admission into main.

The doubled ghost scanning is a cost that should be checked for per-op-time before this mechanism is kept.
