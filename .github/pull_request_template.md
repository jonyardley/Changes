## Summary

<!-- What does this PR do? 1–3 sentences. Squash-merge uses the PR title +
     body as the commit message — write them as the permanent record. -->

## Roadmap alignment

- [ ] Maps to an item in [`docs/roadmap.md`](../docs/roadmap.md) (no item = discuss first)

## Checklist

- [ ] `just ci` passes locally (full CI mirror: fmt-check, clippy, test, deny, links, gitleaks)
- [ ] New bridge-crossing types (`Event`/`Effect`/`ViewModel`) have a real round-trip test
- [ ] New UI uses `Changes*` tokens — no literal spacing/radius/colour/type
- [ ] Storage changes: migration is additive and ships an upgrade-path test
- [ ] Tests skipped? Said so below, with the reason ("all existing tests pass" is not coverage for new code)
- [ ] CLAUDE.md / `docs/roadmap.md` updated if architecture, patterns, or scope changed
