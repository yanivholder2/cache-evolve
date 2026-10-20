# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Preferring never-re-hit stash entries as eviction victims dropped hit-rate on the LIRS workload-a trace at size 2048, because a stash entry that has not yet been hit is often one that has just arrived and has not had a chance to prove reuse, so the new victim scan tends to discard the entries most likely to be hit next.

## Takeaway

Using the one-bit "seen in stash" marker to pick eviction victims, rather than only to gate promotion, is a net loss on this trace. The marker cannot tell a cold entry from a useless one, so the victim scan is biased toward recently arrived entries and works against the reuse it was meant to protect.
