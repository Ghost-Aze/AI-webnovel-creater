use serde::{Deserialize, Serialize};

use super::{
    project::{new_id, now_utc},
    revision::ActorType,
};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManuscriptContentFormat {
    PlainText,
    Html,
}

impl ManuscriptContentFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PlainText => "plain_text",
            Self::Html => "html",
        }
    }
}

impl TryFrom<&str> for ManuscriptContentFormat {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "plain_text" => Ok(Self::PlainText),
            "html" => Ok(Self::Html),
            _ => Err(AppError::InvalidStatus),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChapterStatus {
    Draft,
    Final,
    Archived,
}

impl ChapterStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Final => "final",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for ChapterStatus {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "draft" => Ok(Self::Draft),
            "final" => Ok(Self::Final),
            "archived" => Ok(Self::Archived),
            _ => Err(AppError::InvalidStatus),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub project_id: String,
    pub number: u32,
    pub title: String,
    pub synopsis: String,
    pub status: ChapterStatus,
    pub revision: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateChapterInput {
    pub number: u32,
    pub title: String,
    #[serde(default)]
    pub synopsis: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateChapterInput {
    pub number: u32,
    pub title: String,
    #[serde(default)]
    pub synopsis: Option<String>,
    #[serde(default)]
    pub status: Option<ChapterStatus>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct ChapterListFilter {
    #[serde(default)]
    pub include_archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manuscript {
    pub id: String,
    pub chapter_id: String,
    pub content: String,
    pub content_format: ManuscriptContentFormat,
    pub revision: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManuscriptRevision {
    pub id: String,
    pub manuscript_id: String,
    pub revision: u64,
    pub content: String,
    pub content_format: ManuscriptContentFormat,
    pub label: String,
    pub actor_type: ActorType,
    pub actor_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SaveManuscriptInput {
    pub content: String,
    #[serde(default)]
    pub content_format: Option<ManuscriptContentFormat>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub actor_type: Option<ActorType>,
    #[serde(default)]
    pub actor_id: Option<String>,
    pub expected_revision: u64,
}

pub fn build_chapter(project_id: String, input: CreateChapterInput) -> AppResult<Chapter> {
    if input.number == 0 {
        return Err(AppError::Validation {
            message: "Chapter number must be greater than zero.".to_string(),
        });
    }
    let title = normalize_title(&input.title)?;
    let timestamp = now_utc();
    Ok(Chapter {
        id: new_id(),
        project_id,
        number: input.number,
        title,
        synopsis: normalize_optional(input.synopsis),
        status: ChapterStatus::Draft,
        revision: 1,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    })
}

pub fn normalize_title(title: &str) -> AppResult<String> {
    let value = title.trim();
    if value.is_empty() {
        return Err(AppError::Validation {
            message: "Chapter title cannot be empty.".to_string(),
        });
    }
    Ok(value.to_string())
}

pub fn normalize_optional(value: Option<String>) -> String {
    value.unwrap_or_default().trim().to_string()
}

pub fn normalize_label(value: Option<String>) -> String {
    let value = normalize_optional(value);
    if value.is_empty() {
        "Draft".to_string()
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_input_is_normalized() {
        let chapter = build_chapter(
            "project".to_string(),
            CreateChapterInput {
                number: 2,
                title: "  The Crossing  ".to_string(),
                synopsis: Some("  A difficult choice. ".to_string()),
            },
        )
        .unwrap();
        assert_eq!(chapter.title, "The Crossing");
        assert_eq!(chapter.synopsis, "A difficult choice.");
        assert_eq!(chapter.revision, 1);
    }

    #[test]
    fn chapter_input_rejects_zero_and_blank_titles() {
        assert!(matches!(
            build_chapter(
                "project".to_string(),
                CreateChapterInput {
                    number: 0,
                    title: "Chapter".to_string(),
                    synopsis: None,
                },
            ),
            Err(AppError::Validation { .. })
        ));
        assert!(normalize_title("  ").is_err());
    }

    #[test]
    fn empty_revision_labels_get_a_safe_default() {
        assert_eq!(normalize_label(None), "Draft");
        assert_eq!(normalize_label(Some("  ".to_string())), "Draft");
        assert_eq!(
            normalize_label(Some("Final pass".to_string())),
            "Final pass"
        );
    }

    #[test]
    fn manuscript_content_formats_use_stable_wire_names() {
        assert_eq!(
            serde_json::to_string(&ManuscriptContentFormat::PlainText).unwrap(),
            "\"plain_text\""
        );
        assert_eq!(
            ManuscriptContentFormat::try_from("html").unwrap(),
            ManuscriptContentFormat::Html
        );
    }
}
