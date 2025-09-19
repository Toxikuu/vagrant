// package/mod.rs

pub mod bulk;

use std::fmt::Debug;
use std::fs;
use std::path::Path;
use serde::Deserialize;
use color_eyre::Result;
use color_eyre::eyre::bail;
use tracing::info;

use crate::utils::cmd::cmd;
use crate::utils::shortform::{get_shortform, get_longform};
use crate::utils::ver::Version;

pub type Versions = (String, String, String);

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

#[derive(Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
#[serde(default)]
pub struct PackageConfig {
    pub upstream: String,
    pub delay: u32,
    pub release: PackageRelease,
    pub unstable: PackageUnstable,
    pub commit: PackageCommit,
}

#[derive(Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
#[serde(default)]
pub struct PackageRelease {
    pub enabled: bool,
    pub upstream: Option<String>,
    pub fetch: String,
}

// The default value for fetch is filled out later, as it depends on the value of upstream
impl Default for PackageRelease {
    fn default() -> Self {
        Self {
            enabled: true,
            upstream: None,
            fetch: String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
#[serde(default)]
pub struct PackageUnstable {
    pub enabled: bool,
    pub upstream: Option<String>,
    pub fetch: String,
}

// The default value for fetch is filled out later, as it depends on the value of upstream
impl Default for PackageUnstable {
    fn default() -> Self {
        Self {
            enabled: true,
            upstream: None,
            fetch: String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
#[serde(default)]
pub struct PackageCommit {
    pub enabled: bool,
    pub upstream: Option<String>,
    pub fetch: String,
}

// The default value for fetch is filled out later, as it depends on the value of upstream
impl Default for PackageCommit {
    fn default() -> Self {
        Self {
            enabled: true,
            upstream: None,
            fetch: String::new(),
        }
    }
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            upstream: String::new(),
            delay: 1,
            release: PackageRelease::default(),
            unstable: PackageUnstable::default(),
            commit: PackageCommit::default(),
        }
    }
}

pub enum UpstreamType {
    Arch,
    Curl,
    GitHub,
    // GitLab,
    Git,
    Empty,
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

            s if s.is_empty() => Self::Empty,
            // match gitlab links
            // s if s.contains("://gitlab.com/") => Self::GitLab,

            // assume all else is git
            _ => Self::Git,
        }
    }

    fn default_fetch_release(&self) -> String {
        match self {
            Self::Arch => String::from("archver"),
            Self::Curl => String::from("ca | vsort"),
            Self::GitHub => String::from("ghr | tolower | vtrim | fsl"),
            Self::Git => String::from("gr | vfs | tolower | vtrim | fsl | vsort"),
            Self::Empty => String::with_capacity(0),
        }
    }

    fn default_fetch_unstable(&self) -> String {
        match self {
            Self::Arch => String::from("archver"),
            Self::Curl => String::from("ca | vsort"),
            Self::GitHub => String::from("gr | tolower | vtrim | fsl | vsort"),
            Self::Git => String::from("gr | tolower | vtrim | fsl | vsort"),
            Self::Empty => String::with_capacity(0),
        }
    }

    fn default_fetch_commit(&self) -> String {
        match self {
            Self::Arch => String::from(""), // WARN: Arch-type upstreams don't have commits
            Self::Curl => String::from(""), // WARN: Curl-type upstreams don't have commits
            Self::GitHub => String::from("ghc"),
            Self::Git => String::from("githead"),
            Self::Empty => String::with_capacity(0),
        }
    }
}

impl Package {
    pub fn from_name(name: String) -> Result<Self> {
        let config_path = Path::new("p").join(&name).join("config");

        let raw = std::fs::read_to_string(config_path)?;
        let config: PackageConfig = toml::from_str(&raw)?;

        let mut package = Self { name, config };
        package.set_default_fetches();

        Ok(package)
    }

    fn retrieve_versions(&self) -> Result<Versions> {
        let path = Path::new("p").join(&self.name);

        Ok((
            fs::read_to_string(path.join("release"))?,
            fs::read_to_string(path.join("unstable"))?,
            fs::read_to_string(path.join("commit"))?,
        ))
    }

    pub fn set_default_fetches(&mut self) {
        if self.config.release.enabled && self.config.release.fetch.is_empty() {
            let upstream = &self.config.release.upstream.as_ref().unwrap_or(&self.config.upstream);
            let upstream_type = UpstreamType::from_str(upstream);
            self.config.release.fetch = upstream_type.default_fetch_release();
        }

        if self.config.unstable.enabled && self.config.unstable.fetch.is_empty() {
            let upstream = &self.config.unstable.upstream.as_ref().unwrap_or(&self.config.upstream);
            let upstream_type = UpstreamType::from_str(upstream);
            self.config.unstable.fetch = upstream_type.default_fetch_unstable();
        }

        if self.config.commit.enabled && self.config.commit.fetch.is_empty() {
            let upstream = &self.config.commit.upstream.as_ref().unwrap_or(&self.config.upstream);
            let upstream_type = UpstreamType::from_str(upstream);

            if matches!(upstream_type, UpstreamType::Curl | UpstreamType::Arch | UpstreamType::Empty) {
                self.config.commit.enabled = false
            } else {
                self.config.commit.fetch = upstream_type.default_fetch_commit();
            }
        }
    }

    pub fn fetch(&self) -> Result<Versions> {
        if self.get_current_delay()? > 1 {
            bail!("Delayed")
        }

        let release = if self.config.release.enabled { self.fetch_release()? } else { String::with_capacity(0) };
        let unstable = if self.config.unstable.enabled { self.fetch_unstable()? } else { String::with_capacity(0) };
        let commit = if self.config.commit.enabled { self.fetch_commit()? } else { String::with_capacity(0) };

        info!("Fetched versions for {}: ({release}, {unstable}, {commit})", self.name);

        Ok((release, unstable, commit))
    }

    pub fn write(&self, versions: Versions) -> Result<()> {
        let path = Path::new("p").join(&self.name);
        fs::write(path.join("release"), versions.0)?;
        fs::write(path.join("unstable"), versions.1)?;
        fs::write(path.join("commit"), versions.2)?;

        Ok(())
    }

    pub fn get_current_delay(&self) -> Result<u32> {
        let path = Path::new("p").join(&self.name).join("delay");
        let config_delay = self.config.delay;

        if !path.exists() {
            return Ok(config_delay)
        }

        let read_delay = fs::read_to_string(&path)?.trim().parse::<u32>().unwrap_or(config_delay);

        // Accounts for the case where the configured delay is changed. The counter is set to the
        // newly configured delay if that delay is smaller than the read delay.
        if read_delay > config_delay {
            Ok(config_delay)
        } else {
            Ok(read_delay)
        }
    }

    pub fn update_delay(&self) -> Result<()> {
        let path = Path::new("p").join(&self.name).join("delay");
        let mut delay = self.get_current_delay()?;

        if self.get_current_delay()? == 0 {
            delay = self.config.delay
        } else {
            delay -= 1
        }

        fs::write(path, delay.to_string())?;
        Ok(())
    }

    fn fetch_release(&self) -> Result<String> {
        let _upstream = &self.config.release.upstream.as_ref().unwrap_or(&self.config.upstream);

        let _name = format!("name={}", self.name);
        let _upstream = format!("upstream={}", get_longform(_upstream));
        let _shortform = format!("shortform={}", get_shortform(&_upstream));
        let _fetch = format!(". sh/lib.env && {}", self.config.release.fetch);

        let command = vec![
            "env",
            &_name,
            &_upstream,
            &_shortform,
            "bash",
            "-c",
            &_fetch,
        ];

        let ver = match cmd(command) {
            | Err(e) => bail!("Failed to fetch version: {e}"),
            | Ok(v) => v,
        };

        let mut version = Version::new(ver);
        version.trim(self);

        Ok(version.fmt)
    }

    fn fetch_unstable(&self) -> Result<String> {
        let _upstream = &self.config.unstable.upstream.as_ref().unwrap_or(&self.config.upstream);

        let _name = format!("name={}", self.name);
        let _upstream = format!("upstream={}", get_longform(_upstream));
        let _shortform = format!("shortform={}", get_shortform(&_upstream));
        let _fetch = format!(". sh/lib.env && {}", self.config.unstable.fetch);

        let command = vec![
            "env",
            &_name,
            &_upstream,
            &_shortform,
            "bash",
            "-c",
            &_fetch,
        ];

        let ver = match cmd(command) {
            | Err(e) => bail!("Failed to fetch version: {e}"),
            | Ok(v) => v,
        };

        let mut version = Version::new(ver);
        version.trim(self);

        Ok(version.fmt)
    }

    fn fetch_commit(&self) -> Result<String> {
        let _upstream = self.config.commit.upstream.as_ref().unwrap_or(&self.config.upstream);

        let _name = format!("name={}", self.name);
        let _upstream = format!("upstream={}", get_longform(_upstream));
        let _shortform = format!("shortform={}", get_shortform(&_upstream));
        let _fetch = format!(". sh/lib.env && {}", self.config.commit.fetch);

        let command = vec![
            "env",
            &_name,
            &_upstream,
            &_shortform,
            "bash",
            "-c",
            &_fetch,
        ];

        let ver = match cmd(command) {
            | Err(e) => bail!("Failed to fetch version: {e}"),
            | Ok(v) => v,
        };

        let mut version = Version::new(ver);
        version.trim(self);

        Ok(version.fmt)
    }
}
