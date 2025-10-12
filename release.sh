#!/usr/bin/env bash

set -eu
argv0="$0"

die() {
    printf "%s: %s" "$argv0" "$1"
    exit "${2:-1}"
}

# Checks
[[ -z "$(git status -s)" ]] || die "Uncommitted changes"
cargo clippy || die "Clippy failed"
cargo build --release || die "Build failed"

# Get old semver
old_tag=$(git describe --tags --abbrev=0 @^)
old_tag_major=$(echo "$old_tag" | cut -d. -f1)
old_tag_minor=$(echo "$old_tag" | cut -d. -f2)
old_tag_patch=$(echo "$old_tag" | cut -d. -f3)

changes=$(git log --pretty=%s "$old_tag"..)

# Check for breaking changes, and determine new semver
if echo "$changes" | grep '^!!'; then
    new_tag_major=$((old_tag_major + 1))
    new_tag_minor=0
    new_tag_patch=0
elif echo "$changes" | grep '^!'; then
    new_tag_major=$old_tag_major
    new_tag_minor=$((old_tag_minor + 1))
    new_tag_patch=0
else
    new_tag_major=$old_tag_major
    new_tag_minor=$old_tag_minor
    new_tag_patch=$((old_tag_patch + 1))
fi

new_tag="$new_tag_major.$new_tag_minor.$new_tag_patch"

# Update Cargo version
old_sum=$(sha256sum Cargo.toml)
sed -i "s|version = \"$old_tag\"|version = \"$new_tag\"|"
new_sum=$(sha256sum Cargo.toml)

if [[ "$old_sum" == "$new_sum" ]]; then
    die "Failed to update version in Cargo.toml"
fi

cargo build --release

# Parse changes
# NOTE: Automatic commits are excluded
echo "$changes" | while IFS= read -r change; do
    if echo "$change" | grep -q "feat.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        features+="${msg^}\n"
        continue
    fi

    if echo "$change" | grep -q "fix.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        fixes+="${msg^}\n"
        continue
    fi

    if echo "$change" | grep -q "chore.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        chores+="${msg^}\n"
        continue
    fi

    if echo "$change" | grep -q "ci.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        cis+="${msg^}\n"
        continue
    fi

    if echo "$change" | grep -q "docs.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        docs+="${msg^}\n"
        continue
    fi

    if echo "$change" | grep -q "revert.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        reverts+="${msg^}\n"
        continue
    fi
done

# Assemble the changelog entry
changelog_entry="\n## $new_tag - $(date +%Y-%m-%d)\n"
if [ -n "${features-}" ]; then
    changelog_entry+="### Features\n\n"

    echo "$features" | while IFS= read -r entry; do
        if echo "$entry" | grep '^!!'; then
            changelog_entry+=" - [!!] $entry\n"
        elif echo "$entry" | grep '^!'; then
            changelog_entry+=" - [!] $entry\n"
        else
            changelog_entry+=" - $entry\n"
        fi
    done

    changelog_entry+="\n\n"
fi

if [ -n "${fixes-}" ]; then
    changelog_entry+="### Fixes\n\n"

    echo "$fixes" | while IFS= read -r entry; do
        if echo "$entry" | grep '^!!'; then
            changelog_entry+=" - [!!] $entry\n"
        elif echo "$entry" | grep '^!'; then
            changelog_entry+=" - [!] $entry\n"
        else
            changelog_entry+=" - $entry\n"
        fi
    done

    changelog_entry+="\n\n"
fi

if [ -n "${chores-}" ]; then
    changelog_entry+="### Chores\n\n"

    echo "$chores" | while IFS= read -r entry; do
        if echo "$entry" | grep '^!!'; then
            changelog_entry+=" - [!!] $entry\n"
        elif echo "$entry" | grep '^!'; then
            changelog_entry+=" - [!] $entry\n"
        else
            changelog_entry+=" - $entry\n"
        fi
    done

    changelog_entry+="\n\n"
fi

if [ -n "${docs-}" ]; then
    changelog_entry+="### Docs\n\n"

    echo "$docs" | while IFS= read -r entry; do
        if echo "$entry" | grep '^!!'; then
            changelog_entry+=" - [!!] $entry\n"
        elif echo "$entry" | grep '^!'; then
            changelog_entry+=" - [!] $entry\n"
        else
            changelog_entry+=" - $entry\n"
        fi
    done

    changelog_entry+="\n\n"
fi

if [ -n "${ci-}" ]; then
    changelog_entry+="### CI\n\n"

    echo "$ci" | while IFS= read -r entry; do
        if echo "$entry" | grep '^!!'; then
            changelog_entry+=" - [!!] $entry\n"
        elif echo "$entry" | grep '^!'; then
            changelog_entry+=" - [!] $entry\n"
        else
            changelog_entry+=" - $entry\n"
        fi
    done

    changelog_entry+="\n\n"
fi

if [ -n "${reverts-}" ]; then
    changelog_entry+="### Reverts\n\n"

    echo "$reverts" | while IFS= read -r entry; do
        if echo "$entry" | grep '^!!'; then
            changelog_entry+=" - [!!] $entry\n"
        elif echo "$entry" | grep '^!'; then
            changelog_entry+=" - [!] $entry\n"
        else
            changelog_entry+=" - $entry\n"
        fi
    done

    changelog_entry+="\n\n"
fi

# Write out the new changelog
first_entry_lineno=$(grep '^## ' -n CHANGES.md | cut -d: -f1)
first_entry_lineno=$((first_entry_lineno - 1))

header_temp=$(mktemp)
head -$first_entry_lineno CHANGES.md > "$header_temp"

old_temp=$(mktemp)
tail +$first_entry_lineno CHANGES.md > "$old_temp"

new_temp=$(mktemp)
printf %s "$changelog_entry" > "$new_temp"

cat "$header_temp" "$new_temp" "$body_temp" > CHANGES.md

git add Cargo.{toml,lock} CHANGES.md -m "auto(bump): $new_tag" -m "$changelog_entry"
