use webnovel_ai_studio_lib::{
    db,
    domain::{
        manuscript::{ChapterListFilter, CreateChapterInput, SaveManuscriptInput},
        project::CreateProjectInput,
    },
    error::AppError,
    manuscripts::{repository::ManuscriptRepository, service::ManuscriptService},
    projects::{repository::ProjectRepository, service::ProjectService},
};

fn services() -> (ProjectService, ManuscriptService) {
    let connection = db::in_memory().expect("database should initialize");
    let projects = ProjectRepository::new(connection.clone());
    (
        ProjectService::new(projects.clone()),
        ManuscriptService::new(ManuscriptRepository::new(connection), projects),
    )
}

#[test]
fn chapter_and_manuscript_records_are_separate_from_chat() {
    let (projects, manuscripts) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "A story".to_string(),
            description: None,
        })
        .unwrap();
    let chapter = manuscripts
        .create_chapter(
            project.id.clone(),
            CreateChapterInput {
                number: 1,
                title: "Opening".to_string(),
                synopsis: Some("A door opens.".to_string()),
            },
        )
        .unwrap();
    let document = manuscripts.get_manuscript(&chapter.id).unwrap();
    assert_eq!(document.content, "");
    assert_eq!(
        manuscripts
            .list_chapters(&project.id, ChapterListFilter::default())
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn stale_manuscript_saves_are_rejected_without_losing_history() {
    let (projects, manuscripts) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "A story".to_string(),
            description: None,
        })
        .unwrap();
    let chapter = manuscripts
        .create_chapter(
            project.id,
            CreateChapterInput {
                number: 1,
                title: "Opening".to_string(),
                synopsis: None,
            },
        )
        .unwrap();
    manuscripts
        .save_manuscript(
            &chapter.id,
            SaveManuscriptInput {
                content: "First draft".to_string(),
                label: Some("Draft 2".to_string()),
                actor_type: None,
                actor_id: None,
                expected_revision: 1,
            },
        )
        .unwrap();
    assert_eq!(
        manuscripts
            .save_manuscript(
                &chapter.id,
                SaveManuscriptInput {
                    content: "stale".to_string(),
                    label: None,
                    actor_type: None,
                    actor_id: None,
                    expected_revision: 1,
                },
            )
            .unwrap_err(),
        AppError::Conflict
    );
    assert_eq!(manuscripts.list_revisions(&chapter.id).unwrap().len(), 2);
}
