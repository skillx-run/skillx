use serde::{Deserialize, Serialize};
use std::fmt;

/// Injection scope: project-level or global-level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Project,
    Global,
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scope::Project => write!(f, "project"),
            Scope::Global => write!(f, "global"),
        }
    }
}

impl std::str::FromStr for Scope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "project" => Ok(Scope::Project),
            "global" => Ok(Scope::Global),
            _ => Err(format!("invalid scope: '{s}', expected 'project' or 'global'")),
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Scope::Global
    }
}
