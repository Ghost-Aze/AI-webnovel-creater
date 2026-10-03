use webnovel_ai_studio_lib::{
    db,
    domain::{
        project::{CreateProjectInput, ProjectStatus},
        project_memory::{
            CanonRule, CreateCanonRuleInput, CreateStoryFactInput, ProjectMemoryListFilter,
            ProjectMemoryStatus, StoryFact, UpdateCanonRuleInput, UpdateStoryFactInput,
        },
        revision::CanonStatus,
    },
    error::AppError,
    project_memory::{repository::ProjectMemoryRepository, service::ProjectMemoryService},
    projects::{repository::ProjectRepository, service::ProjectService},
};

fn services() -> (ProjectService, ProjectMemoryService) {
    let connection = db::in_memory().expect("in-memory database should initialize");
    let project_repository = ProjectRepository::new(connection.clone());
    let projects = ProjectService::new(project_repository.clone());
    let memory = ProjectMemoryService::new(
        ProjectMemoryRepository::new(connection),
        project_repository,
    );
    (projects, memory)
}

#[test]
fn project_memory_crud_is_normalized_and_revision_safe() {
    let (projects, memory) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "Story".into(),
            description: None,
        })
        .unwrap();

    let fact = memory
        .create_story_fact(
            project.id.clone(),
            CreateStoryFactInput {
                title: "  Gate  ".into(),
                content: "  Opens at dawn. ".into(),
            },
        )
        .unwrap();
    assert_eq!(fact.title, "Gate");
    assert_eq!(fact.revision, 1);
    assert_eq!(fact.status, ProjectMemoryStatus::Active);
    assert_eq!(fact.canon_status, CanonStatus::Canon);

    let updated = memory
        .update_story_fact(
            &fact.id,
            UpdateStoryFactInput {
                title: "Gate of Dawn".into(),
                content: "The gate opens at dawn.".into(),
            },
            1,
        )
        .unwrap();
    assert_eq!(updated.revision, 2);
    assert_eq!(
        memory
            .update_story_fact(
                &fact.id,
                UpdateStoryFactInput {
                    title: "Stale".into(),
                    content: "Stale".into(),
                },
                1,
            )
            .unwrap_err(),
        AppError::Conflict
    );
}

#[test]
fn project_memory_archive_and_filters_are_explicit() {
    let (projects, memory) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "Story".into(),
            description: None,
        })
        .unwrap();
    let fact = memory
        .create_story_fact(
            project.id.clone(),
            CreateStoryFactInput {
                title: "Fact".into(),
                content: "Content".into(),
            },
        )
        .unwrap();

    let archived = memory.archive_story_fact(&fact.id, 1).unwrap();
    assert_eq!(archived.status, ProjectMemoryStatus::Archived);
    assert_eq!(memory.archive_story_fact(&fact.id, 1).unwrap(), archived);
    assert!(memory
        .list_story_facts(&project.id, ProjectMemoryListFilter::default())
        .unwrap()
        .is_empty());
    assert_eq!(
        memory
            .list_story_facts(
                &project.id,
                ProjectMemoryListFilter {
                    include_archived: true,
                },
            )
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        memory
            .update_story_fact(
                &fact.id,
                UpdateStoryFactInput {
                    title: "No".into(),
                    content: "No".into(),
                },
                2,
            )
            .unwrap_err(),
        AppError::ArchivedMemory
    );
}

#[test]
fn canon_rules_and_project_policies_are_typed() {
    let (projects, memory) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "Story".into(),
            description: None,
        })
        .unwrap();
    let rule = memory
        .create_canon_rule(
            project.id.clone(),
            CreateCanonRuleInput {
                title: "Magic".into(),
                rule: "It costs memory.".into(),
                scope: "world".into(),
            },
        )
        .unwrap();
    assert_eq!(rule.revision, 1);
    let updated = memory
        .update_canon_rule(
            &rule.id,
            UpdateCanonRuleInput {
                title: "Magic".into(),
                rule: "It costs a memory.".into(),
                scope: "world".into(),
            },
            1,
        )
        .unwrap();
    assert_eq!(updated.revision, 2);

    projects.archive(&project.id).unwrap();
    assert_eq!(projects.get(&project.id).unwrap().status, ProjectStatus::Archived);
    assert_eq!(
        memory
            .update_canon_rule(
                &rule.id,
                UpdateCanonRuleInput {
                    title: "No".into(),
                    rule: "No".into(),
                    scope: "world".into(),
                },
                2,
            )
            .unwrap_err(),
        AppError::ArchivedProject
    );
}

#[test]
fn project_memory_types_are_serializable_without_sql_details() {
    let _: Option<StoryFact> = None;
    let _: Option<CanonRule> = None;
    let serialized = serde_json::to_string(&AppError::ArchivedMemory).unwrap();
    assert_eq!(serialized, "{\"code\":\"archived_memory\"}");
    assert!(!serialized.contains("SELECT"));
}
