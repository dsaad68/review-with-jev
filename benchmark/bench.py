"""Benchmark decision models on the labelled Rust examples.

    python3 benchmark/bench.py [--force] [-j N] [-q questions.toml] [MODEL ...]

Asks every question in the questions file (default rust-example/questions/v4.toml,
the 18 checks the review skill uses) about every file in rust-example/examples/*/,
once per model, N calls at a time (default 8). A call that fails is tried three
times (six if the model says it is busy), then counted as failed and left out
of the scores.

Each model's scores go to benchmark/results/<model>.tsv (the same columns as
eval.py writes) and its cost, latency and failures to <model>.json. A model with
results already there is not asked again unless --force is given, so adding a
model to the list costs only that model. Then the comparison tables are written
to benchmark/results/tables.md and printed.
"""

import argparse
import csv
import json
import statistics
import math
import subprocess
import tempfile
import time
import tomllib
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).parent
EXAMPLES = HERE.parent / "rust-example"
RESULTS = HERE / "results"
THRESHOLD = 0.5

MODELS = [
    "~typesafe/jev-latest",
    "~liquid/d1-ultra-latest",
    "~liquid/d1-latest",
    "nace-ai/drex-max",
    "openai/gpt-6-luna-decisions",
    "microsoft/microsoft-decision-1",
    "baseten/mammoth-1",
    "baseten/elephant-1",
    "cloudflare/clef",
]


# Models that refuse more questions than this in one call: their questions are
# split over several calls per file. Each question is answered on its own, so
# this changes the cost and the time, not the scores.
MAX_QUESTIONS = {"nace-ai/drex-max": 16, "parallel/decisions-fast:free": 100, "morph/reflex": 128}


def split(questions, model, workdir):
    """The questions file, or JSON files of at most MAX_QUESTIONS[model] questions each."""
    limit = MAX_QUESTIONS.get(model)
    qs = tomllib.loads(questions.read_text())["question"]
    if not limit or len(qs) <= limit:
        return [questions]
    n = math.ceil(len(qs) / limit)
    size = math.ceil(len(qs) / n)
    parts = []
    for i in range(n):
        chunk = {
            q["name"]: {"type": q["type"], "instructions": q["instruction"], "criteria": q["criteria"]}
            for q in qs[i * size : (i + 1) * size]
        }
        path = Path(workdir) / f"part{i + 1}.json"
        path.write_text(json.dumps(chunk))
        parts.append(path)
    return parts


def slug(model):
    return model.lstrip("~").replace("/", "_").replace(":", "-")


def load_labels():
    labels = {}
    for path in sorted(EXAMPLES.glob("examples/*/labels.tsv")):
        with open(path) as f:
            for row in csv.DictReader(f, delimiter="\t"):
                file = (path.parent / row.pop("file")).relative_to(EXAMPLES)
                row.pop("note", None)
                labels[str(file)] = {k: v.strip() == "yes" for k, v in row.items()}
    return labels


def ask(model, parts, file):
    """One file, every part's questions in turn: their answers, cost and time put together."""
    merged = {"scores": {}, "cost": 0, "seconds": 0, "served": ""}
    for questions in parts:
        r = ask_part(model, questions, file)
        if "error" in r:
            return file, r
        merged["scores"].update(r["scores"])
        merged["cost"] += r["cost"]
        merged["seconds"] += r["seconds"]
        merged["served"] = r["served"]
    return file, merged


def ask_part(model, questions, file):
    err = ""
    attempt = 0
    while attempt < (6 if "HTTP 429" in err else 3):
        start = time.monotonic()
        p = subprocess.run(
            ["jev", "-m", model, "-f", file, "-q", str(questions), "--json"],
            cwd=EXAMPLES,
            capture_output=True,
            text=True,
        )
        took = time.monotonic() - start
        if p.returncode == 0:
            try:
                reply = json.loads(p.stdout)
                return {
                    "scores": {k: a["noul"] for k, a in reply["answers"].items()},
                    "cost": reply.get("usage", {}).get("cost") or 0,
                    "seconds": took,
                    "served": reply.get("model", ""),
                }
            except (ValueError, KeyError, TypeError) as e:
                err = f"unreadable reply: {e}"
        else:
            err = p.stderr.strip() or f"exit {p.returncode}"
        attempt += 1
        time.sleep(2**attempt)
    return {"error": err}


def prices():
    """{model id: dollars per million input tokens} from OpenRouter's listing now, or {}."""
    try:
        p = subprocess.run(["jev", "--list-models", "--json"], capture_output=True, text=True, timeout=60)
        return {m["id"]: m["price"] for m in json.loads(p.stdout) if m.get("price") is not None}
    except (OSError, ValueError, KeyError, TypeError, subprocess.TimeoutExpired):
        return {}


def run(model, questions, labels, scored, jobs):
    start = time.monotonic()
    with tempfile.TemporaryDirectory() as workdir, ThreadPoolExecutor(jobs) as pool:
        parts = split(questions, model, workdir)
        replies = dict(pool.map(lambda f: ask(model, parts, f), labels))
    wall = time.monotonic() - start
    ok = {f: r for f, r in replies.items() if "error" not in r}
    failed = {f: r["error"] for f, r in replies.items() if "error" in r}

    with open(RESULTS / f"{slug(model)}.tsv", "w") as f:
        f.write("file\t" + "\t".join(f"{q}_label\t{q}_score" for q in scored) + "\n")
        for file in sorted(ok):
            s = ok[file]["scores"]
            cells = [f"{label(labels[file], q)}\t{s[q]:.2f}" for q in scored]
            f.write(file + "\t" + "\t".join(cells) + "\n")

    secs = sorted(r["seconds"] for r in ok.values())
    meta = {
        "model": model,
        "served": sorted({r["served"] for r in ok.values() if r["served"]}),
        "date": time.strftime("%Y-%m-%d"),
        "questions": str(questions.relative_to(EXAMPLES.parent)),
        "calls_per_file": len(parts),
        "files": len(labels),
        "answered": len(ok),
        "failed": failed,
        "price_per_million": prices().get(model),
        "cost": round(sum(r["cost"] for r in ok.values()), 6),
        "wall_seconds": round(wall, 1),
        "latency_p50": round(statistics.median(secs), 2) if secs else None,
        "latency_p95": round(secs[int(0.95 * (len(secs) - 1))], 2) if secs else None,
    }
    (RESULTS / f"{slug(model)}.json").write_text(json.dumps(meta, indent=2) + "\n")
    return meta


def label(file_labels, q):
    if q not in file_labels:
        return "-"
    return "yes" if file_labels[q] else "no"


def read_scores(model):
    """{check: [(label, score)]} from a model's TSV."""
    out = {}
    with open(RESULTS / f"{slug(model)}.tsv") as f:
        for row in csv.DictReader(f, delimiter="\t"):
            for k, v in row.items():
                if k.endswith("_label") and v in ("yes", "no"):
                    q = k[: -len("_label")]
                    out.setdefault(q, []).append((v == "yes", float(row[q + "_score"])))
    return out


def auc(rows):
    ys = [s for l, s in rows if l]
    ns = [s for l, s in rows if not l]
    if not ys or not ns:
        return float("nan")
    return sum((y > n) + 0.5 * (y == n) for y in ys for n in ns) / (len(ys) * len(ns))


def counts(rows, t):
    tp = sum(l and s > t for l, s in rows)
    fp = sum((not l) and s > t for l, s in rows)
    fn = sum(l and s <= t for l, s in rows)
    return tp, fp, fn


def f1(tp, fp, fn):
    return 2 * tp / (2 * tp + fp + fn) if tp else 0.0


def summary(model):
    scores = read_scores(model)
    meta = json.loads((RESULTS / f"{slug(model)}.json").read_text())
    tp, fp, fn = map(sum, zip(*(counts(r, THRESHOLD) for r in scores.values())))
    grid = [i / 20 for i in range(1, 20)]
    best_t = max(
        grid,
        key=lambda t: (f1(*map(sum, zip(*(counts(r, t) for r in scores.values())))), -abs(t - 0.5)),
    )
    btp, bfp, bfn = map(sum, zip(*(counts(r, best_t) for r in scores.values())))
    return {
        "model": model,
        "meta": meta,
        "auc": {q: auc(r) for q, r in scores.items()},
        "mean_auc": statistics.mean(auc(r) for r in scores.values()),
        "tp": tp, "fp": fp, "fn": fn,
        "f1": f1(tp, fp, fn),
        "best_t": best_t, "best_f1": f1(btp, bfp, bfn), "best_fp": bfp, "best_fn": bfn,
    }


def tables(models):
    dead = [m for m in models if not read_scores(m)]
    rows = sorted((summary(m) for m in models if m not in dead), key=lambda s: (-s["mean_auc"], -s["f1"]))
    out = [
        f"| Model | Mean AUC | F1 at {THRESHOLD} | Found | False alarms | Best threshold | F1 there "
        "| Price ($/1M input) | Cost (206 files) | Latency p50 / p95 | Calls per file | Failed |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    listed = prices() if any(s["meta"].get("price_per_million") is None for s in rows) else {}
    for s in rows:
        m = s["meta"]
        price = m.get("price_per_million", listed.get(s["model"]))
        price = "-" if price is None else "free" if price == 0 else f"${price:.3f}"
        lat = f"{m['latency_p50']:.1f}s / {m['latency_p95']:.1f}s" if m["latency_p50"] is not None else "-"
        out.append(
            f"| `{s['model']}` | {s['mean_auc']:.3f} | {s['f1']:.2f} | {s['tp']}/{s['tp'] + s['fn']} "
            f"| {s['fp']} | {s['best_t']:.2f} | {s['best_f1']:.2f} | {price} | ${m['cost']:.4f} | {lat} "
            f"| {m.get('calls_per_file', 1)} | {len(m['failed'])} |"
        )
    checks = list(rows[0]["auc"])
    names = [s["model"].lstrip("~").split("/")[-1] for s in rows]
    out += ["", "AUC per check:", "", "| Check | " + " | ".join(names) + " |",
            "| --- |" + " --- |" * len(names)]
    for q in checks:
        vals = [round(s["auc"][q], 3) for s in rows]
        top = max(vals)
        cells = [f"**{v:.3f}**" if v == top else f"{v:.3f}" for v in vals]
        out.append(f"| `{q}` | " + " | ".join(cells) + " |")
    served = [f"- `{s['model']}`: {', '.join(f'`{x}`' for x in s['meta']['served']) or 'not reported'}"
              f" ({s['meta']['date']})" for s in rows]
    out += ["", "Served as:", ""] + served
    for m in dead:
        meta = json.loads((RESULTS / f"{slug(m)}.json").read_text())
        err = next(iter(meta["failed"].values()), "no answers").replace("\n", " ")
        out.append(f"- `{m}`: no file answered ({err[:200]})")
    return "\n".join(out) + "\n"


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("models", nargs="*", default=MODELS)
    p.add_argument("--force", action="store_true", help="ask models that already have results again")
    p.add_argument("-j", type=int, default=8, help="parallel calls per model (default 8)")
    p.add_argument("-q", type=Path, default=EXAMPLES / "questions/v4.toml")
    a = p.parse_args()

    RESULTS.mkdir(exist_ok=True)
    questions = a.q.resolve()
    labels = load_labels()
    asked = [q["name"] for q in tomllib.loads(questions.read_text())["question"]]
    scored = [q for q in asked if any(q in l for l in labels.values())]

    for model in a.models:
        if (RESULTS / f"{slug(model)}.json").exists() and not a.force:
            print(f"{model}: results already there, skipped (--force to ask again)")
            continue
        m = run(model, questions, labels, scored, a.j)
        print(f"{model}: {m['answered']}/{m['files']} files, ${m['cost']:.4f}, "
              f"{m['wall_seconds']}s, {len(m['failed'])} failed")

    every = MODELS + [m for m in a.models if m not in MODELS]
    done = [m for m in every if (RESULTS / f"{slug(m)}.json").exists()]
    text = tables(done)
    (RESULTS / "tables.md").write_text(text)
    print("\n" + text)


if __name__ == "__main__":
    main()
