# Commit Message Guidelines

All commit messages must follow Conventional Commits format.

Format:

<type>(<scope>): <short description>

Types:

- feat: new features
- fix: bug fixes
- refactor: code changes without behavior changes
- docs: documentation changes
- test: tests
- chore: maintenance
- perf: performance improvements

Rules:

- Use imperative mood.
- Keep the subject under 72 characters.
- Do not use past tense.
- Include a body only when the change is complex.
- Prefer specific scopes.

Examples:

- feat(widgets): add image widget support
- fix(layout): prevent image shrink during flex layout
- refactor(rendering): simplify paint pipeline
