# Variant v001 Insight

## Helped

- (none): The splitmix64 finalizer plus Lemire reduction in `get_index`/`hash` did not raise hit-rate; re-scattering keys did not concentrate the hot working set more favorably than the parent's routing.

## Hurt

- hit-rate: Replacing `DefaultHasher`'s direct `% size` reduction with the avalanche-mixed Lemire mapping slightly lowered hit-rate, because reshuffling key-to-bucket assignment moved some co-resident hot keys into bucket collisions the parent's mapping happened to avoid.

## Takeaway

This variant rules the routing-quality axis out for this trace: `DefaultHasher` (SipHash) is already effectively uniform on these keys, so adding a stronger avalanche finalizer and bias-free reduction only reshuffles which keys collide and lands as a small net regression rather than a tie. The hit-rate ceiling here is set by bucket policy and capacity, not by uneven key distribution — capacity-neutral routing changes have no headroom to exploit.
