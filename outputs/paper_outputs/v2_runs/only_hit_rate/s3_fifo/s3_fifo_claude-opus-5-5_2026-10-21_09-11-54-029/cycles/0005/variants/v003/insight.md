# Variant v003 Insight

## Helped

- hit-rate: `evict_main` used to drop its freq-0 victims with a bare `self.release(idx)`. It now calls `ghost_insert(entry.key)` on them. On workload-a, a key that left main and comes back now takes the existing `ghost_set.remove` → `main.push_front` path. Before, it went through the 102-slot small queue and needed `freq > 1` to graduate, so it often got ghosted again first. That extra miss is gone, and hit-rate rose a little, as H03 predicted.

## Hurt

(No measured dimension regressed. The ghost-dilution risk named in H03 is still there but did not dominate. Main victims and small-queue rejects now share one live ghost budget of `main_size`, enforced by the unchanged loop in `ghost_insert`. Some small-queue rejects must therefore leave the ghost sooner. The overall gain means the faster return of main victims outweighed this on workload-a, but the two effects can't be separated with one trace at one size. Two unmeasured costs also grew. Every main eviction now does a ghost insert (a `HashMap` insert and a deque push). The lazy `retain` compaction at `2 * main_size + 16` now runs more often, because the ghost deque fills from two sources.)

## Takeaway

Ghosting main victims is a real but very small win on workload-a. The likely reason: with `MAX_FREQ = 7`, a key only leaves main after its credit has decayed to 0 over several CLOCK sweeps. Few such keys come back within the ghost's `main_size` window, so there isn't much to recover. Re-admission stays conservative (front of main, freq 0), so the policy doesn't give returners a sticky advantage. Larger gains from this route would probably need the ghost to stay a separate signal. Two candidate tests: a separate ghost budget for main victims, so dilution can be measured on its own, and giving returning main victims more than freq 0. Widening the shared ghost's evidence alone isn't enough.
