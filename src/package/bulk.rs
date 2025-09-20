// package/bulk.rs

use super::{Package, Versions};
use std::fs;
use std::path::Path;
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Result;
use indexmap::IndexMap;
use tracing::error;

pub fn find_all() -> Result<Vec<Package>> {
    let search_path = Path::new("p");
    let mut packages = vec![];

    for entry in search_path.read_dir()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let file_name = path.file_name()
                .wrap_err_with(|| format!("Invalid filename in {path:?}"))?
                .to_string_lossy()
                .to_string();

            if let Ok(package) = Package::from_name(file_name) {
                packages.push(package);
            }
        }
    }

    Ok(packages)
}

pub fn fetch_all(packages: &[Package]) -> Result<IndexMap<Package, Versions>> {
    let mut map = IndexMap::new();
    let mut failed_count = 0;
    let mut skipped_count = 0;

    for package in packages {
        let versions = match package.fetch() {
            Ok(v) => v,
            Err(e) if e.to_string().contains("Tails!") => {
                skipped_count += 1;
                package.retrieve_versions()
                    .wrap_err_with(|| format!("Failed to retrieve versions for skipped {}", package.name))?
            },
            Err(e) => {
                failed_count += 1;
                error!("Failed to fetch versions for {}: {e}", package.name);
                package.retrieve_versions()
                    .wrap_err_with(|| format!("Failed to retrieve versions for failed {}", package.name))?
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

pub fn write_all(map: IndexMap<Package, Versions>) -> Result<()> {
    let mut buf = String::new();

    for (k, v) in map.iter() {
        k.write(v.clone())?;

        let formatted = format!("{},{},{},{}\n", k.name, v.0, v.1, v.2);
        buf.push_str(&formatted);
    }

    let path = Path::new("p").join("ALL");
    fs::write(path, buf)?;

    Ok(())
}
