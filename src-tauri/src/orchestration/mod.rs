use serde::{Deserialize, Serialize};

use crate::context::{
    CompiledContext, ContextBlockKind, ContextBudget, ContextCompiler, ContextSource, ContextTask,
    WorkingMemoryBlock,
};
use crate::error::{AppError, AppResult};
use crate::provider::{
    GenerateRequest, GenerateResponse, ModelRef, ModelRouter, ModelTask, PromptMessage, PromptRole,
    ProviderCapabilities, ProviderRegistry, QualityMode, RouteDecision, RoutingRequest,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicalAgent {
    Planner,
    Writer,
    PlotAnalyst,
    ContinuityCritic,
    StyleCritic,
    Revision,
    MemoryCurator,
}

impl LogicalAgent {
    fn label(self) -> &'static str {
        match self {
            Self::Planner => "planner",
            Self::Writer => "writer",
            Self::PlotAnalyst => "plot analyst",
            Self::ContinuityCritic => "continuity critic",
            Self::StyleCritic => "style critic",
            Self::Revision => "revision",
            Self::MemoryCurator => "memory curator",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationStep {
    pub id: String,
    pub agent: LogicalAgent,
    pub task: ModelTask,
    pub quality: QualityMode,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationPlan {
    pub task: ModelTask,
    pub quality: QualityMode,
    pub steps: Vec<OrchestrationStep>,
}

impl OrchestrationPlan {
    pub fn for_request(task: ModelTask, quality: QualityMode) -> Self {
        let steps = match task {
            ModelTask::MainWriting => writing_steps(quality),
            ModelTask::MajorRevision => revision_steps(quality),
            _ => vec![step("task", agent_for(task), task, quality, &[])],
        };
        Self {
            task,
            quality,
            steps,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestrationRequest {
    pub project_id: String,
    pub task: ModelTask,
    pub quality: QualityMode,
    pub preferred_model: Option<ModelRef>,
    pub required_capabilities: ProviderCapabilities,
    pub minimum_context_window_tokens: Option<u32>,
    pub system_instructions: String,
    pub character_ids: Vec<String>,
    pub include_character_states: bool,
    pub working_memory: Vec<WorkingMemoryBlock>,
    pub context_budget: ContextBudget,
    pub user_prompt: String,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestrationStepResult {
    pub step: OrchestrationStep,
    pub route: RouteDecision,
    pub context: CompiledContext,
    pub response: GenerateResponse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestrationResult {
    pub plan: OrchestrationPlan,
    pub steps: Vec<OrchestrationStepResult>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NarrativeOrchestrator;

impl NarrativeOrchestrator {
    pub async fn run(
        &self,
        registry: &ProviderRegistry,
        compiler: &ContextCompiler,
        source: &dyn ContextSource,
        request: OrchestrationRequest,
    ) -> AppResult<OrchestrationResult> {
        validate_request(&request)?;
        let plan = OrchestrationPlan::for_request(request.task, request.quality);
        let mut results = Vec::with_capacity(plan.steps.len());

        for step in &plan.steps {
            let route = ModelRouter.route(
                registry,
                RoutingRequest {
                    task: step.task,
                    quality: step.quality,
                    preferred_model: request.preferred_model.clone(),
                    required_capabilities: request.required_capabilities,
                    minimum_context_window_tokens: request.minimum_context_window_tokens,
                },
            )?;
            let context_request = context_request(&request, step, &results, route.model.clone());
            let context = compiler.compile(context_request, &route.profile, source)?;
            let messages = prompt_messages(&context, &request.user_prompt);
            let resolved = registry.resolve(&route.model.provider_id, &route.model.model_id)?;
            let response = resolved
                .provider
                .generate(GenerateRequest {
                    model: route.model.clone(),
                    messages,
                    max_output_tokens: route.profile.default_output_tokens,
                    temperature: request.temperature,
                })
                .await
                .map_err(AppError::from)?;
            results.push(OrchestrationStepResult {
                step: step.clone(),
                route,
                context,
                response,
            });
        }

        Ok(OrchestrationResult {
            plan,
            steps: results,
        })
    }
}

fn validate_request(request: &OrchestrationRequest) -> AppResult<()> {
    if request.project_id.trim().is_empty()
        || request.system_instructions.trim().is_empty()
        || request.user_prompt.trim().is_empty()
    {
        return Err(AppError::Validation {
            message: "Orchestration project, instructions and prompt are required.".into(),
        });
    }
    if let Some(temperature) = request.temperature {
        if !temperature.is_finite() || !(0.0..=2.0).contains(&temperature) {
            return Err(AppError::Validation {
                message: "Orchestration temperature must be between 0 and 2.".into(),
            });
        }
    }
    Ok(())
}

fn context_request(
    request: &OrchestrationRequest,
    step: &OrchestrationStep,
    previous: &[OrchestrationStepResult],
    model: ModelRef,
) -> crate::context::ContextCompileRequest {
    let mut working_memory = request.working_memory.clone();
    for result in previous {
        working_memory.push(WorkingMemoryBlock {
            id: format!("orchestration:{}", result.step.id),
            label: format!("Previous {} output", result.step.agent.label()),
            content: result.response.text.clone(),
        });
    }
    crate::context::ContextCompileRequest {
        project_id: request.project_id.clone(),
        task: context_task(step.task),
        model,
        system_instructions: format!(
            "{}\n\nExecution role: {}. Complete only this step and return its result for the next step.",
            request.system_instructions.trim(),
            step.agent.label()
        ),
        character_ids: request.character_ids.clone(),
        include_character_states: request.include_character_states,
        working_memory,
        budget: request.context_budget,
    }
}

fn prompt_messages(context: &CompiledContext, user_prompt: &str) -> Vec<PromptMessage> {
    let mut messages = context
        .blocks
        .iter()
        .map(|block| PromptMessage {
            role: if block.kind == ContextBlockKind::System {
                PromptRole::System
            } else {
                PromptRole::User
            },
            content: block.content.clone(),
        })
        .collect::<Vec<_>>();
    messages.push(PromptMessage {
        role: PromptRole::User,
        content: user_prompt.trim().to_string(),
    });
    messages
}

fn context_task(task: ModelTask) -> ContextTask {
    match task {
        ModelTask::ArcPlanning => ContextTask::ArcPlanning,
        ModelTask::ChapterPlanning => ContextTask::ChapterPlanning,
        ModelTask::ScenePlanning => ContextTask::ScenePlanning,
        ModelTask::MainWriting | ModelTask::MajorRevision => ContextTask::Writing,
        ModelTask::ContinuityCheck | ModelTask::PlotAnalysis | ModelTask::StyleCheck => {
            ContextTask::ContinuityCheck
        }
        ModelTask::MemoryExtraction => ContextTask::MemoryExtraction,
        other => ContextTask::Custom(format!("orchestration:{other:?}")),
    }
}

fn agent_for(task: ModelTask) -> LogicalAgent {
    match task {
        ModelTask::ContinuityCheck => LogicalAgent::ContinuityCritic,
        ModelTask::StyleCheck => LogicalAgent::StyleCritic,
        ModelTask::MajorRevision => LogicalAgent::Revision,
        ModelTask::MemoryExtraction
        | ModelTask::Summary
        | ModelTask::EntityExtraction
        | ModelTask::TimelineExtraction
        | ModelTask::Metadata => LogicalAgent::MemoryCurator,
        ModelTask::PlotAnalysis => LogicalAgent::PlotAnalyst,
        _ => LogicalAgent::Planner,
    }
}

fn writing_steps(quality: QualityMode) -> Vec<OrchestrationStep> {
    let mut steps = vec![
        step(
            "planner",
            LogicalAgent::Planner,
            ModelTask::ChapterPlanning,
            quality,
            &[],
        ),
        step(
            "writer",
            LogicalAgent::Writer,
            ModelTask::MainWriting,
            quality,
            &["planner"],
        ),
    ];
    match quality {
        QualityMode::Fast => steps.push(step(
            "memory",
            LogicalAgent::MemoryCurator,
            ModelTask::Summary,
            quality,
            &["writer"],
        )),
        QualityMode::Balanced => {
            steps.push(step(
                "continuity",
                LogicalAgent::ContinuityCritic,
                ModelTask::ContinuityCheck,
                quality,
                &["writer"],
            ));
            steps.push(step(
                "memory",
                LogicalAgent::MemoryCurator,
                ModelTask::MemoryExtraction,
                quality,
                &["continuity"],
            ));
        }
        QualityMode::Deep => {
            steps.push(step(
                "plot",
                LogicalAgent::PlotAnalyst,
                ModelTask::PlotAnalysis,
                quality,
                &["writer"],
            ));
            steps.push(step(
                "continuity",
                LogicalAgent::ContinuityCritic,
                ModelTask::ContinuityCheck,
                quality,
                &["plot"],
            ));
            steps.push(step(
                "style",
                LogicalAgent::StyleCritic,
                ModelTask::StyleCheck,
                quality,
                &["continuity"],
            ));
            steps.push(step(
                "revision",
                LogicalAgent::Revision,
                ModelTask::MajorRevision,
                quality,
                &["style"],
            ));
            steps.push(step(
                "memory",
                LogicalAgent::MemoryCurator,
                ModelTask::MemoryExtraction,
                quality,
                &["revision"],
            ));
        }
    }
    steps
}

fn revision_steps(quality: QualityMode) -> Vec<OrchestrationStep> {
    let mut steps = vec![step(
        "revision",
        LogicalAgent::Revision,
        ModelTask::MajorRevision,
        quality,
        &[],
    )];
    match quality {
        QualityMode::Fast => {}
        QualityMode::Balanced => {
            steps.push(step(
                "continuity",
                LogicalAgent::ContinuityCritic,
                ModelTask::ContinuityCheck,
                quality,
                &["revision"],
            ));
            steps.push(step(
                "memory",
                LogicalAgent::MemoryCurator,
                ModelTask::MemoryExtraction,
                quality,
                &["continuity"],
            ));
        }
        QualityMode::Deep => {
            steps.push(step(
                "plot",
                LogicalAgent::PlotAnalyst,
                ModelTask::PlotAnalysis,
                quality,
                &["revision"],
            ));
            steps.push(step(
                "style",
                LogicalAgent::StyleCritic,
                ModelTask::StyleCheck,
                quality,
                &["plot"],
            ));
            steps.push(step(
                "memory",
                LogicalAgent::MemoryCurator,
                ModelTask::MemoryExtraction,
                quality,
                &["style"],
            ));
        }
    }
    steps
}

fn step(
    id: &str,
    agent: LogicalAgent,
    task: ModelTask,
    quality: QualityMode,
    depends_on: &[&str],
) -> OrchestrationStep {
    OrchestrationStep {
        id: id.into(),
        agent,
        task,
        quality,
        depends_on: depends_on.iter().map(|value| (*value).into()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures::executor::block_on;

    use super::*;
    use crate::context::ContextSource;
    use crate::domain::character::{Character, CharacterState};
    use crate::domain::project::{Project, ProjectStatus};
    use crate::provider::{MockProvider, ModelProfile, ProviderDescriptor};

    struct Source {
        project: Project,
    }

    impl ContextSource for Source {
        fn load_project(&self, _project_id: &str) -> AppResult<Project> {
            Ok(self.project.clone())
        }

        fn load_character(&self, _project_id: &str, _character_id: &str) -> AppResult<Character> {
            Err(AppError::NotFound)
        }

        fn load_character_state(&self, _character_id: &str) -> AppResult<CharacterState> {
            Err(AppError::NotFound)
        }
    }

    fn profile() -> ModelProfile {
        ModelProfile {
            provider_id: "mock".into(),
            model_id: "writer".into(),
            display_name: "Writer".into(),
            context_window_tokens: 8192,
            default_output_tokens: 512,
            strengths: vec!["planning".into(), "prose".into(), "analysis".into()],
            weaknesses: Vec::new(),
            strategy: Vec::new(),
            tier: crate::provider::ModelTier::Medium,
            capabilities: ProviderCapabilities::default(),
        }
    }

    fn request(quality: QualityMode) -> OrchestrationRequest {
        OrchestrationRequest {
            project_id: "project-1".into(),
            task: ModelTask::MainWriting,
            quality,
            preferred_model: None,
            required_capabilities: ProviderCapabilities::default(),
            minimum_context_window_tokens: None,
            system_instructions: "Write within the established canon.".into(),
            character_ids: Vec::new(),
            include_character_states: false,
            working_memory: Vec::new(),
            context_budget: ContextBudget::default(),
            user_prompt: "Draft the next scene.".into(),
            temperature: Some(0.4),
        }
    }

    fn source() -> Source {
        Source {
            project: Project {
                id: "project-1".into(),
                name: "Test novel".into(),
                description: "A test premise".into(),
                status: ProjectStatus::Active,
                created_at: "2026-10-02T00:00:00Z".into(),
                updated_at: "2026-10-02T00:00:00Z".into(),
            },
        }
    }

    fn registry() -> (ProviderRegistry, Arc<MockProvider>) {
        let provider = Arc::new(MockProvider::new(
            ProviderDescriptor {
                id: "mock".into(),
                display_name: "Mock".into(),
            },
            vec![profile()],
        ));
        let registry = ProviderRegistry::new();
        registry.register(provider.clone()).unwrap();
        (registry, provider)
    }

    #[test]
    fn orchestration_future_can_run_on_native_async_runtime() {
        fn assert_send<T: Send>(_: T) {}

        let (registry, _) = registry();
        let compiler = ContextCompiler::default();
        let source = source();
        assert_send(NarrativeOrchestrator.run(
            &registry,
            &compiler,
            &source,
            request(QualityMode::Fast),
        ));
    }
    #[test]
    fn quality_modes_create_expected_writing_plan_shapes() {
        assert_eq!(
            OrchestrationPlan::for_request(ModelTask::MainWriting, QualityMode::Fast)
                .steps
                .len(),
            3
        );
        assert_eq!(
            OrchestrationPlan::for_request(ModelTask::MainWriting, QualityMode::Balanced)
                .steps
                .len(),
            4
        );
        assert_eq!(
            OrchestrationPlan::for_request(ModelTask::MainWriting, QualityMode::Deep)
                .steps
                .len(),
            7
        );
    }

    #[test]
    fn helper_tasks_use_one_task_specific_step() {
        let plan = OrchestrationPlan::for_request(ModelTask::Summary, QualityMode::Deep);
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].agent, LogicalAgent::MemoryCurator);
        assert!(plan.steps[0].depends_on.is_empty());
    }

    #[test]
    fn execution_routes_steps_and_hands_off_working_memory() {
        let (registry, provider) = registry();
        let result = block_on(NarrativeOrchestrator.run(
            &registry,
            &ContextCompiler::default(),
            &source(),
            request(QualityMode::Fast),
        ))
        .unwrap();
        assert_eq!(result.steps.len(), 3);
        assert_eq!(provider.generate_calls(), 3);
        assert!(result.steps[0]
            .context
            .blocks
            .iter()
            .all(|block| { block.kind != ContextBlockKind::WorkingMemory }));
        assert!(result.steps[1]
            .context
            .blocks
            .iter()
            .any(|block| block.kind == ContextBlockKind::WorkingMemory));
        assert_eq!(result.steps[0].route.model.model_id, "writer");
    }

    #[test]
    fn invalid_request_does_not_call_provider() {
        let (registry, provider) = registry();
        let mut request = request(QualityMode::Fast);
        request.user_prompt.clear();
        let error = block_on(NarrativeOrchestrator.run(
            &registry,
            &ContextCompiler::default(),
            &source(),
            request,
        ))
        .unwrap_err();
        assert!(matches!(error, AppError::Validation { .. }));
        assert_eq!(provider.generate_calls(), 0);
    }
}
