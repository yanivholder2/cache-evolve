# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:43:42+00:00

## timeline

### 14:43:51 thinking



### 14:43:56 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Filtering one-shot inserts so they do not displace entries with proven reuse is the only direction that has produced hit-rate gains more than once. It shows up as a shared-frequency admission gate, an eviction-next insert position, and a recurrence (second-miss) gate.\n- Strict frequency comparison in the shared-frequency admission gate has been the more reliable form. Admitting on ties gave no benefit.\n- Probationary-first eviction has held up as a base for the scan-resistance gains.\n\n## Recurring failures\n\n- Placing admitted keys at the probationary victim end lost hit-rate in two separate cycles.\n- Protected-tier variants (second-hit promotion, demote-on-promotion, and demote-skip) lost or stayed below parent in several cycles. Shrinking the protected region changed nothing, so the failure seems to lie in the demotion or second-hit signal, not in capacity.\n- Widening the second-miss ghost window regressed, which suggests stale one-shot keys are admitted when the first-miss memory is longer.\n- Compact or bounded frequency stores (direct-mapped, saturating, u8 counters, singleton pruning) reduced hit-rate or gave small regressions. The cause is confounded across changes.\n- Hit-rate-only geometry and aging-period tweaks (bucket repartitioning, the AgedLfu period, aging counter width, fingerprint filter, sampled victim scan) were flat or marginal. None was paired with a cost measurement.\n\n## Open questions\n\n- The recorded Winner often does not match the hit-rate evidence in its own insight (Cycles 0004, 0006, 0007, 0008). The selection rule needs checking before any Winner is used as precedent.\n- Per-op time and metadata size have not been measured in any cycle. Every cost-motivated hypothesis remains unverified.\n- Do the admission gate and eviction-next insert position stack, or are they redundant? Do they change the same victims?\n- Does the ghost-list recurrence gain (Cycle 0007) reproduce on a second workload or with the same setup?\n- Does the shared-frequency gain (Cycle 0004) hold beyond workload-a?\n- Were overlays active under the harness in several cycles? Flat results cannot be read as null without that check.\n\n## Per-policy notes\n\n- **Aging LFU:** No hit-rate gain in Cycles 0001 and 0002. Whether the evaluation selects the aging policy at all is unresolved, so do not iterate on its period or counter width alone.\n- **Shared-frequency admission gate:** The most consistent positive lever for one-shot filtering. Results depend on a common scale for incoming and resident keys and on unbounded or exact counters; bounding the counter cost hit-rate.\n- **Recurrence (ghost-list) gating:** One-capacity window showed a gain (Cycle 0007). Longer windows regressed. Metadata cost per key is not yet measured.\n- **Probationary/protected (segmented) structure:** Probationary-first eviction helped once. Protected-tier variants and protected-capacity changes did not. Demotion and second-hit signal are the remaining suspects.\n- **Bucket / set geometry and sampled victim scans:** Hit-rate-neutral in every case. Any gain here would have to come from per-op time, which has not been measured.\n"
}
```

### 14:43:59 chat

I wrote `cross-cycle-insights.md` to the output directory with the four required sections, in order. I did not read the previous version of the file.

The main signals across cycles 0001–0008:

- **Recurring wins:** Filtering one-shot inserts so they don't displace proven-reuse entries is the only direction that has produced hit-rate gains more than once. Strict frequency comparison in the admission gate has beaten the tie-admitting variant.
- **Recurring failures:** Victim-end placement for admitted keys has lost hit-rate twice. Protected-tier variants have repeatedly failed, and shrinking the protected region changed nothing, which points at demotion or the second-hit signal. Widening the ghost window regressed. Compact frequency stores lost hit-rate, though the causes are confounded.
- **Open questions:** The recorded Winner often conflicts with its own hit-rate evidence (cycles 0004, 0006, 0007, 0008), so the selection rule needs checking. Per-op time and metadata size have never been measured. Several flat results may come from overlays that were never applied.
- **Per-policy notes:** Aging LFU has no gain so far. The shared-frequency admission gate is the most consistent positive lever. The one-capacity ghost-list gain is unconfirmed. Geometry and sampled-scan changes have been hit-rate-neutral.

The file has no hit-rate magnitudes beyond the qualitative wording in the digest, since the digest gives none.


---

- finished: 2026-10-20T14:43:59+00:00
- duration: 16.83s
- status: ok
- input_tokens: 4
- output_tokens: 3133
- cache_read_tokens: 40237
- cache_write_tokens: 13426
- cost_usd: 0.004654
