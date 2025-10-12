// utils/ver.rs

use crate::package::Package;

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
        let ver = match self.raw.lines().filter(|l| !l.is_empty()).next_back() {
            | None => unreachable!("No output"),
            | Some(v) => v.to_lowercase(),
        };

        let ver = ver.trim_start_matches('v');
        let ver = ver.trim_start_matches(&package.name);
        let ver = ver.trim_start_matches('-');
        let ver = ver.trim_start_matches('_');

        self.fmt = ver.trim().to_string();
    }
}
