"""Score a Jev questions file against the labelled examples.

    python3 eval.py [questions file] [threshold]

Asks every question about every file in examples/*/ (one jev call per file,
8 at a time), compares each answer with examples/*/labels.tsv, prints
accuracy, precision, recall and AUC per labelled check, and writes every
score to results/<questions file name>.tsv. A check is scored on the files
whose labels.tsv has a column for it; checks with no labels are not scored.
"""

import csv
import json
import subprocess
import sys
import tomllib
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).parent
QUESTIONS = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "questions/v4.toml"
THRESHOLD = float(sys.argv[2]) if len(sys.argv) > 2 else 0.5


def question_ids(path):
    if path.suffix == ".toml":
        return [q["name"] for q in tomllib.loads(path.read_text())["question"]]
    return list(json.loads(path.read_text()))


def load_labels():
    labels = {}
    for path in sorted(ROOT.glob("examples/*/labels.tsv")):
        with open(path) as f:
            for row in csv.DictReader(f, delimiter="\t"):
                file = (path.parent / row.pop("file")).relative_to(ROOT)
                row.pop("note", None)
                labels[str(file)] = {k: v.strip() == "yes" for k, v in row.items()}
    return labels


def label(file_labels, q):
    if q not in file_labels:
        return "-"
    return "yes" if file_labels[q] else "no"


def ask(file):
    out = subprocess.run(
        ["jev", "-f", file, "-q", str(QUESTIONS.resolve()), "--json"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    reply = json.loads(out)
    return (
        file,
        {k: a["noul"] for k, a in reply["answers"].items()},
        reply["usage"]["cost"],
    )


def main():
    labels = load_labels()
    asked = question_ids(QUESTIONS)
    scored = [q for q in asked if any(q in l for l in labels.values())]
    with ThreadPoolExecutor(8) as pool:
        replies = {f: (s, c) for f, s, c in pool.map(ask, labels)}

    out = ROOT / "results" / f"{QUESTIONS.stem}.tsv"
    with open(out, "w") as f:
        f.write("file\t" + "\t".join(f"{q}_label\t{q}_score" for q in scored) + "\n")
        for file in sorted(replies):
            scores = replies[file][0]
            cells = [
                f"{label(labels[file], q)}\t{scores[q]:.2f}" for q in scored
            ]
            f.write(file + "\t" + "\t".join(cells) + "\n")

    print(
        f"{len(replies)} files, {len(asked)} questions asked, {len(scored)} scored, "
        f"${sum(c for _, c in replies.values()):.4f} -> {out.relative_to(ROOT)}\n"
    )
    print(
        f"{'check':<19}{'yes':>4}{'no':>5}{'acc':>8}{'prec':>7}{'recall':>8}{'AUC':>7}"
        f"{'mean yes':>10}{'mean no':>9}"
    )
    misses = {}
    for q in scored:
        rows = [(labels[f][q], replies[f][0][q], f) for f in replies if q in labels[f]]
        ys = [s for l, s, _ in rows if l]
        ns = [s for l, s, _ in rows if not l]
        tp = sum(s > THRESHOLD for s in ys)
        fp = sum(s > THRESHOLD for s in ns)
        acc = (tp + len(ns) - fp) / len(rows)
        prec = tp / (tp + fp) if tp + fp else float("nan")
        rec = tp / len(ys) if ys else float("nan")
        auc = (
            sum((y > n) + 0.5 * (y == n) for y in ys for n in ns) / (len(ys) * len(ns))
            if ys and ns
            else float("nan")
        )
        print(
            f"{q:<19}{len(ys):>4}{len(ns):>5}{acc:>8.0%}{prec:>7.0%}{rec:>8.0%}{auc:>7.3f}"
            f"{sum(ys) / max(len(ys), 1):>10.2f}{sum(ns) / max(len(ns), 1):>9.2f}"
        )
        misses[q] = sorted((f, l, s) for l, s, f in rows if (s > THRESHOLD) != l)

    print(f"\nwrong at {THRESHOLD}:")
    for q, ms in misses.items():
        for f, l, s in ms:
            print(f"  {q:<19}{f:<40}{'yes' if l else 'no':<5}{s:.2f}")


if __name__ == "__main__":
    main()
