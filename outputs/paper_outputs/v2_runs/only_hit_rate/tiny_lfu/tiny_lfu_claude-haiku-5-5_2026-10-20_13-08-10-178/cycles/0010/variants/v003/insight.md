# Variant v003 Insight

## Helped

- None measured: the variant's fitness vector only reports hit-rate, so no other dimension moved in a way this variant can be credited with.

## Hurt

- hit-rate: Deleting the `sketch.increment` call at the top of `get()` means reads no longer feed the admission estimate, so resident keys that are read often but rarely re-put stop accumulating frequency. When a newcomer is compared against such a victim in `put()`, the victim looks colder than it is and gets evicted.

## Takeaway

The hypothesis treated read-side sketch increments as noise that inflates estimates for read-only keys, but on this trace those increments were carrying real popularity signal that the `put()`-only estimate does not replace. Any future attempt to narrow sketch updates to writes should keep some read contribution for resident keys, or be tested on a trace where reads and writes are less correlated.
