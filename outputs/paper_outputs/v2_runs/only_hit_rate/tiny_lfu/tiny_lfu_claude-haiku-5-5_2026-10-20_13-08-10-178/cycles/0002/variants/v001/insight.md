# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Narrowing the `CountMinSketch` counters from `u32` to `u16` (with `MAX_COUNT` saturating at `u16::MAX`) halves the sketch's heap footprint, but the fitness vector reports only hit-rate, so the intended metadata-size gain is unmeasured here. Hit-rate on workload-a at size 2048 is unchanged from the parent, which suggests saturation at 65,535 did not create enough admission ties under the strict `>` test to matter on this trace; the tie risk is therefore not ruled out for workloads with more very hot keys. The `as u32` widening in `estimate` keeps the admission comparison type-stable, so any behavioural difference can only come from saturation.
