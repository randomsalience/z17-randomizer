use pyo3::pyclass;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[pyclass]
pub enum LogicMode {
    #[default]
    Normal,
    Hard,
    Glitched,
    AdvGlitched,
    Hell,
    NoLogic,
}
