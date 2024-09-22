use anyhow::{Error, Ok, Result};
use serde::{Deserialize, Serialize};

use super::CustomRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Rule {
    Effect,
    Builtin,
    Relative,
    Alias,
    Npm,
    Custom(CustomRule),
}

impl Rule {
    pub fn regex(pattern: &str) -> Result<Rule> {
        let rule = CustomRule::try_from(pattern)?;
        Ok(Rule::Custom(rule))
    }
}

impl TryFrom<String> for Rule {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(&*value)
    }
}

impl TryFrom<&str> for Rule {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "<effect>" => Ok(Rule::Effect),
            "<builtin>" => Ok(Rule::Builtin),
            "<relative>" => Ok(Rule::Relative),
            "<alias>" => Ok(Rule::Alias),
            "<npm>" => Ok(Rule::Npm),
            x => Rule::regex(x),
        }
    }
}

impl From<Rule> for String {
    fn from(value: Rule) -> Self {
        match value {
            Rule::Effect => "<effect>".to_string(),
            Rule::Builtin => "<builtin>".to_string(),
            Rule::Relative => "<relative>".to_string(),
            Rule::Alias => "<alias>".to_string(),
            Rule::Npm => "<npm>".to_string(),
            Rule::Custom(rule) => rule.into(),
        }
    }
}
