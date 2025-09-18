#!/usr/bin/env bash

packages_updated=$(git status -s p | grep '^ M' | grep -Ev 'ALL|delay' | cut -d/ -f2 | uniq | wc -l)

release_versions_updates=$(git status -s p | grep '^ M' | cut -d/ -f2- | uniq | grep release | wc -l)
unstable_versions_updates=$(git status -s p | grep '^ M' | cut -d/ -f2- | uniq | grep unstable | wc -l)
commit_versions_updates=$(git status -s p | grep '^ M' | cut -d/ -f2- | uniq | grep commit | wc -l)

desc="
[$(date +"%Y-%m-%d %H:%M:%S %z")]
Run #$(<runcount) took $(<elapsed)

- Checked $(<checked) packages
- Skipped $(<skipped) packages
- Failed to fetch versions for $(<failed) packages

- Updated $(($release_versions_updates + $unstable_versions_updates + $commit_versions_updates)) versions for $packages_updated packages:
    - Release versions:  $release_versions_updates
    - Unstable versions: $unstable_versions_updates
    - Commit versions:   $commit_versions_updates
"

echo "$desc"
# git add p/*/delay runcount
# git commit -m "auto(aux): update internal data"
#
# git add p/ALL p/*/{release,unstable,commit}
# git commit -m "auto(p): update versions" -m "$desc"
