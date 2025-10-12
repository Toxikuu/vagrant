#!/usr/bin/env bash

git add runcount
git commit -m "auto(aux): update internal data"

git add p/ALL.* p/*/versions.* p/*/channels/*

packages_updated=$(git status -s p | grep -E '^( M|A |\?\?)' | grep -vF ALL | cut -d/ -f2 | uniq | wc -l)
versions_updates=$(git status -s p | grep -E '^( M|A |\?\?)' | cut -d/ -f2- | uniq | grep -F channels/)

release_versions_updates=$(echo "$versions_updates" | grep -c 'channels/release$')
unstable_versions_updates=$(echo "$versions_updates" | grep -c 'channels/unstable$')
commit_versions_updates=$(echo "$versions_updates" | grep -c 'channels/commit$')
other_versions_updates=$(echo "$versions_updates" | grep -Evc 'channels/(release|unstable|commit)$')

versions_updates=$(echo "$versions_updates" | wc -l)

desc="
[$(date +"%Y-%m-%d %H:%M:%S %z")]
Run #$(<runcount) took $(<elapsed)

- Checked $(<checked) packages
- Skipped $(<skipped) packages
- Failed to fetch versions for $(<failed) packages

- Updated $((versions_updates)) versions for $packages_updated packages:
    - Release versions:  $release_versions_updates
    - Unstable versions: $unstable_versions_updates
    - Commit versions:   $commit_versions_updates
    - Other versions:    $other_versions_updates
"

echo "$desc"

git commit -m "auto(p): update versions" -m "$desc"
