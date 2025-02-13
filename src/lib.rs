use pyo3::prelude::*;
use modinfo::settings::{
    cracks::Cracks,
    cracksanity::Cracksanity,
    keysy::Keysy,
    logic::LogicMode,
    nice_items::NiceItems,
    pedestal::PedestalSetting,
    ravios_shop::RaviosShop,
    trials_door::TrialsDoor,
    weather_vanes::WeatherVanes,
};
use modinfo::Settings;
use randomizer::{ArchipelagoItem, ArchipelagoInfo, SeedInfo, randomize_pre_fill};
use randomizer::filler::filler_item::{
    Item, Goal, Vane, Crack, PyRandomizable,
    new_item, new_goal, new_vane, new_crack
};
use simplelog::{LevelFilter, SimpleLogger};

#[pyfunction]
pub fn logging_on() {
    SimpleLogger::init(LevelFilter::Info, Default::default()).expect("Could not initialize logger.");
}

#[pymodule]
fn albwrandomizer(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Cracks>()?;
    m.add_class::<Cracksanity>()?;
    m.add_class::<Keysy>()?;
    m.add_class::<LogicMode>()?;
    m.add_class::<NiceItems>()?;
    m.add_class::<PedestalSetting>()?;
    m.add_class::<RaviosShop>()?;
    m.add_class::<TrialsDoor>()?;
    m.add_class::<WeatherVanes>()?;
    m.add_class::<Settings>()?;
    m.add_class::<ArchipelagoItem>()?;
    m.add_class::<ArchipelagoInfo>()?;
    m.add_class::<SeedInfo>()?;
    m.add_class::<Item>()?;
    m.add_class::<Goal>()?;
    m.add_class::<Vane>()?;
    m.add_class::<Crack>()?;
    m.add_class::<PyRandomizable>()?;

    m.add_function(wrap_pyfunction!(logging_on, m)?)?;
    m.add_function(wrap_pyfunction!(randomize_pre_fill, m)?)?;
    m.add_function(wrap_pyfunction!(new_item, m)?)?;
    m.add_function(wrap_pyfunction!(new_goal, m)?)?;
    m.add_function(wrap_pyfunction!(new_vane, m)?)?;
    m.add_function(wrap_pyfunction!(new_crack, m)?)?;
    Ok(())
}
