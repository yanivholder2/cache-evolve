# Cycle 0010 Insight

## Cohort outcome

The cohort ran one Pareto-composition bet and two headroom re-probes off the
scalar-mask incumbent: fold cycle 9's `u64` reuse-mask onto cycle 9's
integer-hash bucket (v001), re-open the untested *narrower*-sets geometry
direction (v002), and re-open protection in the one saturated regime where the
binary bit can no longer rank survivors (v003). The clean, decisive win was
v001 — it married the record hit-rate to the latency floor at the 16-byte
metadata floor, bit-for-bit. v002 and v003 both regressed hit-rate and only
qualified by holding the metadata floor; each closes a previously-open thread
as a falsification.

## Patterns

- Orthogonal levers compose cleanly, and that is where the frontier moved this
  cycle. The best hit-rate (hash-driven distribution) and the best latency
  (reuse-set representation) had lived in separate variants; because hash and
  reuse-set representation are independent, splicing them yielded a strict
  Pareto improvement with no trade-off — the mask splice mirrors the policy's
  single-element relocation exactly, so hit-rate held to the digit.
- The geometry seam is now falsified on *both* sides. Prior cycles ruled out
  wider buckets; v002 rules out narrower ones — quartering set width to
  quadruple set count raised conflict misses and widened the put tail. Per-set
  associativity, not set count, guards the SLRU ceiling; the harness default
  split sits at the optimum, not on a slope.
- Permanent one-hit protection is load-bearing even at saturation. The last
  untested weakening — clearing the mask only when a bucket is fully protected
  (v003) — still regressed: a saturated mask retains per-key reuse evidence
  that outlives one eviction, and zeroing it readmits stale/scan traffic. Every
  attempt to re-open protected slots, on any schedule or regime, now regresses.

## Open questions

- With hit-rate and latency now co-located at their floors, is there any
  harness-resolvable dimension left to move, or is the incumbent at the joint
  frontier for this trace?
- Are the remaining tails purely the policy's own `items` front-shifts, and is
  there a resolvable gain there at all?

## Continuation

Adopt v001 as the incumbent: it is the strict Pareto point — record hit-rate,
latency floor, 16-byte metadata floor, all at once — superseding both the
hash-only and mask-only parents. Treat three seams as now closed: the geometry
lever is falsified on both the wider and narrower sides (stop repartitioning
buckets-vs-slots), and protection weakening is falsified in every regime
including saturation (stop probing hit-rate by re-opening protected slots). The
hit-rate ceiling should be considered firm and fully mapped. If any lever
remains, it is the `items` front-shift latency path — but first confirm a
harness-resolvable gain exists before spending a variant, since the incumbent
already sits at the timer floor on the measured paths.
