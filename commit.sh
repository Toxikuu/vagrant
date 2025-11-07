#!/usr/bin/env bash

argv0="$0"
set -eu

die() {
    printf "%s: %s\n" "$argv0" "$1"
    exit "${2:-1}"
}

git add runcount
git commit -m "auto(aux): update internal data"

git add p/ALL.* p/*/versions.* p/*/channels/*

packages_updated=$(git status -s p | grep -vF ALL | cut -d/ -f2 | uniq | wc -l)
versions_updates=$(git status -s p | cut -d/ -f2- | uniq | grep -F channels/)

release_versions_updates=$(echo "$versions_updates" | grep -c 'channels/release$')
unstable_versions_updates=$(echo "$versions_updates" | grep -c 'channels/unstable$')
commit_versions_updates=$(echo "$versions_updates" | grep -c 'channels/commit$')
other_versions_updates=$(echo "$versions_updates" | grep -Evc 'channels/(release|unstable|commit)$')

versions_updated=$(echo "$versions_updates" | wc -l)
vagrant_version=$(git describe --tags || echo "???")

cd .vagrant-cache || die "Couldn't access .vagrant-cache"
desc="
[Vagrant v$vagrant_version - $(date +"%Y-%m-%d %H:%M:%S %z")]
Run #$(<../runcount) took $(<elapsed)

- Processed $(<total) packages
    - Checked   $(<checked)
    - Skipped   $(<skipped)
    - Failed    $(<failed)

- Updated $((versions_updated)) versions for $packages_updated packages:
    - Release   $release_versions_updates
    - Unstable  $unstable_versions_updates
    - Commit    $commit_versions_updates
    - Other     $other_versions_updates
"
# shellcheck disable=2103
cd ..

echo "$desc"
git commit -m "auto(p): update versions" -m "$desc"
