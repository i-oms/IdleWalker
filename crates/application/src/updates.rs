//! Whether a newer release exists. The feed's "latest release" address redirects to the
//! newest stable tag, so the answer is in the address alone. Nothing is downloaded or
//! installed here: the browser does that.
use crate::ports::ReleaseFeed;
use crate::preferences::Preferences;
use std::sync::Arc;

const RELEASES: &str = "https://github.com/i-oms/IdleWalker/releases";

/// What is known about updates.
#[derive(Clone, Debug, PartialEq)]
pub enum UpdateState {
    Checking,
    UpToDate,
    /// The newer version, without the tag's "v".
    Available(String),
    Failed,
}

/// A version as numbers, so that 0.10.0 is newer than 0.9.0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    numbers: (u32, u32, u32),
    /// A plain release sorts after its own pre-releases.
    stable: bool,
}

fn parse(text: &str) -> Option<Version> {
    let text = text.trim();
    let text = text.strip_prefix('v').unwrap_or(text);
    let (core, pre) = match text.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (text, None),
    };
    let mut parts = core.split('.').map(|p| {
        // Digits only: this text ends up in an address.
        p.bytes()
            .all(|b| b.is_ascii_digit())
            .then(|| p.parse::<u32>().ok())
            .flatten()
    });
    let numbers = (parts.next()??, parts.next()??, parts.next()??);
    if parts.next().is_some() {
        return None;
    }
    Some(Version {
        numbers,
        stable: pre.is_none(),
    })
}

/// The tag in the address that the redirect ended at.
fn tag_from_url(url: &str) -> Option<&str> {
    url.trim()
        .strip_prefix(RELEASES)?
        .strip_prefix("/tag/")
        .filter(|t| !t.is_empty())
}

/// Compares the newest release with the running version.
fn judge(final_url: &str, current: &str) -> UpdateState {
    let Some(tag) = tag_from_url(final_url) else {
        return UpdateState::Failed;
    };
    let (Some(latest), Some(running)) = (parse(tag), parse(current)) else {
        return UpdateState::Failed;
    };
    if latest > running {
        UpdateState::Available(tag.trim_start_matches('v').to_string())
    } else {
        UpdateState::UpToDate
    }
}

/// One number per version, for the preferences.
fn stamp(version: &str) -> i32 {
    parse(version).map_or(0, |v| {
        let (a, b, c) = v.numbers;
        (a.min(2000) * 1_000_000 + b.min(999) * 1000 + c.min(999)) as i32
    })
}

#[derive(Clone)]
pub struct Updates {
    feed: Arc<dyn ReleaseFeed>,
    preferences: Preferences,
    current: String,
}

impl Updates {
    /// `current` is the version running.
    pub fn new(feed: Arc<dyn ReleaseFeed>, preferences: Preferences, current: String) -> Self {
        Updates {
            feed,
            preferences,
            current,
        }
    }

    pub fn current_version(&self) -> &str {
        &self.current
    }

    /// Asks the feed. Slow, and needs the network: off the UI thread.
    pub fn check(&self) -> UpdateState {
        match self.feed.latest_release_address() {
            Some(address) => judge(&address, &self.current),
            None => UpdateState::Failed,
        }
    }

    /// Where this OS's file for a version is downloaded from.
    pub fn download_url(&self, version: &str) -> String {
        download_url(version, &self.feed.download_file_name(version))
    }

    pub fn releases_url(&self) -> &'static str {
        RELEASES
    }

    /// The address whose redirect names the newest release.
    pub fn latest_release_link() -> String {
        format!("{RELEASES}/latest")
    }

    /// A background check shows the dialog once per version, not every few hours.
    pub fn already_announced(&self, version: &str) -> bool {
        self.preferences.load_update_announced() == stamp(version)
    }

    pub fn mark_announced(&self, version: &str) {
        self.preferences.save_update_announced(stamp(version));
    }
}

fn download_url(version: &str, file: &str) -> String {
    format!("{RELEASES}/download/v{version}/{file}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(t: &str) -> String {
        format!("{RELEASES}/tag/{t}")
    }

    #[test]
    fn numbers_compare_as_numbers() {
        assert_eq!(
            judge(&tag("v0.10.0"), "0.9.0"),
            UpdateState::Available("0.10.0".into())
        );
        assert_eq!(judge(&tag("v0.9.0"), "0.10.0"), UpdateState::UpToDate);
    }

    #[test]
    fn the_same_or_an_older_release_is_up_to_date() {
        assert_eq!(judge(&tag("v1.2.3"), "1.2.3"), UpdateState::UpToDate);
        assert_eq!(judge(&tag("v0.0.3"), "0.1.0"), UpdateState::UpToDate);
    }

    #[test]
    fn a_release_is_newer_than_its_own_pre_release() {
        assert_eq!(
            judge(&tag("v1.2.3"), "1.2.3-beta.1"),
            UpdateState::Available("1.2.3".into())
        );
        assert_eq!(judge(&tag("v1.2.3-beta.1"), "1.2.3"), UpdateState::UpToDate);
    }

    #[test]
    fn anything_unexpected_is_a_failure() {
        // With no release yet, the feed redirects to the plain releases page.
        assert_eq!(judge(RELEASES, "1.0.0"), UpdateState::Failed);
        assert_eq!(judge("", "1.0.0"), UpdateState::Failed);
        assert_eq!(judge(&tag(""), "1.0.0"), UpdateState::Failed);
        assert_eq!(
            judge("https://example.com/releases/tag/v9.0.0", "1.0.0"),
            UpdateState::Failed
        );
        assert_eq!(judge(&tag("v1.0"), "1.0.0"), UpdateState::Failed);
        assert_eq!(judge(&tag("v1.0.0.0"), "1.0.0"), UpdateState::Failed);
        assert_eq!(judge(&tag("v9.0.0&x"), "1.0.0"), UpdateState::Failed);
        // The commit hash that an untagged build carries is not a version.
        assert_eq!(judge(&tag("v1.0.0"), "abc123"), UpdateState::Failed);
    }

    #[test]
    fn each_version_has_its_own_stamp() {
        assert_ne!(stamp("0.1.0"), stamp("0.0.1"));
        assert_ne!(stamp("1.0.0"), stamp("0.999.0"));
        assert_eq!(stamp("0.1.0"), 1000);
        assert_eq!(stamp("nonsense"), 0);
    }

    #[test]
    fn the_download_has_the_name_the_release_gives_it() {
        let url = download_url("1.2.3", "idlewalker-1.2.3-linux-x64.zip");
        assert_eq!(
            url,
            format!("{RELEASES}/download/v1.2.3/idlewalker-1.2.3-linux-x64.zip")
        );
    }
}
