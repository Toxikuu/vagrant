# TODO

- [x] Add upstream as a field for release, unstable, and commit. If defined, the
  regular upstream field should fill out release, unstable, and commit's
  upstreams. This is just a clearer way to override upstreams.
- [ ] Add [-p|--pretend] flag that doesn't write any versions, but merely
  displays them.
- [x] Pivot from a deterministic delay system to a chance-based one.
- [ ] Add an expected field for release, unstable, and commit. If defined,
  vagrant will compile the regex denoted therein and match against the fetched
  version. A mismatch will indicates a failed fetch. This should help enforce
  correct version detection.
- [ ] Write a shell script to find package versions that have not been updated
  in a while. These might then be manually confirmed.
- [ ] Add normalization for pre-release versions (e.g. 'rc' should always be
  prefixed by a '-')
