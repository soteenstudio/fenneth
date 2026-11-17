#!/bin/sh
# Fails when an English page in docs/ has no Indonesian twin in docs/id/ (or
# the other way around). English lives at the docs root, Indonesian in id/.
set -eu
cd "$(dirname "$0")/../docs"

en="$(find . -name '*.md' -not -path './id/*' -not -path './node_modules/*' -not -path './.vitepress/*' | sed 's#^\./##' | sort)"
id="$(cd id && find . -name '*.md' | sed 's#^\./##' | sort)"

if [ "$en" != "$id" ]; then
    echo "docs: English (docs/) and Indonesian (docs/id/) pages do not match:" >&2
    printf '%s\n' "$en" > /tmp/docs-en.txt
    printf '%s\n' "$id" > /tmp/docs-id.txt
    diff /tmp/docs-en.txt /tmp/docs-id.txt >&2 || true
    exit 1
fi
echo "docs: $(printf '%s\n' "$en" | wc -l | tr -d ' ') pages in each language"
