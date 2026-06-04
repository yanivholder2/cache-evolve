# Variant v001 Insight

## Helped

## Hurt

## Takeaway

The scan-taint pre-pass changed *which* Cold ref-clear slot dies, not *whether* one does — and on lirs:multi1@512 that reshuffling left hit-rate bit-identical to the parent. Tainted slots are themselves valid pass-1 victims, so swapping a tainted Cold→Cold eviction in for an untainted one only buys the spared slot a single extra sweep before pass 1 reaches it on the next miss; on this trace that one-sweep reprieve almost never coincides with a re-reference. Either the first-seen sketch fires rarely at this capacity (few brand-new non-ghost Cold admissions whose fingerprint was unseen) or, when it does, the un-tainted Cold ref-clear slot it protects is itself low-value. Future pass-1 levers on this skeleton need to shield slots pass 1 would otherwise take (e.g. bias toward keeping ref-clear Cold whose ghost-set neighbours hit recently) rather than re-ordering within the existing victim pool.
