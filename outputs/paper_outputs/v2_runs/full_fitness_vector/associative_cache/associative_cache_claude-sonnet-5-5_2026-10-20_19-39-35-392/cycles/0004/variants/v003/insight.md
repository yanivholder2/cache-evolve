# Variant v003 Insight

## Helped

- per-op-time (put): Hashing once per operation and using a mask/multiply-shift bucket index with a tag-first, early-exit scan removed redundant hashing and modulo work, which the put path (insert plus victim selection) pays more of than get does.

## Hurt

- per-op-time (get): The get path got no faster, and its median and tail came out slightly worse. The baseline's get was already cheap, so the added tag compare and branch-hint restructuring bought nothing and may have added noise or a little extra work.

## Takeaway

Hash and index arithmetic is not the bottleneck on the get path. The remaining get cost is probably memory access to the bucket, not dispatch or modulo. The restructuring only paid off on put, where per-call overhead is larger. Hit rate and metadata-size are unchanged because replacement policy and layout were left alone, as the hypothesis intended. Further per-op-time gains on get need to target cache-line and memory behavior, not arithmetic.
