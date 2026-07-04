# Cycle 0009 Insight

## Cohort outcome

With every hit-rate lever ruled out over the prior cycles, this cohort stopped
hunting for hit-rate headroom entirely and spent all three slots on
behaviour-preserving cost-side rewrites: a u64 logical-clock timestamp shrinking
`Item` metadata (v001), a `rotate_left` MRU promotion on the get path (v002),
and a rotate-and-replace full-bucket admission on the put path (v003). All three
were provably byte-identical and all three tied on hit-rate, so all three
"won" as undifferentiated ties and the baseline behaviour stands unchanged.

## Patterns

- The whole cohort is a clean demonstration that the ceiling is real: every
  variant was designed to move nothing on hit-rate, and none did. The search
  has fully pivoted from behaviour to implementation cost.
- The three changes target the three distinct cost axes the fitness cannot see
  — metadata footprint (smaller `Item`), get p50 (one suffix memmove instead of
  remove+push with a capacity check), and put p50 (one prefix rotate plus
  in-place replace instead of two memmoves and a length round-trip). Each is
  provably free but only provably *beneficial* on a per-op-time or size axis
  this hit-rate-only vector does not measure.
- This is the same recurring outcome the last three cycles flagged: genuine,
  defensible cost wins keep landing as ties because there is no dimension to
  reward them. The pattern is now systemic, not incidental.

## Open questions

- How long does the loop keep generating tied cost-side variants before the
  lack of a reward axis makes the cohort uninformative?
- The three cost rewrites are independent and stackable — does adopting all of
  them as the baseline change anything the fitness can observe, or do they
  remain collectively invisible?

## Continuation

Hit-rate is exhausted and this cycle adds no behavioural change; adopt v001's
u64 timestamp, v002's rotate-left get promotion, and v003's rotate-replace put
admission as the baseline since all are byte-equivalent and cost-favourable.
The next cycle should not search for hit-rate headroom — there is none left.
The decision that has now been deferred for four straight cycles must be made:
either formally add a per-op-time / metadata-size fitness dimension so these
recurring cost wins become selectable, or declare the trace solved and stop the
loop. Continuing to emit tied cost rewrites under a hit-rate-only fitness is no
longer producing signal.
