#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<EOF
Usage: $(basename "$0") [-m] [-l lang] [-q questions.toml] [-t threshold] [-o file] [-j jobs] [-e ext] [-- sem diff args...]

Finds changed functions with sem, asks Jev every question about each one in
parallel, and prints what could be improved in each.

  -m, --markdown  print a markdown table instead of plain text
  -l  language: reference/<lang>.toml               (default rust; available: $(available))
  -q  questions file instead of -l
  -t  report answers above this probability          (default 0.5)
  -o  write the report here instead of stdout
  -j  parallel Jev calls                             (default 16)
  -e  only review files with this extension          (default: the file's '# ext:' line)

Anything after -- is passed to 'sem diff' (default: uncommitted changes), e.g.
  $(basename "$0") -- --staged
  $(basename "$0") -t 0.7 -- --from main --to HEAD
EOF
}

SKILL_DIR=$(cd "$(dirname "$0")/.." && pwd)
available() { ls "$SKILL_DIR/reference" 2>/dev/null | sed -n 's/\.toml$//p' | paste -sd, -; }

LANG_NAME=rust
QUESTIONS=
THRESHOLD=0.5
OUT=-
JOBS=16
EXT=
EXT_SET=
FORMAT=text

ARGS=()
while [ $# -gt 0 ]; do
  case $1 in
    --markdown) ARGS+=(-m) ;;
    --help) ARGS+=(-h) ;;
    --) ARGS+=("$@"); break ;;
    *) ARGS+=("$1") ;;
  esac
  shift
done
set -- ${ARGS[@]+"${ARGS[@]}"}

while getopts "ml:q:t:o:j:e:h" opt; do
  case $opt in
    m) FORMAT=markdown ;;
    l) LANG_NAME=$OPTARG ;;
    q) QUESTIONS=$OPTARG ;;
    t) THRESHOLD=$OPTARG ;;
    o) OUT=$OPTARG ;;
    j) JOBS=$OPTARG ;;
    e) EXT=$OPTARG; EXT_SET=1 ;;
    h) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
  esac
done
shift $((OPTIND - 1))
[ "${1:-}" = "--" ] && shift
SEM_ARGS=("$@")

for cmd in git sem jev jq; do
  command -v "$cmd" >/dev/null || { echo "missing: $cmd" >&2; exit 1; }
done

QUESTIONS=${QUESTIONS:-$SKILL_DIR/reference/$LANG_NAME.toml}
[ -f "$QUESTIONS" ] || { echo "no questions file: $QUESTIONS (available: $(available))" >&2; exit 1; }
QUESTIONS=$(cd "$(dirname "$QUESTIONS")" && pwd)/$(basename "$QUESTIONS")
[ -n "$EXT_SET" ] || EXT=$(sed -n 's/^# ext: *//p' "$QUESTIONS" | head -1)

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/fn"

jev --dry-run -q "$QUESTIONS" 'check' >/dev/null 2>"$WORK/preflight.err" || {
  echo "jev rejected $QUESTIONS:" >&2; cat "$WORK/preflight.err" >&2; exit 1
}

awk '
  /^# hint:/ { sub(/^# hint: */, ""); hint = $0; next }
  /^name *=/ && hint != "" { sub(/^name *= *"/, ""); sub(/".*$/, ""); print $0 "\t" hint; hint = "" }
' "$QUESTIONS" | jq -Rn '[inputs | split("\t") | {(.[0]): .[1]}] | add // {}' > "$WORK/hints.json"

START=$SECONDS
REPO=$(git rev-parse --show-toplevel)

(cd "$REPO" && sem diff --format json ${SEM_ARGS[@]+"${SEM_ARGS[@]}"}) > "$WORK/diff.json"

jq -c --arg ext "$EXT" '
  .changes[]
  | select(.changeType == "added" or .changeType == "modified")
  | select(.entityType == "function" or .entityType == "method")
  | select($ext == "" or (.filePath | endswith($ext)))
  | {file: .filePath, name: .entityName, change: .changeType,
     start: .startLine, end: .endLine, code: .afterContent}
' "$WORK/diff.json" > "$WORK/entities.jsonl"

N=$(wc -l < "$WORK/entities.jsonl" | tr -d ' ')
echo "sem: $N changed functions" >&2

jq -r --arg dir "$WORK/fn" '
  "printf %s \(.code | @sh) > \($dir + "/" + (input_line_number | tostring) + ".code" | @sh)"
' "$WORK/entities.jsonl" | sh

if [ "$N" -gt 0 ]; then
  ls "$WORK"/fn/*.code | xargs -P "$JOBS" -I{} sh -c '
    code=$1; questions=$2; base=${code%.code}
    for attempt in 1 2 3; do
      jev -f "$code" -q "$questions" --json > "$base.out" 2> "$base.err" && exit 0
      sleep "$attempt"
    done
    jq -Rs "{error: (if . == \"\" then \"no reply\" else . end)}" < "$base.err" > "$base.out"
  ' _ {} "$QUESTIONS"

  jq -nc --slurpfile e "$WORK/entities.jsonl" '
    (reduce inputs as $o ({};
      .[input_filename | capture("(?<i>[0-9]+)\\.out$").i] = $o)) as $r
    | $e | to_entries[]
    | ($r[(.key + 1) | tostring]) as $o
    | .value | del(.code)
    + if $o == null then {error: "no reply"}
      elif $o.error then {error: $o.error}
      else {scores: ($o.answers | map_values(.noul)), cost: ($o.usage.cost // 0)}
      end
  ' "$WORK"/fn/*.out
fi > "$WORK/results.jsonl"

ELAPSED=$((SECONDS - START))

jq -rs --argjson t "$THRESHOLD" --arg format "$FORMAT" --slurpfile h "$WORK/hints.json" '
  def score: . * 100 | round | tostring | if length < 2 then "0.0" + . elif . == "100" then "1.00" else "0." + . end;

  $h[0] as $hint
  | [ .[] | select(.error | not)
      | . as $f
      | [ .scores | to_entries[] | select(.value > $t) ] | sort_by(-.value)
      | select(length > 0)
      | { name: $f.name, where: "\($f.file):\($f.start)", top: .[0].value,
          items: map({check: .key, score: (.value | score), hint: ($hint[.key] // .key)}) } ]
  | sort_by(-.top)
  | if $format == "markdown" then
      "# Code review",
      "",
      (if length == 0 then "Nothing to improve above \($t)."
       else
         "| Function | File | Could be improved |",
         "| --- | --- | --- |",
         (.[] | "| `\(.name)` | `\(.where)` | \(.items | map("\(.hint) (`\(.check)` \(.score))") | join("<br>")) |")
       end)
    else
      (if length == 0 then "Nothing to improve above \($t)."
       else
         (([.[].items[].check | length] | max) as $w
          | .[] | "\(.where)  \(.name)",
            (.items[] | "  \(.score)  \(.check + (" " * ($w - (.check | length))))  \(.hint | gsub("`"; ""))"),
            "")
       end)
    end
' "$WORK/results.jsonl" > "$WORK/report"

if [ "$OUT" = "-" ]; then cat "$WORK/report"; else cp "$WORK/report" "$OUT"; fi

jq -rs --argjson t "$THRESHOLD" --arg out "$OUT" --arg elapsed "$ELAPSED" '
  "jev: \([.[].scores // {} | .[] | select(. > $t)] | length) findings, $\([.[].cost // 0] | add // 0 | . * 100000 | round / 100000), \($elapsed)s -> \(if $out == "-" then "stdout" else $out end)",
  (.[] | select(.error) | "failed: \(.file):\(.start) \(.name): \(.error | gsub("\n"; " "))")
' "$WORK/results.jsonl" >&2
