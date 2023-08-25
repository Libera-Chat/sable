use once_cell::sync::OnceCell;
use serde::*;
use serde_with::*;

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct WrapOption<T>(Option<T>);

impl<T: Serialize> SerializeAs<OnceCell<T>> for WrapOption<T> {
    fn serialize_as<S>(source: &OnceCell<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        source.get().serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> DeserializeAs<'de, OnceCell<T>> for WrapOption<T> {
    fn deserialize_as<D>(deserializer: D) -> Result<OnceCell<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<T> = Deserialize::deserialize(deserializer)?;
        Ok(match opt {
            None => OnceCell::new(),
            Some(x) => OnceCell::with_value(x),
        })
    }
}
