use serde::{Deserialize, Serialize};

use crate::LearningEvent;

pub const LEARNING_EVIDENCE_VERSION: u32 = 1;

/// Stable Reading transition semantics shared by M1 read tools and M2
/// observation projection. Keep this pure: it is part of the replay contract.
pub fn question_transition_state(previous: Option<bool>, current: Option<bool>) -> &'static str {
    match (previous, current) {
        (None, _) => "first_observation",
        (Some(false), Some(true)) => "corrected",
        (Some(true), Some(false)) => "newly_wrong",
        (Some(false), Some(false)) => "still_wrong",
        (Some(true), Some(true)) => "still_correct",
        _ => "unscored",
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunKind {
    #[default]
    Workspace,
    AttemptReview,
}

impl AgentRunKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::AttemptReview => "attempt_review",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptEvidenceView {
    pub attempt: AttemptEvidenceSummary,
    pub questions: Vec<QuestionEvidence>,
    pub score: AttemptEvidenceScore,
    pub timeline_summary: TimelineSummary,
    pub evidence_version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptEvidenceSummary {
    pub attempt_id: String,
    pub asset_id: String,
    pub mode: String,
    pub started_at: String,
    pub completed_at: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptEvidenceScore {
    pub score_value: Option<f64>,
    pub correct_count: Option<f64>,
    pub question_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineSummary {
    pub answered_count: u32,
    pub marked_count: u32,
    pub change_count: u32,
    pub visit_count: u32,
    pub question_elapsed_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionEvidence {
    pub question_id: String,
    pub is_correct: Option<bool>,
    pub question_kind: Option<String>,
    pub change_count: u32,
    pub visit_count: u32,
    pub elapsed_ms: u64,
    pub marked: bool,
    pub first_try_correct: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareAttemptsQuery {
    pub asset_id: String,
    #[serde(default = "default_compare_limit")]
    pub limit: u32,
    #[serde(default)]
    pub minimum_gap_hours: u32,
}

fn default_compare_limit() -> u32 {
    5
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptComparison {
    pub asset_id: String,
    pub attempts: Vec<AttemptTimelinePoint>,
    pub question_transitions: Vec<QuestionTransition>,
    pub repeat_familiarity_warning: bool,
    pub minimum_gap_hours: u32,
    pub evidence_version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptTimelinePoint {
    pub attempt_id: String,
    pub ordinal: u32,
    pub completed_at: String,
    pub gap_hours: Option<f64>,
    pub score_value: Option<f64>,
    pub correct_count: Option<f64>,
    pub question_count: Option<u32>,
    pub duration_ms: u64,
    pub change_count: u32,
    pub visit_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionTransition {
    pub question_id: String,
    pub attempt_id: String,
    pub previous_attempt_id: Option<String>,
    pub state: String,
    pub first_try_correct: Option<bool>,
    pub change_count: u32,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct QuestionHistoryQuery {
    pub asset_id: String,
    pub question_id: String,
    #[serde(default = "default_question_limit")]
    pub limit: u32,
}

fn default_question_limit() -> u32 {
    10
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionHistory {
    pub asset_id: String,
    pub question_id: String,
    pub observations: Vec<QuestionHistoryObservation>,
    pub evidence_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionHistoryObservation {
    pub attempt_id: String,
    pub completed_at: String,
    pub is_correct: Option<bool>,
    pub change_count: u32,
    pub visit_count: u32,
    pub elapsed_ms: u64,
    pub marked: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SearchLearningEventsQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurred_after: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurred_before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<String>,
    #[serde(default = "default_event_limit")]
    pub limit: u32,
}

fn default_event_limit() -> u32 {
    50
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningEventSearchResult {
    pub events: Vec<LearningEvent>,
    pub truncated: bool,
    pub evidence_version: u32,
}
