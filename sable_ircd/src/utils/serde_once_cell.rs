use serde::*;
use serde_with::*;
use std::sync::OnceLock;

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct WrapOption<T>(Option<T>);

impl<T: Serialize> SerializeAs<OnceLock<T>> for WrapOption<T> {
    fn serialize_as<S>(source: &OnceLock<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        source.get().serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> DeserializeAs<'de, OnceLock<T>> for WrapOption<T> {
    fn deserialize_as<D>(deserializer: D) -> Result<OnceLock<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<T> = Deserialize::deserialize(deserializer)?;
        Ok(match opt {
            None => OnceLock::new(),
            Some(x) => OnceLock::from(x),
        })
    }
}
