# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-06-10T21:42:45+00:00

## timeline

### 21:42:50 thinking



### 21:42:50 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Check working directory and contents"
}
```

### 21:42:53 thinking



### 21:43:11 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Explicit segmentation with a single-hit promotion (promote-on-first-reuse,\n  straight to MRU) is the strongest scan-resistant scheme found; it beat pure\n  LFU victim selection and plain single-position LRU when first introduced and\n  has survived every subsequent challenge.\n- FIFO (oldest-first) probation outperforms newest-first churn: giving a fresh\n  key a full lap to earn promotion recovers rising keys and lifted hit-rate\n  with no metadata change.\n- \"Once-hot immortality\" — durable protection that is hard to lose — is\n  repeatedly the right policy on this collision-bound trace.\n- The one structural hit-rate gain after early saturation came from *geometry,\n  not logic*: a single bucket-count halving (capacity conserved) relieved mild\n  associativity starvation.\n- Behaviour-preserving cost rewrites (O(1) eviction/put paths, smaller `Item`\n  metadata, rotate-based promotion/admission) are reliably byte-identical and\n  safe to adopt.\n\n## Recurring failures\n\n- Any change that withholds or strips earned protection regresses: counter\n  aging, two-hit promotion thresholds, protected-segment caps with demotion,\n  and CLOCK recency approximation all erode the durable protection the trace\n  rewards.\n- Composing the orthogonal levers from early cycles (front admission +\n  frequency-first eviction) conflicts rather than stacks — a fresh key becomes\n  both recency-oldest and min-frequency, so it is evicted immediately.\n- \"Recently evicted\" is not a reliable proxy for \"hot\": LFU-min fallbacks and\n  ghost-ring fast re-entry let weakly-warranted keys seize protected space.\n- Re-tuning protection internals fails: elite/third tiers, reserved\n  probationary slots, and looser/larger protected sizing all net-negative.\n- A second bucket halving overshoots (routing loss), and an absolute\n  associativity target degenerates to a sub-optimal fixed width — geometry is\n  bracketed on both sides.\n- Stronger hashing (avalanche finalizer + Lemire reduction) gains nothing;\n  SipHash is already uniform on this trace.\n\n## Open questions\n\n- Is hit-rate genuinely at its ceiling now that admission, promotion,\n  eviction signal, re-entry, routing, tiering, protected sizing, and bucket\n  width are all exhausted?\n- Should a per-op-time / metadata-size fitness dimension be added? Cost-side\n  wins keep recurring and keep landing as undifferentiated ties because the\n  hit-rate-only vector cannot see or reward them — this decision has been\n  deferred for ~five cycles.\n- Did front-admission \"evict newest unpromoted\" ever actually move hit-rate,\n  or was the scan-resistance claim unsupported? Worth a clean re-test before\n  recycling.\n- Is there any untested trace-structure axis left, or should the trace be\n  declared solved and the loop stopped?\n\n## Per-policy notes\n\n- **LRU / recency:** Full positional recency ranking is load-bearing in small\n  buckets; coarse one-bit (CLOCK) approximations underperform. Promote-to-MRU\n  placement beats seating promotions at the protected-LRU boundary.\n- **LFU / frequency:** Useful as the original victim-selection signal, but\n  banked counts do not separate hot from rising keys better than protected-LRU\n  recency; aging counters and LFU-min fallbacks both regress.\n- **Segmented-LRU (current baseline):** FIFO probation + single-hit\n  promote-to-MRU + full immortality at current protected size is the settled,\n  repeatedly-confirmed policy; internal re-tiering and re-sizing all fail.\n- **Geometry / routing:** One bucket halving is the optimum; further width\n  change (either direction) and hash-quality changes do not help. Treated as\n  an axis independent of, and now exhausted alongside, the segment logic.\n"
}
```

### 21:43:18 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 10-cycle ledger, using the four fixed sections.

Key synthesis across cycles:
- **Recurring wins:** segmented-LRU with single-hit promote-to-MRU, FIFO probation, durable "once-hot immortality," and the single bucket-halving geometry gain — plus safe byte-identical cost rewrites.
- **Recurring failures:** every protection-erosion variant (aging, two-hit thresholds, caps/demotion, CLOCK), the front-admission + frequency-eviction conflict, "recently-evicted ≠ hot" re-entry tricks, protected re-tiering/re-sizing, second halving / absolute-width targets, and stronger hashing.
- **Open questions:** whether hit-rate is truly at ceiling, and the long-deferred decision to add a per-op-time/size fitness axis so recurring cost-side ties become selectable.
- **Per-policy notes** grouped under LRU/recency, LFU/frequency, segmented-LRU baseline, and geometry/routing.


---

- finished: 2026-06-10T21:43:18+00:00
- duration: 33.77s
- status: ok
- input_tokens: 5
- output_tokens: 2141
- cache_read_tokens: 69829
- cache_write_tokens: 12830
- cost_usd: 0.108372
