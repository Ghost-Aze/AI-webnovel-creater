use crate::domain::character::{Character, CharacterState};
use crate::domain::project::Project;
use crate::error::{AppError, AppResult};
use crate::provider::ModelProfile;

use super::types::{
    CompiledContext, ContextBlock, ContextBlockKind, ContextCompileRequest, ContextOmission,
    OmissionReason,
};

pub trait ContextSource: Sync {
    fn load_project(&self, project_id: &str) -> AppResult<Project>;
    fn load_character(&self, project_id: &str, character_id: &str) -> AppResult<Character>;
    fn load_character_state(&self, character_id: &str) -> AppResult<CharacterState>;
}

pub trait TokenEstimator {
    fn estimate(&self, text: &str) -> u32;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CharTokenEstimator;

impl TokenEstimator for CharTokenEstimator {
    fn estimate(&self, text: &str) -> u32 {
        let characters = text.chars().count();
        if characters == 0 {
            0
        } else {
            characters.div_ceil(4) as u32
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ContextCompiler<E = CharTokenEstimator> {
    estimator: E,
}

impl Default for ContextCompiler<CharTokenEstimator> {
    fn default() -> Self {
        Self::new(CharTokenEstimator)
    }
}

impl<E: TokenEstimator> ContextCompiler<E> {
    pub fn new(estimator: E) -> Self {
        Self { estimator }
    }

    pub fn compile(
        &self,
        request: ContextCompileRequest,
        profile: &ModelProfile,
        source: &dyn ContextSource,
    ) -> AppResult<CompiledContext> {
        if request.project_id.trim().is_empty() || request.system_instructions.trim().is_empty() {
            return Err(AppError::Validation {
                message: "Context project and system instructions are required.".into(),
            });
        }
        if request.model.provider_id != profile.provider_id
            || request.model.model_id != profile.model_id
        {
            return Err(AppError::Validation {
                message: "Context model does not match the selected profile.".into(),
            });
        }
        let output_reserve = request
            .budget
            .output_reserve_tokens
            .unwrap_or(profile.default_output_tokens);
        if profile.context_window_tokens <= output_reserve {
            return Err(AppError::Validation {
                message: "Model context window must leave room for input.".into(),
            });
        }
        let input_budget_tokens = profile
            .context_window_tokens
            .checked_sub(output_reserve)
            .and_then(|value| value.checked_sub(request.budget.safety_margin_tokens))
            .ok_or_else(|| AppError::Validation {
                message: "Context input budget must be greater than zero.".into(),
            })?;
        if input_budget_tokens == 0 {
            return Err(AppError::Validation {
                message: "Context input budget must be greater than zero.".into(),
            });
        }

        let project = source.load_project(&request.project_id)?;
        let mut candidates = vec![Candidate {
            kind: ContextBlockKind::System,
            source_id: None,
            content: request.system_instructions.trim().to_string(),
            priority: 100,
        }];
        candidates.push(Candidate {
            kind: ContextBlockKind::Project,
            source_id: Some(project.id.clone()),
            content: format!(
                "Project: {}\nDescription: {}",
                project.name, project.description
            ),
            priority: 80,
        });

        let mut selected_states = Vec::new();
        for character_id in request.character_ids {
            let character = source.load_character(&request.project_id, &character_id)?;
            candidates.push(Candidate {
                kind: ContextBlockKind::Character,
                source_id: Some(character.id.clone()),
                content: format!(
                    "Character: {}\nSummary: {}\nRole: {}",
                    character.name, character.summary, character.role
                ),
                priority: 60,
            });
            if request.include_character_states {
                selected_states.push(source.load_character_state(&character.id)?);
            }
        }

        for state in selected_states {
            candidates.push(Candidate {
                kind: ContextBlockKind::CharacterState,
                source_id: Some(state.character_id.clone()),
                content: serde_json::to_string(&state).map_err(|_| AppError::Internal)?,
                priority: 50,
            });
        }

        for working_memory in request.working_memory {
            candidates.push(Candidate {
                kind: ContextBlockKind::WorkingMemory,
                source_id: Some(working_memory.id),
                content: format!("{}: {}", working_memory.label, working_memory.content),
                priority: 40,
            });
        }

        let mut blocks = Vec::new();
        let mut omissions = Vec::new();
        let mut estimated_input_tokens = 0;
        for candidate in candidates {
            let remaining = input_budget_tokens.saturating_sub(estimated_input_tokens);
            if remaining == 0 {
                omissions.push(ContextOmission {
                    source_id: candidate.source_id,
                    kind: candidate.kind,
                    reason: OmissionReason::BudgetExceeded,
                });
                continue;
            }
            let full_tokens = self.estimator.estimate(&candidate.content);
            if full_tokens <= remaining {
                estimated_input_tokens += full_tokens;
                blocks.push(ContextBlock {
                    kind: candidate.kind,
                    source_id: candidate.source_id,
                    content: candidate.content,
                    estimated_tokens: full_tokens,
                    priority: candidate.priority,
                    truncated: false,
                });
                continue;
            }

            let prefix = prefix_that_fits(&self.estimator, &candidate.content, remaining);
            if prefix.is_empty() {
                omissions.push(ContextOmission {
                    source_id: candidate.source_id,
                    kind: candidate.kind,
                    reason: OmissionReason::BudgetExceeded,
                });
                continue;
            }
            let estimated_tokens = self.estimator.estimate(&prefix);
            estimated_input_tokens += estimated_tokens;
            let source_id = candidate.source_id.clone();
            blocks.push(ContextBlock {
                kind: candidate.kind,
                source_id,
                content: prefix,
                estimated_tokens,
                priority: candidate.priority,
                truncated: true,
            });
            omissions.push(ContextOmission {
                source_id: candidate.source_id,
                kind: candidate.kind,
                reason: OmissionReason::Truncated,
            });
        }

        Ok(CompiledContext {
            project_id: request.project_id,
            task: request.task,
            model: request.model,
            blocks,
            omissions,
            input_budget_tokens,
            estimated_input_tokens,
        })
    }
}

struct Candidate {
    kind: ContextBlockKind,
    source_id: Option<String>,
    content: String,
    priority: u8,
}

fn prefix_that_fits<E: TokenEstimator>(estimator: &E, content: &str, budget: u32) -> String {
    let mut prefix = String::new();
    for character in content.chars() {
        let mut candidate = prefix.clone();
        candidate.push(character);
        if estimator.estimate(&candidate) > budget {
            break;
        }
        prefix.push(character);
    }
    prefix
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::*;
    use crate::domain::character::{CharacterStatus, UpdateCharacterStateInput};
    use crate::domain::project::ProjectStatus;
    use crate::domain::revision::CanonStatus;
    use crate::provider::{ModelProfile, ModelRef, ProviderCapabilities};

    struct FakeSource {
        project: Project,
        character: Character,
        state: CharacterState,
        second_character: Option<(Character, CharacterState)>,
    }

    impl ContextSource for FakeSource {
        fn load_project(&self, _project_id: &str) -> AppResult<Project> {
            Ok(self.project.clone())
        }

        fn load_character(&self, _project_id: &str, _character_id: &str) -> AppResult<Character> {
            if let Some((character, _)) = &self.second_character {
                if _character_id == character.id {
                    return Ok(character.clone());
                }
            }
            Ok(self.character.clone())
        }

        fn load_character_state(&self, character_id: &str) -> AppResult<CharacterState> {
            if let Some((character, state)) = &self.second_character {
                if character_id == character.id {
                    return Ok(state.clone());
                }
            }
            Ok(self.state.clone())
        }
    }

    fn source() -> FakeSource {
        let project = Project {
            id: "project-1".into(),
            name: "Project One".into(),
            description: "A premise".into(),
            status: ProjectStatus::Active,
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        let character = Character {
            id: "character-1".into(),
            project_id: project.id.clone(),
            name: "Ji-an".into(),
            summary: "Courier".into(),
            role: "Lead".into(),
            status: CharacterStatus::Active,
            revision: 1,
            canon_status: CanonStatus::Canon,
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        let mut state = CharacterState::empty(character.id.clone());
        state.apply(UpdateCharacterStateInput {
            current_location: Some("North gate".into()),
            ..Default::default()
        });
        FakeSource {
            project,
            character,
            state,
            second_character: None,
        }
    }

    fn two_character_source() -> FakeSource {
        let mut source = source();
        let character = Character {
            id: "character-2".into(),
            project_id: source.project.id.clone(),
            name: "Mira".into(),
            summary: "Scout".into(),
            role: "Support".into(),
            status: CharacterStatus::Active,
            revision: 1,
            canon_status: CanonStatus::Canon,
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        let state = CharacterState::empty(character.id.clone());
        source.second_character = Some((character, state));
        source
    }

    fn profile(context_window_tokens: u32, default_output_tokens: u32) -> ModelProfile {
        ModelProfile {
            provider_id: "mock".into(),
            model_id: "mock-small".into(),
            display_name: "Mock Small".into(),
            context_window_tokens,
            default_output_tokens,
            strengths: Vec::new(),
            weaknesses: Vec::new(),
            strategy: Vec::new(),
            tier: crate::provider::ModelTier::Medium,
            capabilities: ProviderCapabilities::default(),
        }
    }

    fn request() -> ContextCompileRequest {
        ContextCompileRequest {
            project_id: "project-1".into(),
            task: ContextTask::Writing,
            model: ModelRef {
                provider_id: "mock".into(),
                model_id: "mock-small".into(),
            },
            system_instructions: "Write clearly".into(),
            character_ids: vec!["character-1".into()],
            include_character_states: true,
            working_memory: Vec::new(),
            budget: ContextBudget {
                output_reserve_tokens: None,
                safety_margin_tokens: 0,
            },
        }
    }

    #[derive(Clone, Copy)]
    struct CharacterEstimator;

    impl TokenEstimator for CharacterEstimator {
        fn estimate(&self, text: &str) -> u32 {
            text.chars().count() as u32
        }
    }

    #[test]
    fn rejects_empty_system_instruction() {
        let mut input = request();
        input.system_instructions = "  ".into();
        let error = ContextCompiler::default()
            .compile(input, &profile(100, 10), &source())
            .expect_err("empty system instructions must fail");
        assert!(matches!(error, crate::error::AppError::Validation { .. }));
    }

    #[test]
    fn rejects_zero_or_insufficient_budget() {
        let mut input = request();
        input.budget.safety_margin_tokens = 100;
        let error = ContextCompiler::default()
            .compile(input, &profile(100, 10), &source())
            .expect_err("zero input budget must fail");
        assert!(matches!(error, crate::error::AppError::Validation { .. }));

        let error = ContextCompiler::default()
            .compile(request(), &profile(10, 10), &source())
            .expect_err("model without output room must fail");
        assert!(matches!(error, crate::error::AppError::Validation { .. }));
    }

    #[test]
    fn preserves_priority_while_truncating_lower_blocks() {
        let mut input = request();
        input.character_ids.clear();
        input.working_memory = vec![WorkingMemoryBlock {
            id: "note-1".into(),
            label: "Note".into(),
            content: "abcdefghij".into(),
        }];
        let compiler = ContextCompiler::new(CharacterEstimator);
        let result = compiler
            .compile(input, &profile(25, 5), &source())
            .expect("context should compile");
        assert_eq!(
            result.blocks.first().unwrap().kind,
            ContextBlockKind::System
        );
        assert!(result.blocks.iter().any(|block| block.truncated));
        assert!(result.estimated_input_tokens <= result.input_budget_tokens);
    }

    #[test]
    fn includes_all_characters_before_character_states() {
        let mut input = request();
        input.character_ids = vec!["character-1".into(), "character-2".into()];
        let result = ContextCompiler::new(CharacterEstimator)
            .compile(input, &profile(128, 1), &two_character_source())
            .expect("selected character records should fit before their states");
        assert!(result
            .blocks
            .iter()
            .any(|block| block.source_id.as_deref() == Some("character-1")));
        assert!(result
            .blocks
            .iter()
            .any(|block| block.source_id.as_deref() == Some("character-2")));
    }

    #[test]
    fn reports_omitted_blocks() {
        let mut input = request();
        input.working_memory = vec![WorkingMemoryBlock {
            id: "note-1".into(),
            label: "Note".into(),
            content: "a very long note".into(),
        }];
        let result = ContextCompiler::new(CharacterEstimator)
            .compile(input, &profile(20, 19), &source())
            .expect("system block can use the one-token input budget");
        assert!(result
            .omissions
            .iter()
            .any(|omission| omission.source_id.as_deref() == Some("note-1")));
    }

    #[test]
    fn includes_explicit_working_memory_only() {
        let mut input = request();
        input.character_ids.clear();
        input.include_character_states = false;
        input.working_memory = vec![WorkingMemoryBlock {
            id: "explicit".into(),
            label: "Working note".into(),
            content: "Only this note".into(),
        }];
        let result = ContextCompiler::default()
            .compile(input, &profile(4096, 512), &source())
            .unwrap();
        assert!(result
            .blocks
            .iter()
            .any(|block| block.source_id.as_deref() == Some("explicit")));
        assert!(!result
            .blocks
            .iter()
            .any(|block| block.source_id.as_deref() == Some("character-1")));
    }

    #[test]
    fn uses_model_window_for_input_budget() {
        let input = request();
        let result = ContextCompiler::default()
            .compile(input, &profile(4096, 512), &source())
            .unwrap();
        assert_eq!(result.input_budget_tokens, 3584);
    }
}
