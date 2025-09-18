#!/usr/bin/env bash

desc="
[$(date +"%Y-%m-%d %H:%M:%S %z")]
Run #$(<runcount) took $(<elapsed)

- Checked $(<checked) packages
- Skipped $(<skipped) packages
- Failed to fetch versions for $(<failed) packages
- Updated versions for $(git status -s | grep '^ M ' | grep -o '\w\+/' | tr -d / | uniq | wc -l) packages
"

echo "$desc"
# git add p/*/delay runcount
# git commit -m "Update internal data"
#
# git add p/ALL p/*/{release,unstable,commit}
# git commit -m "Update versions" -m "$desc"
