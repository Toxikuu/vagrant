#!/usr/bin/env bash

set -eu
argv0="$0"

nl="
"

die() {
    printf "%s: %s\n" "$argv0" "$1"
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
if echo "$changes" | grep -q '^!!'; then
    new_tag_major=$((old_tag_major + 1))
    new_tag_minor=0
    new_tag_patch=0
elif echo "$changes" | grep -q '^!'; then
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
sed -i "s|version = \"$old_tag\"|version = \"$new_tag\"|" Cargo.toml
new_sum=$(sha256sum Cargo.toml)

if [[ "$old_sum" == "$new_sum" ]]; then
    die "Failed to update version in Cargo.toml"
fi

old_sum=$(sha256sum Cargo.lock)
cargo build --release
new_sum=$(sha256sum Cargo.lock)

if [[ "$old_sum" == "$new_sum" ]]; then
    die "Cargo.lock unchanged after version bump"
fi

# Parse changes
# NOTE: Automatic commits are excluded
features=""
fixes=""
chores=""
ci=""
docs=""
revert=""
while IFS= read -r change; do
    if echo "$change" | grep -q "feat.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        features+="${msg^}$nl"
        continue
    fi

    if echo "$change" | grep -q "fix.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        fixes+="${msg^}$nl"
        echo "${fixes:?}"
        continue
    fi

    if echo "$change" | grep -q "chore.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        chores+="${msg^}$nl"
        continue
    fi

    if echo "$change" | grep -q "ci.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        cis+="${msg^}$nl"
        continue
    fi

    if echo "$change" | grep -q "docs.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        docs+="${msg^}$nl"
        continue
    fi

    if echo "$change" | grep -q "revert.*:"; then
        msg="$(echo "$change" | cut -d: -f2-)"
        reverts+="${msg^}$nl"
        continue
    fi
done <<< "$changes"

# Assemble the changelog entry
changelog_entry="$nl## $new_tag - $(date +%Y-%m-%d)$nl$nl"

if [ -n "${features-}" ]; then
    changelog_entry+="### Features$nl$nl"

    while IFS= read -r entry; do
        if [ -z "$entry" ]; then continue; fi

        if echo "$entry" | grep -q '^!!'; then
            entry="${entry/^!!/}"
            changelog_entry+=" - **[!!]** ${entry^}$nl"
        elif echo "$entry" | grep -q '^!'; then
            entry="${entry/^!/}"
            changelog_entry+=" - [!] ${entry^}$nl"
        else
            changelog_entry+=" - ${entry^}$nl"
        fi
    done <<< "$features"

    changelog_entry+="$nl"
fi

if [ -n "${fixes-}" ]; then
    changelog_entry+="### Fixes$nl$nl"

    while IFS= read -r entry; do
        if [ -z "$entry" ]; then continue; fi

        if echo "$entry" | grep -q '^!!'; then
            entry="${entry/^!!/}"
            changelog_entry+=" - **[!!]** ${entry^}$nl"
        elif echo "$entry" | grep -q '^!'; then
            entry="${entry/^!/}"
            changelog_entry+=" - [!] ${entry^}$nl"
        else
            changelog_entry+=" - ${entry^}$nl"
        fi
    done <<< "$fixes"

    changelog_entry+="$nl"
fi

if [ -n "${chores-}" ]; then
    changelog_entry+="### Chores$nl$nl"

    while IFS= read -r entry; do
        if [ -z "$entry" ]; then continue; fi

        if echo "$entry" | grep -q '^!!'; then
            entry="${entry/^!!/}"
            changelog_entry+=" - **[!!]** ${entry^}$nl"
        elif echo "$entry" | grep -q '^!'; then
            entry="${entry/^!/}"
            changelog_entry+=" - [!] ${entry^}$nl"
        else
            changelog_entry+=" - ${entry^}$nl"
        fi
    done <<< "$chores"

    changelog_entry+="$nl"
fi

if [ -n "${docs-}" ]; then
    changelog_entry+="### Docs$nl$nl"

    while IFS= read -r entry; do
        if [ -z "$entry" ]; then continue; fi

        if echo "$entry" | grep -q '^!!'; then
            entry="${entry/^!!/}"
            changelog_entry+=" - **[!!]** ${entry^}$nl"
        elif echo "$entry" | grep -q '^!'; then
            entry="${entry/^!/}"
            changelog_entry+=" - [!] ${entry^}$nl"
        else
            changelog_entry+=" - ${entry^}$nl"
        fi
    done <<< "$docs"

    changelog_entry+="$nl"
fi

if [ -n "${ci-}" ]; then
    changelog_entry+="### CI$nl$nl"

    while IFS= read -r entry; do
        if [ -z "$entry" ]; then continue; fi

        if echo "$entry" | grep -q '^!!'; then
            entry="${entry/^!!/}"
            changelog_entry+=" - **[!!]** ${entry^}$nl"
        elif echo "$entry" | grep -q '^!'; then
            entry="${entry/^!/}"
            changelog_entry+=" - [!] ${entry^}$nl"
        else
            changelog_entry+=" - ${entry^}$nl"
        fi
    done <<< "$ci"

    changelog_entry+="$nl"
fi

if [ -n "${reverts-}" ]; then
    changelog_entry+="### Reverts$nl$nl"

    while IFS= read -r entry; do
        if [ -z "$entry" ]; then continue; fi

        if echo "$entry" | grep -q '^!!'; then
            entry="${entry/^!!/}"
            changelog_entry+=" - **[!!]** ${entry^}$nl"
        elif echo "$entry" | grep -q '^!'; then
            entry="${entry/^!/}"
            changelog_entry+=" - [!] ${entry^}$nl"
        else
            changelog_entry+=" - ${entry^}$nl"
        fi
    done <<< "$reverts"

    changelog_entry+="$nl"
fi

# Write out the new changelog
first_entry_lineno=$(grep '^## ' -n CHANGES.md | head -n1 | cut -d: -f1)
first_entry_lineno=$((first_entry_lineno - 1))

header_temp=$(mktemp)
head -$((first_entry_lineno - 1)) CHANGES.md > "$header_temp"

old_temp=$(mktemp)
tail +$first_entry_lineno CHANGES.md > "$old_temp"

new_temp=$(mktemp)
printf %s "$changelog_entry" > "$new_temp"

cat "$header_temp" "$new_temp" "$old_temp" > CHANGES.md
rm  "$header_temp" "$new_temp" "$old_temp"

git add Cargo.{toml,lock} CHANGES.md
git commit -m "auto(bump): $new_tag" -m "$changelog_entry"

git tag "$new_tag"
git push origin "$new_tag"
