use serde::{Deserialize, Serialize};

use super::Rule;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent, default, rename_all = "camelCase")]
pub struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    pub fn single(rule: Rule) -> Self {
        Self { rules: vec![rule] }
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ Rule> {
        self.rules.iter()
    }
}
