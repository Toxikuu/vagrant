// utils/ver.rs

use crate::package::Package;
use crate::utils::str::basename;

#[derive(Debug, Default)]
pub struct Version {
    pub raw: String,
    pub fmt: String,
}

impl Version {
    pub const fn new(raw: String) -> Self {
        Self {
            raw,
            fmt: String::new(),
        }
    }

    pub fn trim(&mut self, package: &Package) {
        let ver = self
            .raw
            .lines()
            .filter(|l| !l.is_empty())
            .next_back()
            .map_or_else(|| unreachable!("No output"), str::to_lowercase);

        let name = basename(&package.name);

        let ver = ver.trim_start_matches('v');
        let ver = ver.trim_start_matches(name);
        let ver = ver.trim_start_matches('-');
        let ver = ver.trim_start_matches('_');

        self.fmt = ver.trim().to_string();
    }
}
