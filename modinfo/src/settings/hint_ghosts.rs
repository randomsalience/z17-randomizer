use pyo3::pyclass;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[pyclass(eq, eq_int, hash, frozen)]
pub enum HintGhosts {
    Off,
    #[default]
    Always,
    Glasses,
}
