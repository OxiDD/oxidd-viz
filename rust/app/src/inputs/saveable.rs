use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

use crate::util::watchables::DynSignaller;

pub trait Saveable {
    type Val: Serialize + DeserializeOwned;

    /// Loads the state specified in the deserializer
    fn load<'de, D>(&mut self, deserializer: D) -> Result<DynSignaller, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = Self::Val::deserialize(deserializer)?;
        Ok(self.load_value(val))
    }

    /// Saves the state tot he serializer
    fn save<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.save_value().serialize(serializer)
    }

    // Loads the given value
    fn load_value(&mut self, val: Self::Val) -> DynSignaller;

    // Uses the value of the saveable
    fn save_value(&self) -> Self::Val;
}
