# Variant v002 Insight

## Helped

- hit-rate: Raising `SKETCH_DEPTH` by one adds a fifth row with its own position seed while the per-row width stays fixed, so the min-across-rows estimate used by the admission comparison in `put()` has one more independent collision sample and is less often inflated by hash collisions. Hit-rate rose slightly on the one trace measured.

## Hurt

## Takeaway

The change is a single constant edit (`SKETCH_DEPTH` 4 to 5) that exploits the existing per-row loops in `increment` and `estimate`, so depth alone is a working lever on the admission estimate at fixed width. The gain is small and comes from one trace at one size, so it shows depth still limits accuracy at this width but does not establish a robust win. This variant's fitness vector reports only hit-rate; the sketch's counter memory grows by a quarter as a direct consequence of the extra row, and that cost is unmeasured here.
