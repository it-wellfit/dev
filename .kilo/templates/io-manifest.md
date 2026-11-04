# IO Manifest Template

Copy to `<project>/artifacts/<session-id>/io/manifest.md`.

```markdown
# IO Manifest — <session-id>

| Direction | File | SHA-256 | Source / Destination | Notes |
|-----------|------|---------|----------------------|-------|
| input | input/<name> | <hash> | <origin path or URL> | snapshot |
| output | output/<name> | <hash> | <command that produced it> | generated |
```

Rules:
- Inputs are immutable snapshots; re-runs use the snapshot, not the original.
- Record SHA-256 for every file (use `Get-FileHash -Algorithm SHA256`).
- Never commit files under `input/` or `output/`; commit only this manifest.
```
