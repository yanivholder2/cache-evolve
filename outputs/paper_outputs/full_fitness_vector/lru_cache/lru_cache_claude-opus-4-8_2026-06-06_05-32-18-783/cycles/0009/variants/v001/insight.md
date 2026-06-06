# Variant v001 Insight

## Helped

- metadata-size: Packing the CLOCK reference bit into the high bit of the
  existing `next` word (mirroring `PROT_MASK` on `prev`) added the second-chance
  state at zero new per-entry storage, so `Node<u64,u64>` stayed in the 24-byte
  class exactly as the hypothesis required.

## Hurt

- per-op-time: The change made get *slower*, not faster — the opposite of the
  claim. The parent's protected-MRU branch returned with no writes at all; the
  CLOCK rewrite replaces it with `set_referenced(true)` on *every* protected hit,
  converting the dominant hot-key read into a read-modify-write store on the node.
- per-op-time: put also regressed because stealing `REF_MASK` from `next` turned
  the formerly free `next_link()`/`set_next_link()` into mask-and-preserve
  operations, taxing every `unlink`/`push_back` on the put path, plus the rewritten
  `demote_protected_lru` now loops with second-chance rotations instead of a single
  unlink.
- hit-rate: The CLOCK sweep only approximates LRU within the protected set — its
  second-chance rotation picks a different demotion victim than strict
  protected-LRU did, shaving a sliver of accuracy off the parent.

## Takeaway

Moving protected-hit surgery off the read path backfired: the parent's
protected-MRU fast path was already write-free, so the reference bit reintroduces
a store on the hottest accesses while burdening every link touch with mask/preserve
— get and put both regress and CLOCK's approximation nicks hit-rate. The "per-hit
link surgery" the hypothesis targeted wasn't actually being paid by the dominant
MRU access pattern, so generalizing the short-circuit traded a real fast path for
overhead everywhere.
