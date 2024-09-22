use anyhow::{bail, Error};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CustomRule(pub Regex);

impl CustomRule {
    pub fn matches(&self, module: &str) -> bool {
        self.0.is_match(module)
    }
}

impl TryFrom<String> for CustomRule {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(&*value)
    }
}

impl TryFrom<&str> for CustomRule {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.starts_with('<') || value.ends_with('>') {
            bail!("invalid rule");
        }

        if value.is_empty() {
            bail!("invalid regex pattern")
        }

        let re = Regex::new(value)?;
        Ok(CustomRule(re))
    }
}

impl From<CustomRule> for String {
    fn from(value: CustomRule) -> Self {
        value.0.to_string()
    }
}
