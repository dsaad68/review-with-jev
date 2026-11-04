# Benchmark: decision models on the Rust checks

Nine models on OpenRouter's decisions endpoint, each asked the 18 checks of
[`rust-example/questions/v4.toml`](../rust-example/questions/v4.toml) (the ones the review skill
uses) about all 206 labelled files in [`rust-example/examples/`](../rust-example/examples/). Run on
2026-11-04 with fuzzy-jev 0.7.0; each model was run once.

## Results

Sorted by mean AUC. A finding counts as a yes above 0.5, as in the review's default.

- **Mean AUC**: how well the scores rank files with the issue above files without it, averaged over
  the 18 checks (1.0 is perfect). It doesn't depend on a threshold.
- **Found**: real issues scored above 0.5, out of the 108 labelled ones across all checks.
  **False alarms**: near misses scored above 0.5, out of 2,568 labelled near misses.
- **F1 at 0.5**: found and false alarms in one number. **Best threshold** is the one, in steps of
  0.05, with the highest F1 on these same files, so **F1 there** is optimistic.
- **Price** is OpenRouter's listed price per million input tokens on the day. **Cost** is what
  OpenRouter charged for all 206 files; one changed function costs about 1/200 of that.
- **Latency** is per file, from this machine, 8 calls at a time (4 for `nace-ai/drex-max`, which
  answered "Model busy" at 8). It includes starting `jev`.

| Model | Mean AUC | F1 at 0.5 | Found | False alarms | Best threshold | F1 there | Price ($/1M input) | Cost (206 files) | Latency p50 / p95 | Calls per file | Failed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `nace-ai/drex-max` | 0.997 | 0.89 | 105/108 | 24 | 0.80 | 0.90 | $0.060 | $0.0706 | 2.2s / 4.0s | 2 | 0 |
| `~liquid/d1-ultra-latest` | 0.997 | 0.93 | 103/108 | 11 | 0.60 | 0.94 | $0.042 | $0.0361 | 0.6s / 1.1s | 1 | 0 |
| `baseten/mammoth-1` | 0.995 | 0.78 | 104/108 | 55 | 0.70 | 0.85 | $0.100 | $0.0901 | 0.6s / 0.7s | 1 | 0 |
| `~typesafe/jev-latest` | 0.991 | 0.70 | 99/108 | 75 | 0.65 | 0.85 | $0.042 | $0.0383 | 0.4s / 0.5s | 1 | 0 |
| `~liquid/d1-latest` | 0.989 | 0.81 | 96/108 | 34 | 0.75 | 0.88 | $0.020 | $0.0172 | 0.5s / 0.7s | 1 | 0 |
| `microsoft/microsoft-decision-1` | 0.987 | 0.54 | 101/108 | 166 | 0.95 | 0.79 | $0.042 | $0.0356 | 0.8s / 1.2s | 1 | 0 |
| `cloudflare/clef` | 0.983 | 0.77 | 98/108 | 47 | 0.60 | 0.85 | $0.192 | $0.2206 | 1.0s / 1.8s | 1 | 0 |
| `openai/gpt-6-luna-decisions` | 0.980 | 0.63 | 92/108 | 91 | 0.85 | 0.83 | $0.100 | $0.1307 | 0.5s / 0.9s | 1 | 0 |
| `baseten/elephant-1` | 0.976 | 0.46 | 85/108 | 175 | 0.80 | 0.67 | $0.035 | $0.0315 | 0.5s / 0.6s | 1 | 0 |

AUC per check:

| Check | drex-max | d1-ultra-latest | mammoth-1 | jev-latest | d1-latest | microsoft-decision-1 | clef | gpt-6-luna-decisions | elephant-1 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `cow` | **1.000** | **1.000** | **1.000** | 0.999 | 0.998 | 0.981 | **1.000** | 0.960 | 0.981 |
| `slice_param` | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | 0.999 | 0.999 | 0.995 | 0.974 |
| `needless_clone` | 0.989 | **0.998** | 0.995 | 0.922 | 0.997 | 0.937 | 0.946 | 0.877 | 0.906 |
| `unwrap` | **1.000** | **1.000** | 0.997 | 0.992 | **1.000** | **1.000** | 0.998 | 0.999 | 0.995 |
| `needless_collect` | **0.966** | 0.960 | 0.932 | 0.957 | 0.930 | 0.897 | 0.894 | 0.875 | 0.914 |
| `quadratic` | **1.000** | 0.999 | **1.000** | 0.999 | 0.997 | **1.000** | 0.998 | **1.000** | 0.998 |
| `lock_scope` | **1.000** | **1.000** | **1.000** | 0.998 | 0.997 | 0.981 | **1.000** | **1.000** | 0.981 |
| `silent_error` | 0.997 | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | 0.998 |
| `stringly_typed` | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** |
| `boolean_trap` | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** |
| `index_loop` | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | 0.993 | 0.993 |
| `manual_reimpl` | **0.998** | 0.988 | 0.993 | 0.995 | 0.896 | 0.990 | 0.989 | 0.956 | 0.875 |
| `double_lookup` | **1.000** | **1.000** | **1.000** | 0.997 | 0.985 | 0.991 | 0.995 | 0.989 | 0.988 |
| `lossy_cast` | **1.000** | **1.000** | **1.000** | **1.000** | 0.997 | 0.998 | 0.998 | **1.000** | 0.986 |
| `unsigned_underflow` | **1.000** | **1.000** | **1.000** | 0.983 | 0.997 | 0.988 | 0.888 | 0.998 | 0.985 |
| `float_misuse` | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | 0.997 | **1.000** | **1.000** |
| `rc_cycle` | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | 0.994 |
| `string_errors` | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** |

Served as:

- `nace-ai/drex-max`: `nace-ai/drex-max-20261016` (2026-11-04)
- `~liquid/d1-ultra-latest`: `liquid/d1-ultra-20261015` (2026-11-04)
- `baseten/mammoth-1`: `baseten/mammoth-1-20261021` (2026-11-04)
- `~typesafe/jev-latest`: `typesafe/jev-1.14-20261014` (2026-11-04)
- `~liquid/d1-latest`: `liquid/d1.1-20261027` (2026-11-04)
- `microsoft/microsoft-decision-1`: `microsoft/microsoft-decision-1-20261009` (2026-11-04)
- `cloudflare/clef`: `cloudflare/clef` (2026-11-04)
- `openai/gpt-6-luna-decisions`: `openai/gpt-6-luna-decisions-20261006` (2026-11-04)
- `baseten/elephant-1`: `baseten/elephant-1-20261021` (2026-11-04)

## What it shows

- **Every model ranks well; they differ in where they put 0.5.** All nine have a mean AUC of 0.976
  or more, so at a well-chosen threshold most get to an F1 of 0.83–0.94. At the review's default
  of 0.5, F1 ranges from 0.46 to 0.93, because some models give near misses scores of 0.5–0.8.
  With those, raise `-t` to the model's best threshold.
- **`~liquid/d1-ultra-latest` did best at the default threshold**: 103 of 108 issues found with 11
  false alarms, for the same price as Jev. Its best threshold is 0.60, close to the default.
- **`nace-ai/drex-max` found the most** (105 of 108) with 24 false alarms. It accepts at most 16
  questions per call, so the 18 checks take two calls per function (the review script and this
  benchmark split them). That doubles its latency, its cost is almost twice Jev's, and it rate-limits
  more than the others.
- **`~typesafe/jev-latest` (Jev 1.14, the default)** ranks well (0.991) but raises 75 false alarms
  at 0.5, 53 of them from `needless_clone` and `needless_collect`. At `-t 0.65` its F1 goes from
  0.70 to 0.85. It had the lowest latency.
- **`~liquid/d1-latest` was the cheapest**, at about half Jev's cost, with an F1 of 0.81 at 0.5.
- **`microsoft/microsoft-decision-1` and `baseten/elephant-1` scored too many near misses above 0.5**:
  166 and 175 false alarms. Microsoft's ranking is still good (0.987, best threshold 0.95);
  Elephant, the 9B model, ranks the worst and found the fewest issues.
- **`cloudflare/clef` cost the most** ($0.22, six times Jev) and was the slowest, with middling results;
  `openai/gpt-6-luna-decisions` found the fewest issues at 0.5 of the large models.
- **The hard checks are the same for every model**: `needless_clone`, `needless_collect`,
  `manual_reimpl` and `cow` cause most false alarms, while `stringly_typed`, `boolean_trap` and
  `string_errors` rank perfectly everywhere.

## Read with care

- **One run, on one day.** The `~...-latest` ids move: "Served as" says which version answered.
  Scores from the same model vary slightly between runs (Jev 1.13 reproduced its October scores
  within 0.007 on average).
- **The test set is small and tidy.** Twelve of the checks have five positive files each, and the
  examples were written from the same definitions the questions use. The ordering of models close
  together, such as the top three, could change on real code.
- **The questions were written and tuned against Jev 1.13.** Other models may do better with
  wording tuned for them.

## Run it

```sh
python3 benchmark/bench.py                      # every model above; skips ones with results
python3 benchmark/bench.py respan/span-02       # add a model
python3 benchmark/bench.py --force -j 4 nace-ai/drex-max   # ask one again, 4 calls at a time
```

It writes each model's scores to `results/<model>.tsv` (the same columns as `eval.py`), its cost,
price, latency, failures and the version that answered to `results/<model>.json`, and the tables
above to [`results/tables.md`](results/tables.md). The nine models cost about $0.67 in all.
