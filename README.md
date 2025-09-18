# Vagrant

## Version Aggregator and Tracker
Vagrant aggregates and tracks the release, unstable, and commit versions for a
collection of packages. Though designed with Linux From Scratch maintenance in
mind, Vagrant is generic enough to be applicable to most tasks requiring version
fetching.

## Usage
Vagrant is intended to be used to quickly check for package updates, supporting
both bulk requests and single requests.

### Single
For instance, to check the release version of only ffmpeg:
```sh
curl -fsSL https://raw.githubusercontent.com/Toxikuu/vagrant/refs/heads/master/p/ffmpeg/release
```

To check the unstable version of bc:
```sh
curl -fsSL https://raw.githubusercontent.com/Toxikuu/vagrant/refs/heads/master/p/bc/unstable
```

To check the commit version of btop:
```sh
curl -fsSL https://raw.githubusercontent.com/Toxikuu/vagrant/refs/heads/master/p/btop/commit
```

### Bulk
For bulk requests, a CSV file is made available:
```
curl -fsSL https://raw.githubusercontent.com/Toxikuu/vagrant/refs/heads/master/p/ALL
```

It's recommended you cache this file and read multiple package versions from it.
It should be pretty trivial to parse with `cut -d, -f1-4`.

## Roadmap

### Automation
I intend for Vagrant to be a mostly automatic system that runs on its own at
specified intervals. It should only need intervention for the addition of new
packages or the amelioration of upstream messing with their versioning scheme.

For the moment, I intend to use GitHub workflows for this automation, but plan
to self-host Vagrant in the future. (This should also make for a less ugly URL.)

### Collaboration
I'd love to work alongside anyone building a package repository, and I want this
to be a community-driven project. I'm open to tracking new packages. For more
information, read [the contributing guidelines](./CONTRIBUTING.md).
