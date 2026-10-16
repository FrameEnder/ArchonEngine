#!/usr/bin/env bash
# Checks the files a push or pull request changed (BASE..HEAD), for the fork's own rules:
#   * changed Rust files are formatted (rustfmt on just those files: running cargo fmt on the whole workspace would
#     reformat upstream's files, so it isn't used);
#   * a change outside docs/ comes with an entry in docs/src/fork/changelog.md (see CLAUDE.md), unless a commit
#     message in the range says [no-docs] (upstream merges, for one).
# Usage: tools/ci/check-changes.sh BASE HEAD
set -euo pipefail

base="${1:-}"
head="${2:-HEAD}"
if [[ -z "$base" || "$base" =~ ^0+$ ]] || ! git cat-file -e "$base^{commit}" 2>/dev/null; then
    # A new branch (or a shallow history): check the last commit only.
    base="$(git rev-parse "$head~1")"
fi

changed="$(git diff --name-only --diff-filter=ACMR "$base" "$head")"
echo "Changed files ($base..$head):"
echo "$changed" | sed 's/^/  /'
status=0

rust_files="$(echo "$changed" | grep -E '\.rs$' | grep -v -E '^core/src/(opcodes|constants|festivals)\.rs$' || true)"
if [[ -n "$rust_files" ]]; then
    echo "Checking formatting of the changed Rust files"
    # shellcheck disable=SC2086
    if ! rustfmt --check --edition 2024 $rust_files; then
        echo "::error::Some changed Rust files aren't formatted: run rustfmt --edition 2024 on them (not cargo fmt)."
        status=1
    fi
fi

outside_docs="$(echo "$changed" | grep -v -E '^(docs/|$)' || true)"
if [[ -n "$outside_docs" ]] && ! echo "$changed" | grep -qx 'docs/src/fork/changelog.md'; then
    if git log --format=%B "$base..$head" | grep -q '\[no-docs\]'; then
        echo "No changelog entry, but a commit says [no-docs]."
    else
        echo "::error::Code changed without an entry in docs/src/fork/changelog.md (see CLAUDE.md). Put [no-docs] in a commit message to skip this, e.g. for an upstream merge."
        status=1
    fi
fi

exit $status
