use pyo3::prelude::*;
use simplelog::{LevelFilter, SimpleLogger};

#[pyfunction]
pub fn logging_on() {
    SimpleLogger::init(LevelFilter::Info, Default::default()).expect("Could not initialize logger.");
}

#[pymodule]
mod albwrandomizer {
    #[pymodule_export]
    use modinfo::settings::{
        Cracks, Cracksanity, HintGhosts, Keysy, LogicMode, NiceItems,
        PedestalSetting, RaviosShop, TrialsDoor, WeatherVanes
    };

    #[pymodule_export]
    use modinfo::Settings;

    #[pymodule_export]
    use randomizer::{
        ArchipelagoItem, ArchipelagoInfo, SeedInfo,
        filler::filler_item::{
            Item, Goal, Vane, Crack, PyRandomizable,
            new_item, new_goal, new_vane, new_crack
        },
        hints::set_custom_hints,
        randomize_pre_fill
    };

    #[pymodule_export]
    use super::logging_on;
}
