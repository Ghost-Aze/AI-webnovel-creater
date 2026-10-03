use serde::{Deserialize, Serialize};

use super::{
    ModelProfile, ModelRef, ModelTier, ProviderCapabilities, ProviderError, ProviderRegistry,
    ProviderResult,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityMode {
    Fast,
    #[default]
    Balanced,
    Deep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTask {
    StoryArchitecture,
    ArcPlanning,
    ChapterPlanning,
    ScenePlanning,
    DeveloperChat,
    MainWriting,
    CharacterPsychology,
    MajorRevision,
    PlotAnalysis,
    StyleCheck,
    MemoryExtraction,
    Summary,
    EntityExtraction,
    TimelineExtraction,
    Metadata,
    RetrievalQuery,
    ContinuityCheck,
}

impl ModelTask {
    fn affinity_tags(self) -> &'static [&'static str] {
        match self {
            Self::StoryArchitecture => &["architecture", "plot", "planning", "world"],
            Self::ArcPlanning => &["arc", "planning", "plot"],
            Self::ChapterPlanning => &["chapter", "planning", "plot"],
            Self::ScenePlanning => &["scene", "planning", "structure"],
            Self::DeveloperChat => &["chat", "writing", "dialogue", "analysis"],
            Self::MainWriting => &["prose", "writing", "dialogue"],
            Self::CharacterPsychology => &["character", "psychology", "dialogue"],
            Self::MajorRevision => &["revision", "prose", "writing"],
            Self::PlotAnalysis => &["plot", "analysis", "planning", "architecture"],
            Self::StyleCheck => &["style", "prose", "writing", "revision"],
            Self::MemoryExtraction => &["memory", "extraction", "summary"],
            Self::Summary => &["summary", "summarization", "compression"],
            Self::EntityExtraction => &["entity", "extraction", "metadata"],
            Self::TimelineExtraction => &["timeline", "extraction", "continuity"],
            Self::Metadata => &["metadata", "classification", "tagging"],
            Self::RetrievalQuery => &["retrieval", "query", "search"],
            Self::ContinuityCheck => &["continuity", "analysis", "checking"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteSelectionReason {
    Preferred,
    Policy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingRequest {
    pub task: ModelTask,
    pub quality: QualityMode,
    pub preferred_model: Option<ModelRef>,
    pub required_capabilities: ProviderCapabilities,
    pub minimum_context_window_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteDecision {
    pub model: ModelRef,
    pub profile: ModelProfile,
    pub quality: QualityMode,
    pub reason: RouteSelectionReason,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ModelRouter;

impl ModelRouter {
    pub fn route(
        &self,
        registry: &ProviderRegistry,
        request: RoutingRequest,
    ) -> ProviderResult<RouteDecision> {
        self.route_profiles(registry.list_models(None)?, request)
    }

    pub fn route_profiles(
        &self,
        profiles: Vec<ModelProfile>,
        request: RoutingRequest,
    ) -> ProviderResult<RouteDecision> {
        for profile in &profiles {
            profile.validate()?;
        }

        if let Some(preferred) = &request.preferred_model {
            let profile = profiles
                .iter()
                .find(|profile| {
                    profile.provider_id == preferred.provider_id
                        && profile.model_id == preferred.model_id
                })
                .ok_or(ProviderError::ModelNotFound)?;
            if !eligible(profile, &request) {
                return Err(ProviderError::NoSuitableModel);
            }
            return Ok(decision(
                profile.clone(),
                request.quality,
                RouteSelectionReason::Preferred,
            ));
        }

        let mut candidates = profiles
            .into_iter()
            .filter(|profile| eligible(profile, &request))
            .map(|profile| {
                let score = score(&profile, request.task, request.quality);
                (score, profile)
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|(left_score, left), (right_score, right)| {
            right_score
                .cmp(left_score)
                .then_with(|| left.provider_id.cmp(&right.provider_id))
                .then_with(|| left.model_id.cmp(&right.model_id))
        });
        let (_, profile) = candidates
            .into_iter()
            .next()
            .ok_or(ProviderError::NoSuitableModel)?;
        Ok(decision(
            profile,
            request.quality,
            RouteSelectionReason::Policy,
        ))
    }
}

fn decision(
    profile: ModelProfile,
    quality: QualityMode,
    reason: RouteSelectionReason,
) -> RouteDecision {
    RouteDecision {
        model: ModelRef {
            provider_id: profile.provider_id.clone(),
            model_id: profile.model_id.clone(),
        },
        profile,
        quality,
        reason,
    }
}

fn eligible(profile: &ModelProfile, request: &RoutingRequest) -> bool {
    capabilities_match(request.required_capabilities, profile.capabilities)
        && request
            .minimum_context_window_tokens
            .is_none_or(|minimum| profile.context_window_tokens >= minimum)
}

fn capabilities_match(required: ProviderCapabilities, available: ProviderCapabilities) -> bool {
    (!required.streaming || available.streaming)
        && (!required.embeddings || available.embeddings)
        && (!required.tools || available.tools)
        && (!required.vision || available.vision)
        && (!required.structured_output || available.structured_output)
        && (!required.prompt_caching || available.prompt_caching)
}

fn score(profile: &ModelProfile, task: ModelTask, quality: QualityMode) -> i32 {
    let tags = task.affinity_tags();
    let strengths = profile
        .strengths
        .iter()
        .filter(|value| tags.iter().any(|tag| tag_matches(value, tag)))
        .count() as i32;
    let weaknesses = profile
        .weaknesses
        .iter()
        .filter(|value| tags.iter().any(|tag| tag_matches(value, tag)))
        .count() as i32;
    let strategies = profile
        .strategy
        .iter()
        .filter(|value| tags.iter().any(|tag| tag_matches(value, tag)))
        .count() as i32;
    strengths * 5 - weaknesses * 4 + strategies * 2 + tier_score(profile.tier, quality)
}

fn tag_matches(value: &str, tag: &str) -> bool {
    let value = value.to_ascii_lowercase();
    let tag = tag.to_ascii_lowercase();
    value == tag || value.contains(&tag) || tag.contains(&value)
}

fn tier_score(tier: ModelTier, quality: QualityMode) -> i32 {
    match (quality, tier) {
        (QualityMode::Fast, ModelTier::Local) => 7,
        (QualityMode::Fast, ModelTier::Small) => 6,
        (QualityMode::Fast, ModelTier::Medium) => 3,
        (QualityMode::Fast, ModelTier::Large) => 1,
        (QualityMode::Balanced, ModelTier::Local) => 2,
        (QualityMode::Balanced, ModelTier::Small) => 4,
        (QualityMode::Balanced, ModelTier::Medium) => 7,
        (QualityMode::Balanced, ModelTier::Large) => 6,
        (QualityMode::Deep, ModelTier::Local) => 0,
        (QualityMode::Deep, ModelTier::Small) => 2,
        (QualityMode::Deep, ModelTier::Medium) => 7,
        (QualityMode::Deep, ModelTier::Large) => 9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(
        provider_id: &str,
        model_id: &str,
        tier: ModelTier,
        strengths: &[&str],
    ) -> ModelProfile {
        ModelProfile {
            provider_id: provider_id.into(),
            model_id: model_id.into(),
            display_name: model_id.into(),
            context_window_tokens: 8192,
            default_output_tokens: 1024,
            strengths: strengths.iter().map(|value| (*value).into()).collect(),
            weaknesses: Vec::new(),
            strategy: Vec::new(),
            tier,
            capabilities: ProviderCapabilities::default(),
        }
    }

    fn request(task: ModelTask, quality: QualityMode) -> RoutingRequest {
        RoutingRequest {
            task,
            quality,
            preferred_model: None,
            required_capabilities: ProviderCapabilities::default(),
            minimum_context_window_tokens: None,
        }
    }

    #[test]
    fn fast_mode_prefers_local_helper_models() {
        let router = ModelRouter;
        let result = router
            .route_profiles(
                vec![
                    profile("cloud", "large-writer", ModelTier::Large, &["prose"]),
                    profile(
                        "local",
                        "small-extractor",
                        ModelTier::Local,
                        &["extraction"],
                    ),
                ],
                request(ModelTask::MemoryExtraction, QualityMode::Fast),
            )
            .unwrap();
        assert_eq!(result.model.provider_id, "local");
        assert_eq!(result.model.model_id, "small-extractor");
    }

    #[test]
    fn deep_mode_prefers_large_models_for_writing() {
        let router = ModelRouter;
        let result = router
            .route_profiles(
                vec![
                    profile("local", "local-writer", ModelTier::Local, &["prose"]),
                    profile("cloud", "large-writer", ModelTier::Large, &["prose"]),
                ],
                request(ModelTask::MainWriting, QualityMode::Deep),
            )
            .unwrap();
        assert_eq!(result.model.provider_id, "cloud");
    }

    #[test]
    fn task_affinity_can_outweigh_tier_preference() {
        let router = ModelRouter;
        let result = router
            .route_profiles(
                vec![
                    profile(
                        "cloud",
                        "planner",
                        ModelTier::Large,
                        &["planning", "architecture"],
                    ),
                    profile("local", "writer", ModelTier::Local, &["prose"]),
                ],
                request(ModelTask::StoryArchitecture, QualityMode::Fast),
            )
            .unwrap();
        assert_eq!(result.model.model_id, "planner");
    }

    #[test]
    fn preferred_model_wins_when_eligible() {
        let router = ModelRouter;
        let mut request = request(ModelTask::MainWriting, QualityMode::Deep);
        request.preferred_model = Some(ModelRef {
            provider_id: "local".into(),
            model_id: "local-writer".into(),
        });
        let result = router
            .route_profiles(
                vec![
                    profile("cloud", "large-writer", ModelTier::Large, &["prose"]),
                    profile("local", "local-writer", ModelTier::Local, &["prose"]),
                ],
                request,
            )
            .unwrap();
        assert_eq!(result.reason, RouteSelectionReason::Preferred);
        assert_eq!(result.model.provider_id, "local");
    }

    #[test]
    fn capabilities_and_context_window_filter_candidates() {
        let router = ModelRouter;
        let mut request = request(ModelTask::Summary, QualityMode::Balanced);
        request.required_capabilities.structured_output = true;
        request.minimum_context_window_tokens = Some(16_000);
        let mut profile = profile("cloud", "summary", ModelTier::Medium, &["summary"]);
        profile.capabilities.structured_output = true;
        profile.context_window_tokens = 32_000;
        let result = router.route_profiles(vec![profile], request).unwrap();
        assert_eq!(result.model.model_id, "summary");
    }

    #[test]
    fn ties_are_stable_by_provider_and_model_id() {
        let router = ModelRouter;
        let result = router
            .route_profiles(
                vec![
                    profile("zeta", "same", ModelTier::Medium, &[]),
                    profile("alpha", "same", ModelTier::Medium, &[]),
                ],
                request(ModelTask::Metadata, QualityMode::Balanced),
            )
            .unwrap();
        assert_eq!(result.model.provider_id, "alpha");
    }

    #[test]
    fn no_candidate_returns_safe_route_error() {
        let router = ModelRouter;
        let mut request = request(ModelTask::ContinuityCheck, QualityMode::Deep);
        request.required_capabilities.tools = true;
        assert_eq!(
            router.route_profiles(
                vec![profile("cloud", "writer", ModelTier::Large, &[])],
                request,
            ),
            Err(ProviderError::NoSuitableModel)
        );
    }
}
