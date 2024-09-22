use dprint_core::configuration::{ConfigKeyMap, ConfigurationDiagnostic};
use regex::Regex;
use serde::{Deserialize, Serialize};

mod custom;
mod group;
mod rule;
mod set;

pub use custom::*;
pub use group::*;
pub use rule::*;
pub use set::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    #[serde(default = "get_default_aliases")]
    pub aliases: Vec<CustomRule>,
    #[serde(default = "get_default_groups")]
    pub groups: Vec<ImportGroup>,
}

impl Configuration {
    pub fn empty() -> Self {
        Self {
            aliases: vec![],
            groups: vec![],
        }
    }
}

fn get_default_aliases() -> Vec<CustomRule> {
    vec![CustomRule(Regex::new("^[@~]/").unwrap())]
}

fn get_default_groups() -> Vec<ImportGroup> {
    vec![
        ImportGroup {
            include: RuleSet::single(Rule::Effect),
            exclude: RuleSet::default(),
        },
        ImportGroup {
            include: RuleSet::single(Rule::Builtin),
            exclude: RuleSet::default(),
        },
        ImportGroup {
            include: RuleSet::single(Rule::Npm),
            exclude: RuleSet::default(),
        },
        ImportGroup {
            include: RuleSet::single(Rule::Alias),
            exclude: RuleSet::default(),
        },
        ImportGroup {
            include: RuleSet::single(Rule::Relative),
            exclude: RuleSet::default(),
        },
    ]
}

pub fn resolve_config(raw: &mut ConfigKeyMap) -> Result<Configuration, ConfigurationDiagnostic> {
    let json = serde_json::to_value(&raw).unwrap();

    raw.swap_remove("aliases");
    raw.swap_remove("groups");

    let mut config: Configuration = match serde_path_to_error::deserialize(json) {
        Ok(it) => it,
        Err(err) => {
            return Err(ConfigurationDiagnostic {
                property_name: err.path().to_string(),
                message: err.to_string(),
            });
        }
    };

    if config.aliases.is_empty() {
        config.aliases = get_default_aliases();
    }

    if config.groups.is_empty() {
        config.groups = get_default_groups();
    }

    Ok(config)
}
