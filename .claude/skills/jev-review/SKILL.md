---
name: jev-review
description: Review the functions changed in a diff for code-quality issues the compiler does not catch — unnecessary clones and allocations (Cow, &String params, needless collect), panics on recoverable errors, silently dropped errors, O(n²) loops, locks held across slow work, lossy `as` casts, unsigned underflow, floats for money, Rc cycles, stringly-typed state, boolean-trap parameters and more. Uses sem to find changed functions and Jev to score every check per function, in parallel, for fractions of a cent. Use when asked to review, check or audit changes, a branch, a commit or a PR for quality, or to find what could be improved in changed Rust code; also before committing when the user wants a quality pass.
---

# jev-review: score changed functions against quality checks

`script/review.sh` runs three steps:

1. `sem diff --format json` lists every added or modified function with its new code.
2. Each function goes to fuzzy-jev (`jev`) with every question in `reference/<lang>.toml`, 16 calls at a time.
   Every question is a yes/no Noul, so each answer is the probability that the issue is present.
3. The answers above the threshold are printed per function: `file:line`, then each finding's
   score, check id and what could be improved. `--markdown` prints the same as a table.

Requires `git`, `sem`, [fuzzy-jev](https://github.com/dsaad68/fuzzy-jev) 0.6+ (the `jev`
command, with `OPENROUTER_API_KEY` set) and `jq`. Run it from inside the repository being reviewed.

**The code of every changed function is sent to OpenRouter.** Before reviewing code the user may
consider private, make sure they are fine with that.

## Run

`script/review.sh` is relative to this skill's base directory, wherever the skill is installed:

```sh
S=<this skill's base directory>/script/review.sh
$S                                        # uncommitted changes, Rust, plain text to stdout
$S -- --staged                            # staged changes only
$S -- --from main --to HEAD               # a branch
$S -- --commit abc1234                    # one commit
$S --markdown -o review.md -- --from main # markdown table, saved to a file
```

| Option | |
| --- | --- |
| `-m`, `--markdown` | A markdown table instead of plain text, e.g. for a PR comment or a file. |
| `-l LANG` | Questions from `reference/LANG.toml` (default `rust`). |
| `-q FILE` | Any other questions file. |
| `-t N` | Report answers above N (default 0.5). |
| `-o FILE` | Write the report to FILE instead of stdout. |
| `-j N` | Parallel Jev calls (default 16). |
| `-e EXT` | Only files ending in EXT (default: the TOML's `# ext:` line). |
| `-- ARGS` | Passed to `sem diff`. |

The report goes to stdout; a summary line (`jev: 22 findings, $0.0023, 1s`) and any calls that
failed after three tries go to stderr. About $0.0002 and well under a second per function.

```text
src/store.rs:37  load
  0.97  double_lookup       Use the entry API instead of looking the key up twice
  0.87  silent_error        Report or count errors instead of silently dropping them

src/main.rs:1  last_word
  0.94  unsigned_underflow  Guard the unsigned subtraction (checked_sub / saturating_sub)
```

With `--markdown`:

```text
| Function | File | Could be improved |
| --- | --- | --- |
| `load` | `src/store.rs:37` | Use the `entry` API instead of looking the key up twice (`double_lookup` 0.97)<br>Report or count errors instead of silently dropping them (`silent_error` 0.87) |
```

## Read the results as leads, not verdicts

Each finding is a probability from a model that saw **only that one function**: not its callers, not
the types it uses. Before reporting a finding to the user or acting on it, open the function at
`file:line` and confirm it. Known patterns from testing on labelled examples:

- **Above ~0.9 is usually real.** Planted issues scored 0.9–0.97.
- **0.5–0.7 is where the false positives are.** Check these before mentioning them.
- **`unwrap` fires on lock, join and channel-send unwraps**, which the question says do not count.
- **Callers inherit their callees' findings**: a `main` that calls a function returning
  `Result<_, String>` may itself be flagged for `string_errors`. Report the finding on the callee.
- **Context-dependent issues score low** even when real: `lossy_cast` (is the value ever negative?)
  and `lock_scope` (who else waits on the lock?) came in at 0.56–0.62. Read the surrounding code.
- **`cow` and `needless_clone` overlap**: a `v.clone()` on a return path can trip both.
- **A changed function brings its old problems with it.** Findings may predate the diff; say so if
  the user asked only about their change.

For the exact definition of a check, read its `instruction` and `criteria` in `reference/<lang>.toml`.
Do not reword findings into certainties: "Jev scored `silent_error` 0.85 on `load`; the
`if let Ok(n) = score.parse()` on line 41 drops bad lines without a log" is the useful form.

## Add or change checks

`reference/<lang>.toml` is a fuzzy-jev questions file (`[[question]]` tables with `name`, `type`,
`instruction` and `[question.criteria]` `true`/`false`). fuzzy-jev rejects unknown keys, so the script
reads two comment conventions instead:

- `# ext: .rs`: which changed files this language covers.
- `# hint: <text>` on the line before a `[[question]]`: the suggestion shown when it is a yes.
  Without one, the report shows the check id.

To add a language, copy `rust.toml` to `<lang>.toml`, change `# ext:` and the questions. After any
edit, validate for free with `jev --dry-run -q reference/<lang>.toml 'x' > /dev/null`. Write the
`instruction` so it says what does *not* count; near misses are where these checks go wrong.
