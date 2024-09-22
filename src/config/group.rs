use serde::{Deserialize, Serialize};

use super::RuleSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportGroup {
    #[serde(default)]
    pub include: RuleSet,
    #[serde(default)]
    pub exclude: RuleSet,
}
