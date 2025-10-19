// package/mod.rs

pub mod bulk;

use std::fmt::Debug;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use rand::random_range;
use regex::Regex;
use serde::{Deserialize, Serialize};
use color_eyre::Result;
use color_eyre::eyre::bail;
use tracing::{debug, info, error};

use crate::args::ARGS;
use crate::utils::cmd::cmd;
use crate::utils::float::defloat;
use crate::utils::shortform::{get_shortform, get_longform};
use crate::utils::ver::Version;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Package {
    pub name: String,
    pub config: PackageConfig,
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct PackageConfig {
    pub upstream: String,
    pub chance: f64,
    pub channels: Vec<PackageChannel>,
}

/// Struct to be used when serializing into p/ALL
#[derive(Debug, Serialize, Clone)]
pub struct PackageVersions {
    pub package: String,
    pub versions: Vec<VersionChannel>,
}

#[derive(Hash, PartialEq, Eq, Debug, Deserialize, Clone)]
#[serde(default)]
pub struct PackageChannel {
    pub name: String,
    pub enabled: bool,
    pub upstream: Option<String>,
    pub fetch: String,
    pub expected: Option<String>,
    // TODO: Consider adding per-channel chances
}

impl Default for PackageChannel {
    fn default() -> Self {
        Self {
            name: String::new(),
            enabled: true,
            upstream: None,
            fetch: String::new(),
            expected: None,
        }
    }
}

impl PackageChannel {
    pub fn fetch(&self, package: &Package) -> Result<String> {
        let name = format!("name={}", package.name);
        let upstream = format!("upstream={}", get_longform(
            self.upstream.as_ref().unwrap_or(&package.config.upstream)
        ));

        let shortform = format!("shortform={}", get_shortform(&upstream));
        let fetch = format!(". sh/lib.env && {}", self.fetch);

        let command = [
            "env",
            &name,
            &upstream,
            &shortform,
            "bash",
            "-c",
            &fetch,
        ];

        let ver = match cmd(&command) {
            | Err(e) => bail!("Failed to fetch version: {e}"),
            | Ok(v) => v,
        };

        let mut version = Version::new(ver);
        version.trim(package);
        let v = version.fmt;

        if let Some(re) = &self.expected {
            let re = match Regex::from_str(re) {
                Ok(re) => re,
                Err(e) => {
                    error!("Invalid expected regex '{re}': {e}");
                    bail!("Invalid expected regex");
                }
            };

            if !re.is_match(&v) {
                error!("Version '{v}' does not match expected '{re}'");
                bail!("Version does not match expected");
            }
        }

        Ok(v)
    }
}

impl Hash for PackageConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.upstream.hash(state);
        defloat(self.chance).hash(state);
        self.channels.hash(state);
    }
}

impl PartialEq for PackageConfig {
    fn eq(&self, other: &Self) -> bool {
        self.upstream == other.upstream &&
        (self.chance - other.chance).abs() < 0.01 &&
        self.channels == other.channels
    }
}

impl Eq for PackageConfig {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionChannel {
    pub channel: String,
    pub version: String,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            upstream: String::new(),
            chance: 1.,
            channels: {
                vec![
                    PackageChannel {
                        name: "release".into(),
                        enabled: true,
                        upstream: None,
                        // the default value of fetch is filled out later as it depends on upstream
                        fetch: String::new(),
                        expected: Some(String::from(r"^[0-9]+(\.[0-9]+)*$")),
                    },
                    PackageChannel {
                        name: "unstable".into(),
                        enabled: true,
                        upstream: None,
                        // the default value of fetch is filled out later as it depends on upstream
                        fetch: String::new(),
                        expected: Some(String::from(r"^[0-9]+(\.[0-9]+)*-?(rc|alpha|beta|a|b|pre|dev)?[0-9]*$")),
                    },
                    PackageChannel {
                        name: "commit".into(),
                        enabled: true,
                        upstream: None,
                        // the default value of fetch is filled out later as it depends on upstream
                        fetch: String::new(),
                        expected: Some(String::from(r"^[0-9a-f]{40}$")),
                    },
                ]
            },
        }
    }
}

pub enum UpstreamType {
    Arch,
    Curl,
    Empty,
    Git,
    GitHub,
}

impl UpstreamType {
    fn from_str(str: &str) -> Self {
        match str {
            // match arch
            s if s.contains("archlinux.org") => Self::Arch,

            // match distfile pages
            s if s.contains("C=M") && s.contains("O=D") => Self::Curl,

            // match github links or shortform
            s if s.starts_with("https://github.com/") || s.split('/').count() == 2 => Self::GitHub,

            "" => Self::Empty,

            // assume all else is git
            _ => Self::Git,
        }
    }

    fn default_fetch_release(&self) -> String {
        match self {
            Self::Arch   => String::from("archver"),
            Self::Curl   => String::from("ca | vsort"),
            Self::Empty  => String::with_capacity(0),
            Self::Git    => String::from("gr | vfs | tolower | vtrim | fsl | vsort"),
            Self::GitHub => String::from("ghr | tolower | vtrim | fsl"),
        }
    }

    fn default_fetch_unstable(&self) -> String {
        match self {
            Self::Arch               => String::from("archver"),
            Self::Curl               => String::from("ca | vsort"),
            Self::Empty              => String::with_capacity(0),
            Self::Git | Self::GitHub => String::from("gr | tolower | vtrim | fsl | vsort"),
        }
    }

    fn default_fetch_commit(&self) -> String {
        match self {
            Self::Arch | Self::Curl | Self::Empty => String::with_capacity(0),
            Self::Git  | Self::GitHub             => String::from("githead"),
        }
    }
}

impl Package {
    pub fn from_name(name: String) -> Result<Self> {
        let config_path = Path::new("p").join(&name).join("config");

        let raw = std::fs::read_to_string(config_path)?;
        let config: PackageConfig = toml::from_str(&raw)?;

        let mut package = Self { name, config };
        package.set_defaults();

        Ok(package)
    }

    pub fn get_channel(&self, name: &str) -> Option<&PackageChannel> {
        self.config.channels.iter().find(|c| c.name == name)
    }

    // pub fn get_channel_mut(&mut self, name: &str) -> Option<&mut PackageChannel> {
    //     self.config.channels.iter_mut().find(|c| c.name == name)
    // }

    pub fn set_defaults(&mut self) {
        if let Some(c) = self.config.channels.iter_mut().find(|c| c.name == "release")
        && c.enabled {
            if c.fetch.is_empty() {
                let upstream = &c.upstream.as_ref().unwrap_or(&self.config.upstream);
                let upstream_type = UpstreamType::from_str(upstream);
                c.fetch = upstream_type.default_fetch_release();
            }

            if c.expected.is_none() {
                c.expected = Some(r"^[0-9]+(\.[0-9]+)*$".into());
            }
        }

        if let Some(c) = self.config.channels.iter_mut().find(|c| c.name == "unstable")
        && c.enabled {
            if c.fetch.is_empty() {
                let upstream = &c.upstream.as_ref().unwrap_or(&self.config.upstream);
                let upstream_type = UpstreamType::from_str(upstream);
                c.fetch = upstream_type.default_fetch_unstable();
            }

            if c.expected.is_none() {
                c.expected = Some(r"^[0-9]+(\.[0-9]+)*-?(rc|alpha|beta|a|b|pre|dev)?[0-9]*$".into());
            }
        }

        if let Some(c) = self.config.channels.iter_mut().find(|c| c.name == "commit")
        && c.enabled {
            if c.fetch.is_empty() {
                let upstream = &c.upstream.as_ref().unwrap_or(&self.config.upstream);
                let upstream_type = UpstreamType::from_str(upstream);

                if matches!(upstream_type, UpstreamType::Curl | UpstreamType::Arch | UpstreamType::Empty) {
                    c.enabled = false;
                } else {
                    c.fetch = upstream_type.default_fetch_commit();
                }
            }

            if c.expected.is_none() {
                c.expected = Some(r"^[0-9a-f]{40}$".into());
            }
        }
    }

    pub fn has_fallback_versions(&self) -> bool {
        let path = Path::new("p").join(&self.name).join("versions.json");
        if !path.exists() { return false }

        let Ok(version_channels_str) = fs::read_to_string(path) else {return false};
        let Ok(version_channels) = serde_json::from_str::<Vec<VersionChannel>>(&version_channels_str) else { return false };

        for channel in &version_channels {
            if self.get_channel(&channel.channel).is_none() { return false }
        }

        true
    }

    pub fn fetch(&self) -> Result<Vec<VersionChannel>> {
        // if fallback versions don't exist, or --guarantee is passed, guarantee a fetch
        let should_guarantee = ARGS.guarantee || !self.has_fallback_versions();

        if self.config.chance < 1.0
        && !should_guarantee
        && random_range(0.0..=1.0) > self.config.chance
        { bail!("Tails!") }

        let mut version_channels = vec![];
        for channel in &self.config.channels {
            if channel.enabled {
                version_channels.push(
                    VersionChannel {
                        channel: channel.name.clone(),
                        version: channel.fetch(self)?,
                    }
                );
            }
        }

        info!("Fetched versions for {}: {version_channels:#?}", self.name);
        debug!("Versions as JSON: {}", serde_json::to_string_pretty(&version_channels)?);

        Ok(version_channels)
    }

    pub fn get_package_path(&self) -> PathBuf {
        Path::new("p").join(&self.name)
    }

    /// Write version data for all version channels for all APIs
    pub fn write_versions(&self, version_channels: Vec<VersionChannel>) -> Result<()> {
        let path = self.get_package_path();
        fs::write(path.join("versions.json"), serde_json::to_string_pretty(&version_channels)?)?;

        let channels_dir = path.join("channels");

        if !channels_dir.exists() {
            fs::create_dir(&channels_dir)?;
        }

        let mut versionstxt = String::new();
        for channel in version_channels {
            fs::write(channels_dir.join(&channel.channel), &channel.version)?;
            versionstxt = format!("{versionstxt}{}\t{}\n", channel.channel, channel.version);
        }

        fs::write(path.join("versions.txt"), versionstxt)?;
        Ok(())
    }

    /// Write version data for all version channels (reads from JSON API)
    pub fn read_versions(&self) -> Result<Vec<VersionChannel>> {
        let path = self.get_package_path().join("versions.json");
        let json_str = fs::read_to_string(path)?;

        let version_channels = serde_json::from_str(&json_str)?;
        Ok(version_channels)
    }
}
