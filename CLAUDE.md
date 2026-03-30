# hub

## Rust Workflow

bacon is running in the background and continuously writes compiler
diagnostics to `.bacon-claude-diagnostics` in the project root.

Before attempting to fix compiler errors, read `.bacon-claude-diagnostics` to see
current errors and warnings with their exact file/line/column locations.
Prefer reading this file over running `cargo check` yourself — it's
already up to date and costs no compile time.

Each line in `.bacon-claude-diagnostics` uses a pipe-delimited format:

```
level|:|file|:|line_start|:|line_end|:|message|:|rendered
```

- `level` — severity: `error`, `warning`, `note`, `help`
- `file` — relative path to the source file
- `line_start` / `line_end` — affected line range
- `message` — short diagnostic message
- `rendered` — full cargo-rendered output including code context and suggestions

After making changes, wait a moment for bacon to recompile, then re-read
`.bacon-claude-diagnostics` to verify the fix.

**All compiler warnings must be fixed before committing.** Zero warnings is the
standard. Check `.bacon-claude-diagnostics` for warnings (not just errors) and
resolve them as part of every change.

If `.bacon-claude-diagnostics` is absent or clearly stale (e.g. the file doesn't
exist after the first save), warn the user that bacon does not appear to
be running and ask them to start it in a Zellij pane with `mise run bacon-claude-diagnostics`.

## Planning

Use 'bd' for task tracking, bead(s) is alias of task(s)

**DO NOT use `bd edit`** - it opens an interactive editor ($EDITOR) which AI agents cannot use.

Use `bd update` with flags instead:
```bash
bd update <id> --description "new description"
bd update <id> --title "new title"
bd update <id> --design "design notes"
bd update <id> --notes "additional notes"
bd update <id> --acceptance "acceptance criteria"

# Use stdin for descriptions with special characters (backticks, !, nested quotes)
echo 'Description with `backticks` and "quotes"' | bd create "Title" --description=-
echo 'Updated text' | bd update <id> --description=-
```

Do NOT write design docs or plans to `docs/plans/`. All planning and design
work should be captured directly in beads.

Do NOT start implementation during the planning stage. The outcome of planning
is beads with clear specs — enough detail for a clean design and implementation
stage later.

## Development Workflow

Follow this workflow for all implementation work — whether interactive or autonomous.

### Test-Driven Development

Write tests **before** implementation. The sequence:

1. Write tests that capture the expected behavior from the spec
2. Run `cargo test --workspace` — confirm tests fail for the right reasons (not compilation errors from missing types, but assertion failures or missing functionality)
3. Implement the minimum code to make tests pass
4. Verify all tests pass (not just the new ones)

### Commit Granularity

Each task should produce 2–3 focused commits:

1. **Tests commit** — the failing tests that define the expected behavior
2. **Implementation commit** — the code that makes them pass, plus any warning fixes
3. **Review fixes commit** (if needed) — issues caught during code review

Each commit should also update bead accordingly, (checked-off todo items, status changes).

### Code Review

After the implementation commit, do a code review before considering the task done.
Prefer spawning a subagent for a fresh perspective — it should review the last 1–2
commits looking for: logic errors, missed edge cases, violations of existing code
patterns, missing test coverage, and clippy-level issues (unnecessary clones, unused
imports, etc.). If a subagent isn't available, self-review by re-reading the full diff.

Fix any real issues found, then commit the fixes separately.

### Acceptance Criteria

Every task must pass before being marked complete:

- All `cargo test --workspace` tests pass
- Zero compiler warnings in `.bacon-claude-diagnostics`
- Bead todo items all checked off
- Bead marked as `completed` with a `## Summary of Changes` section
- Changes committed with descriptive messages
