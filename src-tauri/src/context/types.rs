use serde::{Deserialize, Serialize};

use crate::provider::ModelRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextTask {
    DeveloperChat,
    ArcPlanning,
    ChapterPlanning,
    ScenePlanning,
    Writing,
    ContinuityCheck,
    MemoryExtraction,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextBlockKind {
    System,
    Project,
    Character,
    CharacterState,
    WorkingMemory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingMemoryBlock {
    pub id: String,
    pub label: String,
    pub content: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBudget {
    pub output_reserve_tokens: Option<u32>,
    pub safety_margin_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextCompileRequest {
    pub project_id: String,
    pub task: ContextTask,
    pub model: ModelRef,
    pub system_instructions: String,
    pub character_ids: Vec<String>,
    pub include_character_states: bool,
    pub working_memory: Vec<WorkingMemoryBlock>,
    pub budget: ContextBudget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OmissionReason {
    BudgetExceeded,
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBlock {
    pub kind: ContextBlockKind,
    pub source_id: Option<String>,
    pub content: String,
    pub estimated_tokens: u32,
    pub priority: u8,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextOmission {
    pub source_id: Option<String>,
    pub kind: ContextBlockKind,
    pub reason: OmissionReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledContext {
    pub project_id: String,
    pub task: ContextTask,
    pub model: ModelRef,
    pub blocks: Vec<ContextBlock>,
    pub omissions: Vec<ContextOmission>,
    pub input_budget_tokens: u32,
    pub estimated_input_tokens: u32,
}

#[cfg(test)]
mod tests {
    use crate::context::compiler::{CharTokenEstimator, TokenEstimator};

    #[test]
    fn char_estimator_rounds_up_and_keeps_empty_zero() {
        let estimator = CharTokenEstimator;
        assert_eq!(estimator.estimate(""), 0);
        assert_eq!(estimator.estimate("abcd"), 1);
        assert_eq!(estimator.estimate("abcde"), 2);
    }
}
