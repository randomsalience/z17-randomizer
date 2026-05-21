use pyo3::prelude::*;
use std::{fs::File, path::Path};
use simplelog::{LevelFilter, SimpleLogger, CombinedLogger, WriteLogger, SharedLogger};

#[pyfunction]
pub fn logging_on() {
    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![SimpleLogger::new(LevelFilter::Info, Default::default())];
    if let Ok(logfile) = File::create(Path::new("logs").join("albwrandomizer.log")) {
        loggers.push(WriteLogger::new(LevelFilter::Info, Default::default(), logfile));
    }
    CombinedLogger::init(loggers).expect("Could not initialize logger.");
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
