use chert::ChertStructTrait;

use super::*;
use crate::network::*;

use std::collections::HashMap;

// Convenience alias for the engine type
type Engine<V> = chert::compile::Engine<V, NetworkBanId>;

/// A collection of network bans, supporting efficient lookup based on
/// (partial) user details
#[derive(Debug, Clone)]
pub struct BanRepository {
    pre_registration_bans: HashMap<NetworkBanId, state::NetworkBan>,
    new_connection_bans: HashMap<NetworkBanId, state::NetworkBan>,
    pre_sasl_bans: HashMap<NetworkBanId, state::NetworkBan>,

    pre_registration_engine: Engine<PreRegistrationBanSettings>,
    new_connection_engine: Engine<NewConnectionBanSettings>,
    pre_sasl_engine: Engine<PreSaslBanSettings>,
}

impl BanRepository {
    pub fn new() -> Self {
        let pre_registration_bans = HashMap::new();
        let new_connection_bans = HashMap::new();
        let pre_sasl_bans = HashMap::new();
        Self {
            pre_registration_engine: Self::compile_engine(&pre_registration_bans),
            new_connection_engine: Self::compile_engine(&new_connection_bans),
            pre_sasl_engine: Self::compile_engine(&pre_sasl_bans),
            pre_registration_bans,
            new_connection_bans,
            pre_sasl_bans,
        }
    }

    pub fn from_ban_set(bans: Vec<state::NetworkBan>) -> Self {
        let mut pre_registration_bans = HashMap::new();
        let mut new_connection_bans = HashMap::new();
        let mut pre_sasl_bans = HashMap::new();

        for ban in bans {
            use BanMatchType::*;
            match ban.match_type {
                PreRegistration => pre_registration_bans.insert(ban.id, ban),
                NewConnection => new_connection_bans.insert(ban.id, ban),
                PreSasl => pre_sasl_bans.insert(ban.id, ban),
            };
        }

        Self {
            pre_registration_engine: Self::compile_engine(&pre_registration_bans),
            new_connection_engine: Self::compile_engine(&new_connection_bans),
            pre_sasl_engine: Self::compile_engine(&pre_sasl_bans),
            pre_registration_bans,
            new_connection_bans,
            pre_sasl_bans,
        }
    }

    pub fn add(&mut self, ban: state::NetworkBan) {
        use BanMatchType::*;
        match ban.match_type {
            PreRegistration => {
                self.pre_registration_bans.insert(ban.id, ban);
                self.pre_registration_engine = Self::compile_engine(&self.pre_registration_bans);
            }
            NewConnection => {
                self.new_connection_bans.insert(ban.id, ban);
                self.new_connection_engine = Self::compile_engine(&self.new_connection_bans);
            }
            PreSasl => {
                self.pre_sasl_bans.insert(ban.id, ban);
                self.pre_sasl_engine = Self::compile_engine(&self.pre_sasl_bans);
            }
        };
    }

    pub fn remove(&mut self, id: NetworkBanId) {
        if self.pre_registration_bans.remove(&id).is_some() {
            self.pre_registration_engine = Self::compile_engine(&self.pre_registration_bans);
        }
        if self.new_connection_bans.remove(&id).is_some() {
            self.new_connection_engine = Self::compile_engine(&self.new_connection_bans);
        }
        if self.pre_sasl_bans.remove(&id).is_some() {
            self.pre_sasl_engine = Self::compile_engine(&self.pre_sasl_bans);
        }
    }

    pub fn get(&self, id: &NetworkBanId) -> Option<&state::NetworkBan> {
        self.pre_registration_bans
            .get(id)
            .or_else(|| self.new_connection_bans.get(id))
            .or_else(|| self.pre_sasl_bans.get(id))
    }

    pub fn find_pre_registration(
        &self,
        matching: &PreRegistrationBanSettings,
    ) -> impl Iterator<Item = &state::NetworkBan> {
        let matches = self.pre_registration_engine.eval(matching);

        matches
            .into_iter()
            .filter_map(move |id| self.pre_registration_bans.get(id))
    }

    pub fn find_new_connection(
        &self,
        matching: &NewConnectionBanSettings,
    ) -> impl Iterator<Item = &state::NetworkBan> {
        let matches = self.new_connection_engine.eval(matching);

        matches
            .into_iter()
            .filter_map(move |id| self.new_connection_bans.get(id))
    }

    pub fn find_pre_sasl(
        &self,
        matching: &PreSaslBanSettings,
    ) -> impl Iterator<Item = &state::NetworkBan> {
        let matches = self.pre_sasl_engine.eval(matching);

        matches
            .into_iter()
            .filter_map(move |id| self.pre_sasl_bans.get(id))
    }

    fn compile_engine<V: ChertStructTrait>(
        bans: &HashMap<NetworkBanId, state::NetworkBan>,
    ) -> Engine<V> {
        chert::compile::compile_unsafe(bans.iter().map(|(k, v)| (*k, &v.pattern)))
    }
}

impl serde::ser::Serialize for BanRepository {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(
            self.pre_registration_bans
                .values()
                .chain(self.new_connection_bans.values())
                .chain(self.pre_sasl_bans.values()),
        )
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
