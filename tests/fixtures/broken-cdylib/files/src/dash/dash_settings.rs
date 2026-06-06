// Broken-overlay fixture for the real-cargo cdylib build test
// (test_build_cdylib). Overlays a non-pinned dash source file with a forced
// compile error so the cdylib build fails. factory.rs — the shared `broken`
// fixture's vehicle — is contract-protected and now pinned out of overlays, so
// it can no longer carry the breakage for dash; see
// Subject.is_pinned_overlay_path.
compile_error!("broken-overlay fixture: forced cdylib build failure");
