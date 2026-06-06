# Variants are stored as an overlay-by-path, not a diff or a branch

A Variant ships a whole cache implementation, but most Variants change only a
few files of the Subject tree. We store a Variant as an **overlay**: an
`overlay/` directory mirroring the Subject's layout, containing only the files
the Variant adds or modifies. To build, the harness copies the Subject tree into
a temp workspace, copies the overlay on top at matching relative paths (except
the gate-integrity surface, see below), rewrites the `framework` dependency path
to absolute, and runs `cargo`. No deletions.

This keeps each Variant a small, human-readable set of full files (not a patch
to apply), makes "what did this Variant change?" a directory listing, and lets
the same harness build the baseline (empty overlay) and any Variant uniformly.

## Considered options

- **Unified diff / patch.** Compact, but brittle against Subject drift, hard for
  an LLM to emit correctly, and unreadable as a standalone artifact. Rejected.
- **Full per-Variant tree copy.** Trivially correct but obscures what changed
  and bloats storage. Rejected.
- **Git branch/worktree per Variant.** Couples Variant identity to VCS state and
  complicates the from-any-cwd build. Rejected.
- **Overlay-by-path (whole files, mirrored paths).** Chosen.

## Gate-integrity surface is pinned

Because the overlay is applied verbatim onto the Subject copy, an overlay file at
any relative path overwrites its Subject counterpart — including the gate harness
the variant is judged by. Left unchecked, a Variant could ship a gutted
`tests/invariant_gate.rs` or `src/test_utils/invariants.rs` and pass gates it
never satisfied. So a fixed set of crate-relative paths is **pinned to the
pristine crate** and never taken from the overlay, in any authoring scope: the
manifest `[contract].protected-files` (the JNI/factory ABI seam), `Cargo.toml`
(gate test targets + required features), `src/lib.rs` (module wiring, which could
otherwise shadow the suite with an inline module), and everything under `tests/`
and `src/test_utils/`. The implementer rejects overlay writes to these up front
(corrective feedback); `_apply_overlay` skips them regardless (the structural
backstop that holds even if the reject is bypassed). See
`Subject.is_pinned_overlay_path`.

## Consequences

- A Variant cannot delete a Subject file — only add or replace. Acceptable: the
  Cache contract is satisfied by replacement, not removal.
- A Variant cannot author the gate-integrity surface; it changes the cache to
  pass the gates, never the gates. The gate's binding surface (the public cache
  API and module paths the pristine harness imports) is thereby an implicit
  contract, like the JNI ABI — break it and the build gate fails honestly.
- The build is hermetic and cwd-independent (temp workspace + absolute
  `framework` path), so `cargo build`/`cargo test` run anywhere.
- Subject changes are not auto-propagated into existing Variant overlays; a
  Variant pins the files it overrode at authoring time.
- A cycle Variant's overlay lives under `attempts/NN/overlay/` — one per
  self-repair attempt — alongside that attempt's `task.md` and agent log;
  gates and fitness resolve against the latest attempt's overlay. The baseline
  and the prose `v<NNN>` template keep a flat `overlay/` with no `attempts/`.
