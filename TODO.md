# TODO

- [x] Add upstream as a field for release, unstable, and commit. If defined, the
  regular upstream field should fill out release, unstable, and commit's
  upstreams. This is just a clearer way to override upstreams.
- [ ] Pivot from a deterministic delay system to a chance-based one.
- [ ] Add an expected field for release, unstable, and commit. If defined,
  vagrant will compile the regex denoted therein and match against the fetched
  version. This should help enforce correct version detection.
- [ ] Write a shell script to find package versions that have not been updated
  in a while. These might then be manually confirmed.
