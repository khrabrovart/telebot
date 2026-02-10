// generate according to usages
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SchedulerEvent {
    pub posting_rule_id: String,
}
