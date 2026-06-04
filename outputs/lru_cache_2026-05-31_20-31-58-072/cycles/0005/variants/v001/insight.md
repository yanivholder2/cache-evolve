# Variant v001 Insight

## Helped

## Hurt

## Takeaway

The bypass-on-first-sight gate is functionally inert against this trace because `get` marks `seen[fp] = true` before `put` ever reads it: under the framework's standard `if get.is_none() { put }` pattern, `was_seen` is true on every miss-then-put, so the gate at line 172 of `put` never fires and the CLOCK-Pro-lite skeleton runs unchanged — hit-rate matches the parent to four decimals. To actually pull the cold-admission lever the hypothesis named, the sketch must be marked only on `put` (so a brand-new key's first put sees `was_seen == false`), or the gating decision must be moved into `get` and threaded forward; marking on `get` collapses the gate's precondition before it can act.
