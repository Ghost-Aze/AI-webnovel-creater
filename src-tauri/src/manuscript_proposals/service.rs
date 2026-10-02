use crate::{
    domain::{
        manuscript_proposal::{build_proposal, CreateManuscriptProposalInput, ManuscriptProposal},
        project::ProjectStatus,
        revision::ProposalStatus,
    },
    error::AppResult,
    projects::repository::ProjectRepository,
};

use super::repository::ManuscriptProposalRepository;

#[derive(Clone)]
pub struct ManuscriptProposalService {
    repository: ManuscriptProposalRepository,
    projects: ProjectRepository,
}

impl ManuscriptProposalService {
    pub fn new(repository: ManuscriptProposalRepository, projects: ProjectRepository) -> Self {
        Self {
            repository,
            projects,
        }
    }

    pub fn create(
        &self,
        project_id: String,
        input: CreateManuscriptProposalInput,
    ) -> AppResult<ManuscriptProposal> {
        let project = self.projects.get(&project_id)?;
        if project.status == ProjectStatus::Archived {
            return Err(crate::error::AppError::ArchivedProject);
        }
        self.projects
            .chapter_belongs_to_project(&input.chapter_id, &project_id)?;
        self.projects.chapter_is_active(&input.chapter_id)?;
        self.repository.create(&build_proposal(project_id, input)?)
    }

    pub fn list(
        &self,
        chapter_id: &str,
        status: Option<ProposalStatus>,
    ) -> AppResult<Vec<ManuscriptProposal>> {
        self.repository.list(chapter_id, status)
    }

    pub fn get(&self, id: &str) -> AppResult<ManuscriptProposal> {
        self.repository.get(id)
    }

    pub fn promote(
        &self,
        id: &str,
        expected_revision: u64,
    ) -> AppResult<crate::domain::manuscript::Manuscript> {
        let proposal = self.repository.get(id)?;
        if self.projects.get(&proposal.project_id)?.status == ProjectStatus::Archived {
            return Err(crate::error::AppError::ArchivedProject);
        }
        self.projects
            .chapter_belongs_to_project(&proposal.chapter_id, &proposal.project_id)?;
        self.repository.promote(id, expected_revision)
    }

    pub fn reject(&self, id: &str) -> AppResult<ManuscriptProposal> {
        let proposal = self.repository.get(id)?;
        self.projects
            .chapter_belongs_to_project(&proposal.chapter_id, &proposal.project_id)?;
        self.repository.reject(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db,
        domain::{
            manuscript::{CreateChapterInput, ManuscriptContentFormat},
            manuscript_proposal::CreateManuscriptProposalInput,
            project::CreateProjectInput,
        },
        manuscripts::{repository::ManuscriptRepository, service::ManuscriptService},
        projects::service::ProjectService,
    };

    #[test]
    fn proposal_promotion_is_explicit_and_conflict_safe() {
        let connection = db::in_memory().unwrap();
        let projects_repo = ProjectRepository::new(connection.clone());
        let projects = ProjectService::new(projects_repo.clone());
        let manuscripts = ManuscriptService::new(
            ManuscriptRepository::new(connection.clone()),
            projects_repo.clone(),
        );
        let proposals = ManuscriptProposalService::new(
            ManuscriptProposalRepository::new(connection),
            projects_repo,
        );
        let project = projects
            .create(CreateProjectInput {
                name: "Proposal story".into(),
                description: None,
            })
            .unwrap();
        let chapter = manuscripts
            .create_chapter(
                project.id.clone(),
                CreateChapterInput {
                    number: 1,
                    title: "Opening".into(),
                    synopsis: None,
                },
            )
            .unwrap();
        let proposal = proposals
            .create(
                project.id,
                CreateManuscriptProposalInput {
                    chapter_id: chapter.id.clone(),
                    base_revision: 1,
                    proposed_content: "<p>AI draft</p>".into(),
                    content_format: ManuscriptContentFormat::Html,
                    rationale: Some("Tighten the opening".into()),
                    actor_type: None,
                    actor_id: None,
                },
            )
            .unwrap();
        assert_eq!(manuscripts.get_manuscript(&chapter.id).unwrap().revision, 1);
        let updated = proposals.promote(&proposal.id, 1).unwrap();
        assert_eq!(updated.revision, 2);
        assert_eq!(updated.content, "<p>AI draft</p>");
        assert_eq!(
            proposals.promote(&proposal.id, 2).unwrap_err(),
            crate::error::AppError::InvalidProposal
        );
        let rejected = proposals
            .create(
                chapter.project_id.clone(),
                CreateManuscriptProposalInput {
                    chapter_id: chapter.id.clone(),
                    base_revision: 2,
                    proposed_content: "<p>Another</p>".into(),
                    content_format: ManuscriptContentFormat::Html,
                    rationale: None,
                    actor_type: None,
                    actor_id: None,
                },
            )
            .unwrap();
        assert_eq!(
            proposals.reject(&rejected.id).unwrap().status,
            ProposalStatus::Rejected
        );
    }

    #[test]
    fn proposal_creation_rejects_an_archived_chapter() {
        let connection = db::in_memory().unwrap();
        let projects_repo = ProjectRepository::new(connection.clone());
        let projects = ProjectService::new(projects_repo.clone());
        let manuscripts = ManuscriptService::new(
            ManuscriptRepository::new(connection.clone()),
            projects_repo.clone(),
        );
        let proposals = ManuscriptProposalService::new(
            ManuscriptProposalRepository::new(connection),
            projects_repo,
        );
        let project = projects
            .create(CreateProjectInput {
                name: "Archived proposal chapter".into(),
                description: None,
            })
            .unwrap();
        let chapter = manuscripts
            .create_chapter(
                project.id.clone(),
                CreateChapterInput {
                    number: 1,
                    title: "Opening".into(),
                    synopsis: None,
                },
            )
            .unwrap();
        manuscripts
            .archive_chapter(&chapter.id, chapter.revision)
            .unwrap();

        assert_eq!(
            proposals.create(
                project.id,
                CreateManuscriptProposalInput {
                    chapter_id: chapter.id,
                    base_revision: 1,
                    proposed_content: "Archived draft".into(),
                    content_format: ManuscriptContentFormat::PlainText,
                    rationale: None,
                    actor_type: None,
                    actor_id: None,
                },
            ),
            Err(crate::error::AppError::ArchivedChapter)
        );
    }
}
