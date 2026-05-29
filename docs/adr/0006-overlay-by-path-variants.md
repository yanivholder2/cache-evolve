# Variants are stored as an overlay-by-path, not a diff or a branch

A Variant ships a whole cache implementation, but most Variants change only a
few files of the Subject tree. We store a Variant as an **overlay**: an
`overlay/` directory mirroring the Subject's layout, containing only the files
the Variant adds or modifies. To build, the harness copies the Subject tree into
a temp workspace, copies the overlay on top at matching relative paths, rewrites
the `framework` dependency path to absolute, and runs `cargo`. No deletions.

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

## Consequences

- A Variant cannot delete a Subject file — only add or replace. Acceptable: the
  Cache contract is satisfied by replacement, not removal.
- The build is hermetic and cwd-independent (temp workspace + absolute
  `framework` path), so `cargo build`/`cargo test` run anywhere.
- Subject changes are not auto-propagated into existing Variant overlays; a
  Variant pins the files it overrode at authoring time.
- A cycle Variant's overlay lives under `attempts/NN/overlay/` — one per
  self-repair attempt — alongside that attempt's `task.md` and agent log;
  gates and fitness resolve against the latest attempt's overlay. The baseline
  and the prose `v<NNN>` template keep a flat `overlay/` with no `attempts/`.
