//! Open hardware controller: named power profiles that can be saved, restored
//! and reverted to defaults when settings are managed externally.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{fs, io, path::Path};

/// Energy performance preference (amd-pstate / intel-pstate EPP).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Epp {
    Performance,
    BalancePerformance,
    BalancePower,
    Power,
}

/// A point of a fan curve: temperature (°C) -> fan duty (0-100 %).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanPoint {
    pub temp_c: u8,
    pub duty_pct: u8,
}

/// A complete, named power profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerProfile {
    pub epp: Epp,
    /// Powercap limits in microwatts, keyed by constraint name (e.g. "long_term").
    pub powercap_uw: BTreeMap<String, u64>,
    pub fan_curve: Vec<FanPoint>,
}

impl Default for PowerProfile {
    fn default() -> Self {
        Self {
            epp: Epp::BalancePerformance,
            powercap_uw: BTreeMap::new(),
            fan_curve: Vec::new(),
        }
    }
}

/// Store of named profiles with a built-in default.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileStore {
    profiles: BTreeMap<String, PowerProfile>,
    active: Option<String>,
    /// When true, settings are managed by another tool and the default is used.
    externally_managed: bool,
}

impl ProfileStore {
    pub fn save(&mut self, name: &str, profile: PowerProfile) {
        self.profiles.insert(name.to_owned(), profile);
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.profiles.keys().map(String::as_str)
    }

    /// Activate a saved profile; returns None if it does not exist.
    pub fn activate(&mut self, name: &str) -> Option<&PowerProfile> {
        if !self.profiles.contains_key(name) {
            return None;
        }
        self.active = Some(name.to_owned());
        self.externally_managed = false;
        self.profiles.get(name)
    }

    /// Mark settings as externally managed: fall back to defaults so this
    /// controller does not cross-talk with other tools.
    pub fn set_externally_managed(&mut self, managed: bool) {
        self.externally_managed = managed;
    }

    /// The profile that should currently be applied.
    pub fn effective(&self) -> PowerProfile {
        if self.externally_managed {
            return PowerProfile::default();
        }
        self.active
            .as_ref()
            .and_then(|n| self.profiles.get(n))
            .cloned()
            .unwrap_or_default()
    }

    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let data = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, data)
    }

    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        serde_json::from_str(&fs::read_to_string(path)?).map_err(io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PowerProfile {
        PowerProfile {
            epp: Epp::Power,
            powercap_uw: BTreeMap::from([("long_term".to_owned(), 15_000_000)]),
            fan_curve: vec![FanPoint { temp_c: 60, duty_pct: 40 }],
        }
    }

    #[test]
    fn activate_and_default_when_external() {
        let mut s = ProfileStore::default();
        s.save("quiet", sample());
        assert!(s.activate("missing").is_none());
        assert!(s.activate("quiet").is_some());
        assert_eq!(s.effective(), sample());
        s.set_externally_managed(true);
        assert_eq!(s.effective(), PowerProfile::default());
    }

    #[test]
    fn roundtrip_file() {
        let mut s = ProfileStore::default();
        s.save("quiet", sample());
        s.activate("quiet");
        let p = std::env::temp_dir().join("ohc_profiles_test.json");
        s.save_to_file(&p).unwrap();
        assert_eq!(ProfileStore::load_from_file(&p).unwrap(), s);
        let _ = fs::remove_file(p);
    }
}
