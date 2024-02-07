use super::*;
use crate::network::*;

use std::collections::HashMap;

// Convenience alias for the engine type
type Engine = chert::compile::Engine<PreRegistrationBanSettings, NetworkBanId>;

/// A collection of network bans, supporting efficient lookup based on
/// (partial) user details
#[derive(Debug, Clone)]
pub struct BanRepository {
    all_bans: HashMap<NetworkBanId, state::NetworkBan>,

    engine: Engine,
}

impl BanRepository {
    pub fn new() -> Self {
        let all_bans = HashMap::new();
        Self {
            engine: Self::compile_engine(&all_bans),
            all_bans,
        }
    }

    pub fn from_ban_set(bans: Vec<state::NetworkBan>) -> Self {
        let mut all_bans = HashMap::new();

        for ban in bans {
            all_bans.insert(ban.id, ban);
        }

        let engine = Self::compile_engine(&all_bans);

        Self { all_bans, engine }
    }

    pub fn add(&mut self, ban: state::NetworkBan) {
        self.all_bans.insert(ban.id, ban);
        self.recompile();
    }

    pub fn remove(&mut self, id: NetworkBanId) {
        if self.all_bans.remove(&id).is_some() {
            self.recompile();
        }
    }

    pub fn get(&self, id: &NetworkBanId) -> Option<&state::NetworkBan> {
        self.all_bans.get(id)
    }

    pub fn find(&self, matching: &PreRegistrationBanSettings) -> Option<&state::NetworkBan> {
        let matches = self.engine.eval(&matching);

        matches.get(0).and_then(|id| self.all_bans.get(id))
    }

    fn recompile(&mut self) {
        self.engine = Self::compile_engine(&self.all_bans);
    }

    fn compile_engine(bans: &HashMap<NetworkBanId, state::NetworkBan>) -> Engine {
        chert::compile::compile_unsafe(bans.iter().map(|(k, v)| (*k, &v.pattern)))
    }
}

impl serde::ser::Serialize for BanRepository {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.all_bans.values())
    }
}

impl<'de> serde::de::Deserialize<'de> for BanRepository {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bans = Vec::deserialize(deserializer)?;
        Ok(Self::from_ban_set(bans))
    }
}
