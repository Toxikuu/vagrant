// package/bulk.rs

use crate::package::PackageVersions;

use super::{Package, VersionChannel};
use std::fs;
use std::path::Path;
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Result;
use indexmap::IndexMap;
use tracing::{debug, error};

pub fn find_all() -> Result<Vec<Package>> {
    let search_path = Path::new("p");
    let mut packages = vec![];

    for entry in search_path.read_dir()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let file_name = path.file_name()
                .wrap_err_with(|| format!("Invalid filename in {}", path.display()))?
                .to_string_lossy()
                .to_string();

            packages.push(Package::from_name(file_name.clone())
                .wrap_err_with(|| format!("Failed to form package '{file_name}'"))?);
        }
    }

    packages.sort();
    Ok(packages)
}

pub fn fetch_all(packages: &[Package]) -> Result<IndexMap<Package, Vec<VersionChannel>>> {
    let mut map = IndexMap::new();
    let mut failed_count = 0;
    let mut skipped_count = 0;

    for package in packages {
        let versions = match package.fetch() {
            Ok(v) => v,
            Err(e) if e.to_string().contains("Tails!") => {
                skipped_count += 1;
                debug!("Skipped fetching versions for package '{}'", package.name);
                package.read_versions()
                    .wrap_err_with(|| format!("Failed to read old versions for skipped package '{}'", package.name))?
            },
            Err(e) => {
                failed_count += 1;
                error!("Failed to fetch versions for {}: {e}", package.name);
                package.read_versions()
                    .wrap_err_with(|| format!("Failed to read old versions for failed package '{}'", package.name))?
            }
        };

        map.insert(package.clone(), versions);
    }

    fs::write("failed", failed_count.to_string())?;
    fs::write("skipped", skipped_count.to_string())?;
    fs::write("checked", map.len().to_string())?;
    map.sort_keys();

    Ok(map)
}

pub fn write_all(map: &IndexMap<Package, Vec<VersionChannel>>) -> Result<()> {
    let mut all_vec = vec![];

    for (k, v) in map {
        k.write_versions(v.clone())?;
        all_vec.push(PackageVersions { package: k.name.clone(), versions: v.clone() });
    }

    let path = Path::new("p");

    let alljson = serde_json::to_string_pretty(&all_vec)?;
    fs::write(path.join("ALL.json"), alljson)?;

    let mut alltxt = String::new();
    for p in all_vec {
        for c in p.versions {
            alltxt = format!("{alltxt}{}\t{}\t{}\n", p.package, c.channel, c.version);
        }
    }
    fs::write(path.join("ALL.txt"), alltxt)?;

    Ok(())
}
