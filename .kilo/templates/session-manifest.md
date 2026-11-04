# Session Manifest Template

Copy to `.kilo/sessions/<session-id>/manifest.md` (filled by `scripts/session-start.ps1`).

```markdown
# Session: <session-id>

- Project(s): <comma-separated>
- Goal: <one line>
- Created: <ISO-8601 timestamp>
- Owner: <user or agent name>
- git_head_start: <short sha>
- git_head_end: <short sha>
- dirty_start: <true|false>
- dirty_end: <true|false>

## Artifacts

| Type | Path | Tracked? |
|------|------|----------|
| conversation | .kilo/sessions/<session-id>/conversation/ | yes |
| tests | <project>/artifacts/<session-id>/tests/ | partial |
| runs | <project>/artifacts/<session-id>/runs/ | no |
| io | <project>/artifacts/<session-id>/io/ | partial |

## Commands run

- <ISO timestamp> `<command>` (exit <code>, cwd <dir>)

## Notes

- <decisions, blockers, follow-ups>
```
