use pyo3::pyclass;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Setting for handling Nice Items and Mother Maiamai Rewards
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[pyclass(eq, eq_int, hash, frozen)]
pub enum SuperItems {
    /// Two progressive copies of the Lamp and Net are shuffled.
    Shuffled,

    /// Only one copy of the Lamp and Net are shuffled.
    #[default]
    Off,

    /// Upgrades give you the Super Lamp or Super Net only if you already have the item.
    Upgrades,
}

impl TryFrom<u8> for SuperItems {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Shuffled),
            1 => Ok(Self::Off),
            2 => Ok(Self::Upgrades),

            _ => Err("Invalid SuperItems Setting: {}".to_owned()),
        }
    }
}

impl Display for SuperItems {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Shuffled => "Shuffled",
                Self::Off => "Off",
                Self::Upgrades => "Upgrades",
            }
        )
    }
}