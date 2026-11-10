# Session Index

Master list of work sessions. One row per session directory under `.kilo/sessions/`.
Heavy artifacts live in `<project>/artifacts/<session-id>/`.

| Session ID | Project(s) | Goal (short) | Created | Revision (start..end) | Status |
|------------|------------|--------------|---------|-----------------------|--------|
| 20261110-dev-memory-v2 | (none) | (see manifest) | 2026-11-10 | (start..77df191) | complete |
| 20261107-dev-flaps-completion | root workspace | Execute v2.0 completion plan S0-S7 (api.rs repair, P1-P5 pages, quality gates) | 2026-11-07 | 9704933..(open) | active |
| 20261106-autonomous-evolution-init | ekpi-platform, root workspace | Initialize autonomous engineering evolution session; validate baseline; record evidence | 2026-11-06 | a268319..a268319 | active |
| 20261103-dev-elite-impl | (none) | (see manifest) | 2026-11-03 | (start..nogit) | complete |
| 20261102-dev-sessionmgmt-pilot | Wellfit | (see manifest) | 2026-11-02 | (start..nogit) | complete |

## Rules
- Every directory under `.kilo/sessions/` must have a `manifest.md`.
- Session IDs match `^\d{8}-[a-z0-9-]+-[a-z0-9-]+$`.
- Update this file at session end (see `scripts/session-end.ps1`).


