# Variant v002 Insight

## Helped

- put: The cached two-candidate victim pair skips the full-bucket scan on most evictions, so the median put path got slightly cheaper, which is the latency effect the hypothesis targeted.

## Hurt

- hit-rate: The cached pair goes stale between full scans, and the approximation no longer tracks the true lowest counter, so frequency-protected keys are sometimes chosen as victims. This is the loss of frequency protection the hypothesis named as its refutation condition, and it triggered.
- get: The hypothesis left the get path unchanged, yet get latency worsened on both median and tail. The stated mechanism does not explain this, so it likely comes from layout or code-path side effects of the overlay rather than from the victim-selection change.
- put: Tail latency on full-bucket puts rose sharply even though the fallback path is the same full scan v003 already ran, so the tail increase points to something beyond the shortcut itself (for example, how often the invalid-cache fallback fires).
- metadata-size: No change, consistent with the two-index cache being stored at bucket level as specified, though the size probe may not capture it.

## Takeaway

Shortcutting the victim scan with a cached two-candidate pair buys only a small put-median gain and costs hit-rate, because stale candidates hide the true frequency minimum that v003's exact scan was protecting. The exact minimum search is doing real work for hit-rate, so any follow-up should keep it or refresh the pair on counter updates instead of accepting staleness.
