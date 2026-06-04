# Variant v001 Insight

## Helped

- hit-rate: Stripping the `self.seen[fp] = true` from `get` restored the gate's precondition — under the framework's `get_miss → put` pattern, the *first* `put` of a brand-new key now actually observes `was_seen == false`, so the `if self.len >= self.capacity && !from_ghost && !was_seen { return; }` line at the top of `put` finally fires and drops one-touch scan keys instead of letting them evict a Hot resident.

## Hurt

## Takeaway

The cold-admission gate is a real lever on lirs:multi1@512, not an inert one — the cycle-5 result was a precondition-collapse bug, not a verdict on the policy. Once `get` stops touching the sketch, requiring a key to be sketched twice before it can displace a resident measurably out-performs unconditional CLOCK-Pro-lite admission on this scan-heavy trace. The sketch must be owned by `put` alone; any mark on the read path defeats the gate before it can act.
