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
- [ ] Add GitHub issue templates
- [ ] Mirror the results to vagrant.tox.wtf
- [ ] Add version channels
    - Syntax: release-CHANNEL, e.g. release-3 for gtk3, release-sdk for glslang
    - Would require compatibility tweaks to commit.sh and the codebase
    - Would require changing the p/ALL CSV (might be easiest to just use JSON?)
