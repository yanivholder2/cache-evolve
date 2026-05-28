# Segment is the only generic seam; every other extension point is a trait object

A Subject exposes many axes a Variant might swap — eviction policy, bucket,
item, probing function, cascade order, and more. We make **the Segment trait
the sole generic seam**: `Dash<K, V, S: Segment<K, V>>` carries one type
parameter, and every other Extension point is a `Box<dyn Trait>` owned by the
smallest struct that uses it (e.g. `EvictionPolicy` inside the bucket, `Bucket`
inside `SegmentCore`). No type parameter propagates above the Segment.

The payoff: adding or swapping an axis is a local, mechanical change — publish a
trait, give the owning struct a `Box<dyn Trait>` field, optionally add a
factory — rather than a cross-cutting generics refactor that ripples through
`Dash`, `SegmentCore`, `Segment`, and the JNI signatures. This keeps those
surfaces stable and widens the LLM's action space cheaply.

## Considered options

- **Generics everywhere.** A type parameter per axis. Maximises monomorphised
  performance but every new axis is a breaking, repo-wide refactor and the JNI
  boundary (which is concrete) would need per-combination instantiation.
  Rejected.
- **Trait objects everywhere, including Segment.** Uniform, but Segment is the
  strategy axis already grandfathered as generic on `Dash`, and the per-`get`
  strategy dispatch is the hottest path. Kept it generic.
- **Segment generic, everything below it a trait object.** Chosen. One vtable
  indirection per axis on the call path — negligible against the work each trait
  method already does.

## Consequences

- Adding a future axis follows a fixed recipe and touches only the owning
  struct; reviewers can recognise the shape.
- Variants pay one vtable indirection per swapped axis. Accepted as the cost of
  a stable, pluggable surface.
- `Dash`, `SegmentCore`, `Segment`, and JNI signatures stay untouched as axes
  are added.
