use crate::{
    domain::{
        manuscript::{
            build_chapter, normalize_label, normalize_title, Chapter, ChapterListFilter,
            ChapterStatus, CreateChapterInput, Manuscript, ManuscriptContentFormat,
            ManuscriptRevision, SaveManuscriptInput, UpdateChapterInput,
        },
        project::now_utc,
        revision::ActorType,
    },
    error::{AppError, AppResult},
    projects::repository::ProjectRepository,
};

use super::repository::{ChapterUpdateData, ManuscriptRepository, ManuscriptSaveData};

#[derive(Clone)]
pub struct ManuscriptService {
    repository: ManuscriptRepository,
    projects: ProjectRepository,
}

impl ManuscriptService {
    pub fn new(repository: ManuscriptRepository, projects: ProjectRepository) -> Self {
        Self {
            repository,
            projects,
        }
    }

    pub fn create_chapter(
        &self,
        project_id: String,
        input: CreateChapterInput,
    ) -> AppResult<Chapter> {
        self.projects.get(&project_id)?;
        self.repository.project_is_active(&project_id)?;
        self.repository
            .create_chapter(&build_chapter(project_id, input)?)
    }

    pub fn list_chapters(
        &self,
        project_id: &str,
        filter: ChapterListFilter,
    ) -> AppResult<Vec<Chapter>> {
        match self.repository.project_is_active(project_id) {
            Ok(()) | Err(AppError::ArchivedProject) => {
                self.repository.list_chapters(project_id, filter)
            }
            Err(error) => Err(error),
        }
    }

    pub fn get_chapter(&self, id: &str) -> AppResult<Chapter> {
        self.repository.get_chapter(id)
    }

    pub fn update_chapter(
        &self,
        id: &str,
        input: UpdateChapterInput,
        expected_revision: u64,
    ) -> AppResult<Chapter> {
        if input.number == 0 {
            return Err(AppError::Validation {
                message: "Chapter number must be greater than zero.".to_string(),
            });
        }
        let existing = self.repository.get_chapter(id)?;
        self.repository.project_is_active(&existing.project_id)?;
        if input.status == Some(ChapterStatus::Archived) {
            return self.archive_chapter(id, expected_revision);
        }
        let title = normalize_title(&input.title)?;
        let synopsis = input.synopsis.unwrap_or_default();
        let timestamp = now_utc();
        self.repository.update_chapter(
            id,
            ChapterUpdateData {
                number: input.number,
                title: &title,
                synopsis: synopsis.trim(),
                status: input.status.unwrap_or(existing.status),
                updated_at: &timestamp,
                expected_revision,
            },
        )
    }

    pub fn archive_chapter(&self, id: &str, expected_revision: u64) -> AppResult<Chapter> {
        let existing = self.repository.get_chapter(id)?;
        self.repository.project_is_active(&existing.project_id)?;
        self.repository
            .archive_chapter(id, &now_utc(), expected_revision)
    }

    pub fn get_manuscript(&self, chapter_id: &str) -> AppResult<Manuscript> {
        self.repository.get_chapter(chapter_id)?;
        self.repository.get_manuscript(chapter_id)
    }

    pub fn save_manuscript(
        &self,
        chapter_id: &str,
        input: SaveManuscriptInput,
    ) -> AppResult<Manuscript> {
        let chapter = self.repository.get_chapter(chapter_id)?;
        self.repository.project_is_active(&chapter.project_id)?;
        let label = normalize_label(input.label);
        let timestamp = now_utc();
        self.repository.save_manuscript(
            chapter_id,
            ManuscriptSaveData {
                content: &input.content,
                content_format: input
                    .content_format
                    .unwrap_or(ManuscriptContentFormat::PlainText),
                label: &label,
                actor_type: input.actor_type.unwrap_or(ActorType::User),
                actor_id: input.actor_id.as_deref(),
                expected_revision: input.expected_revision,
                updated_at: &timestamp,
            },
        )
    }

    pub fn list_revisions(&self, chapter_id: &str) -> AppResult<Vec<ManuscriptRevision>> {
        self.repository.get_chapter(chapter_id)?;
        self.repository.list_revisions(chapter_id)
    }

    pub fn restore_manuscript(
        &self,
        chapter_id: &str,
        revision: u64,
        expected_revision: u64,
    ) -> AppResult<Manuscript> {
        let snapshot = self.repository.get_revision(chapter_id, revision)?;
        self.save_manuscript(
            chapter_id,
            SaveManuscriptInput {
                content: snapshot.content,
                content_format: Some(snapshot.content_format),
                label: Some(format!("Restore revision {}", revision)),
                actor_type: Some(ActorType::User),
                actor_id: None,
                expected_revision,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db,
        domain::project::{CreateProjectInput, ProjectStatus},
        projects::service::ProjectService,
    };

    fn service() -> (ProjectService, ManuscriptService) {
        let connection = db::in_memory().unwrap();
        let projects = ProjectRepository::new(connection.clone());
        let project_service = ProjectService::new(projects.clone());
        (
            project_service,
            ManuscriptService::new(ManuscriptRepository::new(connection), projects),
        )
    }

    #[test]
    fn chapter_and_manuscript_lifecycle_is_revision_safe() {
        let (projects, service) = service();
        let project = projects
            .create(CreateProjectInput {
                name: "Manuscript".into(),
                description: None,
            })
            .unwrap();
        let chapter = service
            .create_chapter(
                project.id,
                CreateChapterInput {
                    number: 1,
                    title: "Opening".into(),
                    synopsis: Some("Arrival".into()),
                },
            )
            .unwrap();
        let initial = service.get_manuscript(&chapter.id).unwrap();
        assert_eq!(initial.revision, 1);
        let saved = service
            .save_manuscript(
                &chapter.id,
                SaveManuscriptInput {
                    content: "The gate opened.".into(),
                    content_format: Some(ManuscriptContentFormat::PlainText),
                    label: Some("Draft 2".into()),
                    actor_type: None,
                    actor_id: None,
                    expected_revision: 1,
                },
            )
            .unwrap();
        assert_eq!(saved.revision, 2);
        assert_eq!(service.list_revisions(&chapter.id).unwrap().len(), 2);
        assert_eq!(
            service
                .save_manuscript(
                    &chapter.id,
                    SaveManuscriptInput {
                        content: "stale".into(),
                        content_format: Some(ManuscriptContentFormat::PlainText),
                        label: None,
                        actor_type: None,
                        actor_id: None,
                        expected_revision: 1,
                    },
                )
                .unwrap_err(),
            AppError::Conflict
        );
        let restored = service.restore_manuscript(&chapter.id, 1, 2).unwrap();
        assert_eq!(restored.revision, 3);
        assert_eq!(restored.content, "");
    }

    #[test]
    fn archived_projects_and_duplicate_numbers_are_rejected() {
        let (projects, service) = service();
        let project = projects
            .create(CreateProjectInput {
                name: "Archive".into(),
                description: None,
            })
            .unwrap();
        service
            .create_chapter(
                project.id.clone(),
                CreateChapterInput {
                    number: 1,
                    title: "One".into(),
                    synopsis: None,
                },
            )
            .unwrap();
        assert_eq!(
            service
                .create_chapter(
                    project.id.clone(),
                    CreateChapterInput {
                        number: 1,
                        title: "Duplicate".into(),
                        synopsis: None,
                    },
                )
                .unwrap_err(),
            AppError::DuplicateChapterNumber
        );
        let archived = projects.archive(&project.id).unwrap();
        assert_eq!(archived.status, ProjectStatus::Archived);
        assert!(matches!(
            service.create_chapter(
                project.id,
                CreateChapterInput {
                    number: 2,
                    title: "Nope".into(),
                    synopsis: None,
                },
            ),
            Err(AppError::ArchivedProject)
        ));
    }
}
