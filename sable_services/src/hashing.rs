use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HashingError {
    #[error("bcrypt failed: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
}

const fn default_bcrypt_cost() -> u32 {
    bcrypt::DEFAULT_COST
}

/// [`bcrypt::Version`] but it's Serde-deserializable
///
/// [Bcrypt versions](https://en.wikipedia.org/wiki/Bcrypt#Versioning_history)
#[derive(Deserialize, Clone, Default)]
pub enum BcryptVersion {
    #[serde(rename = "2a")]
    TwoA,
    #[serde(rename = "2x")]
    TwoX,
    #[serde(rename = "2y")]
    TwoY,
    #[serde(rename = "2b")]
    #[default]
    TwoB,
}

impl From<BcryptVersion> for bcrypt::Version {
    fn from(v: BcryptVersion) -> bcrypt::Version {
        match v {
            BcryptVersion::TwoA => bcrypt::Version::TwoA,
            BcryptVersion::TwoX => bcrypt::Version::TwoX,
            BcryptVersion::TwoY => bcrypt::Version::TwoY,
            BcryptVersion::TwoB => bcrypt::Version::TwoB,
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(tag = "algorithm", rename_all = "lowercase")]
pub enum HashConfig {
    Bcrypt {
        #[serde(default = "default_bcrypt_cost")]
        cost: u32,
        #[serde(default)]
        version: BcryptVersion,
    },
}

impl Default for HashConfig {
    fn default() -> HashConfig {
        HashConfig::Bcrypt {
            cost: default_bcrypt_cost(),
            version: BcryptVersion::default(),
        }
    }
}

impl HashConfig {
    pub fn hash(&self, data: &str) -> Result<String, HashingError> {
        match self.clone() {
            HashConfig::Bcrypt { cost, version } => {
                Ok(bcrypt::hash_with_result(data, cost)?.format_for_version(version.into()))
            }
        }
    }
}
