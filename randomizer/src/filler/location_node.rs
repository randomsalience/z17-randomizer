use crate::filler::check::Check;
use crate::filler::path::Path;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct LocationNode {
    name: String,
    checks: Option<Vec<Check>>,
    paths: Option<Vec<Path>>,
}

impl LocationNode {
    pub fn new<C, P>(name: &'static str, checks: C, paths: P) -> Self
    where
        C: Into<Option<Vec<Check>>>,
        P: Into<Option<Vec<Path>>>,
    {
        Self { name: name.into(), checks: checks.into(), paths: paths.into() }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_checks(&self) -> &Option<Vec<Check>> {
        &self.checks
    }

    pub fn get_paths(&self) -> &Option<Vec<Path>> {
        &self.paths
    }
}
