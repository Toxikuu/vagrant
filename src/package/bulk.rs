// package/bulk.rs

use crate::package::PackageVersions;

use super::{Package, VersionChannel};
use std::{env, fs};
use std::path::Path;
use color_eyre::eyre::{Context, ContextCompat, Error};
use color_eyre::Result;
use indexmap::IndexMap;
use tracing::{debug, error};
use rayon::prelude::*;

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
    let threads = env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| num_cpus::get() * 2);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("Failed to create thread pool");

    let res = pool.install(|| {
        packages.par_iter()
            .map(|package| {
                let mut skipped = 0;
                let mut failed = 0;

                let versions = match package.fetch() {
                    Ok(v) => v,
                    Err(e) if e.to_string().contains("Tails!") => {
                        skipped = 1;
                        debug!("Skipped fetching versions for package '{}'", package.name);
                        package.read_versions()
                            .wrap_err_with(|| format!("Failed to read old versions for skipped package '{}'", package.name))?
                    },
                    Err(e) => {
                        failed = 1;
                        error!("Failed to fetch versions for {}: {e}", package.name);
                        package.read_versions()
                            .wrap_err_with(|| format!("Failed to read old versions for failed package '{}'", package.name))?
                    }
                };

                Ok::<_, Error>((package.clone(), versions, skipped, failed))
            })
        .collect::<Result<Vec<_>, _>>()
    }).wrap_err("Failed to bulk fetch versions")?;

    let mut map = IndexMap::new();
    let mut skipped_count = 0;
    let mut failed_count = 0;

    for (pkg, ver, skipped, failed) in res {
        map.insert(pkg, ver);
        skipped_count += skipped;
        failed_count += failed;
    }

    fs::write("total", packages.len().to_string())?;
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
