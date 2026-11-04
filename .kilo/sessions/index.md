# Session Index

Master list of work sessions. One row per session directory under `.kilo/sessions/`.
Heavy artifacts live in `<project>/artifacts/<session-id>/`.

| Session ID | Project(s) | Goal (short) | Created | Revision (start..end) | Status |
|------------|------------|--------------|---------|-----------------------|--------|
| 20261103-dev-elite-impl | (none) | (see manifest) | 2026-11-03 | (start..nogit) | complete |
| 20261102-dev-sessionmgmt-pilot | Wellfit | (see manifest) | 2026-11-02 | (start..nogit) | complete |

## Rules
- Every directory under `.kilo/sessions/` must have a `manifest.md`.
- Session IDs match `^\d{8}-[a-z0-9-]+-[a-z0-9-]+$`.
- Update this file at session end (see `scripts/session-end.ps1`).

