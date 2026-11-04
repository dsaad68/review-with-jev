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

Requires `git`, `sem`, [fuzzy-jev](https://github.com/dsaad68/fuzzy-jev) 0.7+ (the `jev`
command, with `OPENROUTER_API_KEY` set) and `jq`. Run it from inside the repository being reviewed.

**The code of every changed function is sent to OpenRouter.** Before reviewing code the user may
consider private, make sure they are fine with that.

## Choose the model first

The script uses `~liquid/d1-ultra-latest` by default: in this repository's benchmark
(`benchmark/README.md`, November 2026, 206 labelled files) it did best at the default threshold, 103
of 108 issues found with 11 false alarms, at Jev's price. Use another model instead, passed with
`-M`, when the user asks for one, in this order:

1. **The user names a model in the conversation** ("review it with Jev", "use drex-max"). That wins
   for as long as they don't say otherwise.
2. **A `Jev model:` line in `AGENTS.md` or `CLAUDE.md`**, the project's first, then the user's
   global ones, e.g. `Jev model: typesafe/jev-1.14`.
3. Otherwise the default: run the script without `-M`. (`$JEV_MODEL`, if set, also replaces it.)

Say which model reviewed the code when you report the findings. If the user picks a model for good,
offer to add the `Jev model:` line to `AGENTS.md` or `CLAUDE.md`. If they ask what the options are,
run `jev --list-models` (needs a connection, no key) for each model's context window, price and
inputs, or `jev --help` offline. The supported ones in fuzzy-jev 0.7:

| Model | |
| --- | --- |
| `~liquid/d1-ultra-latest`, `liquid/d1-ultra` | Liquid's largest; the review's default. Pin `liquid/d1-ultra` to keep the benchmarked version |
| `~typesafe/jev-latest` | TypeSafe's Jev, the latest version (1.14 since October 2026); jev's own default |
| `typesafe/jev-1.14`, `typesafe/jev-1.13` | Jev pinned to one version. The README's accuracy table is 1.13's |
| `jaredpalmer/kev-4b` | Kev 4B |
| `respan/span-02` | Respan's Span 02 |
| `respan/span-01`, `respan/span-01-lite`, `respan/span-01-lite:free` | yes/no only (fine here); skip the middle of anything past ~3,000 tokens, so long functions are half read |
| `~liquid/d1-latest`, `liquid/d1.1`, `liquid/d1` | Liquid's d1 (`liquid/d1` leaves OpenRouter on 2026-11-20) |
| `~perplexity/pplx-decider-latest`, `perplexity/pplx-decider-v1.2` | Perplexity's Decider |
| `~perplexity/pplx-decider-flash-latest`, `perplexity/pplx-decider-flash-v1.2` | the smaller Decider |
| `baseten/mammoth-1`, `baseten/elephant-1`, `baseten/joey-1` | Baseten's, 27B, 9B and 2B |
| `cloudflare/clef`, `cloudflare/clef-flash`, `cloudflare/clef-omni` | Cloudflare's Clef, 27B, 9B and mixture-of-experts |
| `nace-ai/drex-max`, `nace-ai/drex-v2.0`, `nace-ai/drex-v1.5` | Nace.AI's Drex; `drex-max` takes at most 16 questions a call, so the script splits the checks over two |
| `openai/gpt-6-luna-decisions` | GPT-6 Luna, served for decisions |
| `inception/mercury-decide` | Inception's Mercury Decide |
| `microsoft/microsoft-decision-1` | Microsoft's Decision 1 |
| `upstage/solar-decide`, `upstage/solar-decide-flash` | Upstage's Solar Decide |
| `togethercomputer/tev1-4b-experimental` | Together AI's experimental Tev1 4B |
| `scaledown/classify-1` | ScaleDown's Classify 1 |

Every check is a yes/no question about text, so all of them can run it. From the benchmark, for a
user choosing: `nace-ai/drex-max` found the most (105/108, 24 false alarms, about twice the cost and
four times the latency), `~liquid/d1-latest` was cheapest (half the cost, needs `-t 0.75`), Jev
raises more false alarms at 0.5 (use `-t 0.65`), and `microsoft/microsoft-decision-1` and
`baseten/elephant-1` need `-t 0.95` and `-t 0.80`. With a model other than the default, pass its
best threshold from the benchmark with `-t`. Models not in the benchmark are untested on these
checks (`python3 benchmark/bench.py <id>` in the review-with-jev repository measures one).

## Run

`script/review.sh` is relative to this skill's base directory, wherever the skill is installed:

```sh
S=<this skill's base directory>/script/review.sh
$S                                        # uncommitted changes, Rust, plain text to stdout
$S -- --staged                            # staged changes only
$S -- --from main --to HEAD               # a branch
$S -- --commit abc1234                    # one commit
$S --markdown -o review.md -- --from main # markdown table, saved to a file
$S -M typesafe/jev-1.14 -- --staged       # another model than ~liquid/d1-ultra-latest
```

| Option | |
| --- | --- |
| `-m`, `--markdown` | A markdown table instead of plain text, e.g. for a PR comment or a file. |
| `-M ID`, `--model ID` | The model (default `$JEV_MODEL`, else `~liquid/d1-ultra-latest`). Quote a `~` id. |
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

- **Above ~0.9 is usually real.** In the demo, `~liquid/d1-ultra-latest` scored most planted issues
  0.91–0.99 (Jev 0.9–0.97).
- **0.5–0.7 is where the false positives are.** Check these before mentioning them.
- **`unwrap` fires on lock, join and channel-send unwraps**, which the question says do not count.
- **Callers inherit their callees' findings**: a `main` that calls a function returning
  `Result<_, String>` may itself be flagged for `string_errors`. Report the finding on the callee.
- **Context-dependent issues score low** even when real: `lossy_cast` (is the value ever negative?)
  and `lock_scope` (who else waits on the lock?) came in at 0.78–0.80 on the default model and
  0.56–0.62 on Jev. Read the surrounding code.
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
