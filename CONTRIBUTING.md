# Contributing

## Quick Start
You're probably interested in adding a package. To do so, fork and clone this
repo, then source the maintainer environment with `. sh/m`. Once you've done
this, you can add a package with `va mypackage`. Take a look at the functions
defined in `sh/lib.env`, and peruse existing packages.

## Commits
Vagrant follows a variant of conventional commits.

Some general rules:
- Keep commit subject length to 72 characters or fewer.
- Commit subjects should be lowercase and limited to ASCII. Descriptions should
  also keep to ASCII, but may be capitalized as desired.
- Breaking changes (i.e. changes that might impact the version fetching of other
  packages) should start with '!' in the subject line.

To add a package, the commit message would be:
> feat(p): add mypackage

To fix the release fetch for a package, the commit message would be:
> fix(p): fix release fetch for mypackage

To make a breaking tweak to the vtrim function in the shell library, addressing
an issue, and signing off:
> !feat(lib): adjust vtrim behavior
>
> Instead of only trimming a leading 'v', vtrim now trims any leading alphabetic
> character if it's immediately followed by a number.
>
> Resolves: #488
> References: #122, #556

### Commit Types
- auto: automatic commits made by vagrant
- ci: changes to the ci files and scripts
- chore: changes to auxiliary files
- docs: changes to any documentation
- feat: a new feature or package
- fix: a bugfix
- revert: revert something

### Scopes
Scopes include but are not limited to:
- (p): packages
- (lib): the shell library

<!-- TODO: Add some more information -->
