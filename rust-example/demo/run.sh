#!/usr/bin/env bash
set -euo pipefail

# Runs the jev-review skill on a pull-request-sized change with known issues.
#
# before/ is five example files; after/ is the same files with ten issues
# planted (listed in planted.tsv) plus two clean changes. This script commits
# before/ to a throwaway git repo, copies after/ over it, and reviews the
# uncommitted change. Arguments are passed to review.sh, e.g. --markdown.

HERE=$(cd "$(dirname "$0")" && pwd)
REVIEW="$HERE/../../.claude/skills/jev-review/script/review.sh"
REPO=$(mktemp -d)
trap 'rm -rf "$REPO"' EXIT

cp "$HERE"/before/*.rs "$REPO"/
git -C "$REPO" init -q
git -C "$REPO" add .
git -C "$REPO" -c user.name=demo -c user.email=demo@example.com commit -qm before
cp "$HERE"/after/*.rs "$REPO"/

cd "$REPO"
"$REVIEW" "$@"
