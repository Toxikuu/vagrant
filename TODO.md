# TODO

- [x] Add upstream as a field for release, unstable, and commit. If defined, the
  regular upstream field should fill out release, unstable, and commit's
  upstreams. This is just a clearer way to override upstreams.
- [x] Add [-p|--pretend] flag that doesn't write any versions, but merely
  displays them.
- [x] Pivot from a deterministic delay system to a chance-based one.
- [x] Add an expected field for release, unstable, and commit. If defined,
  vagrant will compile the regex denoted therein and match against the fetched
  version. A mismatch will indicates a failed fetch. This should help enforce
  correct version detection.
- [ ] Write a shell script to find package versions that have not been updated
  in a while. These might then be manually confirmed.
- [ ] Implement parallelization
- [x] Add GitHub issue templates
- [ ] Mirror to vagrant.tox.wtf
- [x] Add changelog
- [x] Add caching for `gr`
- [x] Add version channels
    - [x] Overhaul the codebase and API
- [x] Test release.sh
- [x] Fix chance skipping
- [ ] Fix changelog formatting for breaking changes
