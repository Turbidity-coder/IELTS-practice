use async_trait::async_trait;
use ielts_application::{
    AgentToolCall, AgentToolDefinition, AgentToolExecution, AgentToolExecutor,
};
use ielts_domain::{CompareAttemptsQuery, QuestionHistoryQuery, SearchLearningEventsQuery};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::app::state::AppDb;

const MAX_MODEL_RESULT_BYTES: usize = 64 * 1024;

pub(crate) struct LearningReadTools<'a> {
    db: &'a AppDb,
}

impl<'a> LearningReadTools<'a> {
    pub(crate) fn new(db: &'a AppDb) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AgentToolExecutor for LearningReadTools<'_> {
    fn definitions(&self) -> Vec<AgentToolDefinition> {
        vec![
            AgentToolDefinition {
                name: "get_attempt_detail".into(),
                description: "Read a compact canonical evidence view for one completed Reading attempt. Raw answers and passage content are excluded.".into(),
                parameters: object_schema(json!({
                    "attemptId": {"type":"string","minLength":1,"maxLength":256}
                }), &["attemptId"]),
            },
            AgentToolDefinition {
                name: "compare_attempts_for_asset".into(),
                description: "Compare up to ten completed attempts for one Reading asset using deterministic score, timing, and question-transition metrics.".into(),
                parameters: object_schema(json!({
                    "assetId": {"type":"string","minLength":1,"maxLength":256},
                    "limit": {"type":"integer","minimum":1,"maximum":10,"default":5},
                    "minimumGapHours": {"type":"integer","minimum":0,"maximum":8760,"default":12}
                }), &["assetId"]),
            },
            AgentToolDefinition {
                name: "get_question_history".into(),
                description: "Read bounded canonical outcome and timeline evidence for one question across attempts of one Reading asset.".into(),
                parameters: object_schema(json!({
                    "assetId": {"type":"string","minLength":1,"maxLength":256},
                    "questionId": {"type":"string","minLength":1,"maxLength":256},
                    "limit": {"type":"integer","minimum":1,"maximum":50,"default":10}
                }), &["assetId", "questionId"]),
            },
            AgentToolDefinition {
                name: "search_learning_events".into(),
                description: "Search the append-only learning evidence ledger using bounded structured filters. This tool is read-only.".into(),
                parameters: object_schema(json!({
                    "eventType": {"type":"string","maxLength":128},
                    "skillKey": {"type":"string","maxLength":256},
                    "activity": {"type":"string","enum":["reading","writing"]},
                    "occurredAfter": {"type":"string","maxLength":64},
                    "occurredBefore": {"type":"string","maxLength":64},
                    "assetId": {"type":"string","maxLength":256},
                    "attemptId": {"type":"string","maxLength":256},
                    "limit": {"type":"integer","minimum":1,"maximum":100,"default":50}
                }), &[]),
            },
        ]
    }

    fn audit_arguments(&self, call: &AgentToolCall) -> Value {
        match call.name.as_str() {
            "get_attempt_detail" => parse::<AttemptDetailArgs>(&call.arguments_json)
                .map(|args| json!({"attemptId":args.attempt_id,"valid":true}))
                .unwrap_or_else(|_| json!({"valid":false})),
            "compare_attempts_for_asset" => parse::<CompareAttemptsQuery>(&call.arguments_json)
                .map(|args| json!({"assetId":args.asset_id,"limit":args.limit,"minimumGapHours":args.minimum_gap_hours,"valid":true}))
                .unwrap_or_else(|_| json!({"valid":false})),
            "get_question_history" => parse::<QuestionHistoryQuery>(&call.arguments_json)
                .map(|args| json!({"assetId":args.asset_id,"questionId":args.question_id,"limit":args.limit,"valid":true}))
                .unwrap_or_else(|_| json!({"valid":false})),
            "search_learning_events" => parse::<SearchLearningEventsQuery>(&call.arguments_json)
                .map(|args| json!({"eventType":args.event_type,"activity":args.activity,"assetId":args.asset_id,"attemptId":args.attempt_id,"limit":args.limit,"valid":true}))
                .unwrap_or_else(|_| json!({"valid":false})),
            _ => json!({"known":false}),
        }
    }

    async fn execute(&self, call: &AgentToolCall) -> AgentToolExecution {
        match call.name.as_str() {
            "get_attempt_detail" => {
                let args: AttemptDetailArgs = match parse_or_reject(&call.arguments_json) {
                    Ok(args) => args,
                    Err(result) => return result,
                };
                let result = self
                    .db
                    .with_conn(|conn| ielts_db::get_attempt_evidence(conn, &args.attempt_id));
                encode_result(
                    "get_attempt_detail",
                    result,
                    |value| json!({"attemptId":value.attempt.attempt_id,"questionCount":value.questions.len()}),
                )
            }
            "compare_attempts_for_asset" => {
                let args: CompareAttemptsQuery = match parse_or_reject(&call.arguments_json) {
                    Ok(args) => args,
                    Err(result) => return result,
                };
                let result = self
                    .db
                    .with_conn(|conn| ielts_db::compare_attempts_for_asset(conn, &args));
                encode_result(
                    "compare_attempts_for_asset",
                    result,
                    |value| json!({"assetId":value.asset_id,"attemptCount":value.attempts.len(),"transitionCount":value.question_transitions.len()}),
                )
            }
            "get_question_history" => {
                let args: QuestionHistoryQuery = match parse_or_reject(&call.arguments_json) {
                    Ok(args) => args,
                    Err(result) => return result,
                };
                let result = self
                    .db
                    .with_conn(|conn| ielts_db::get_question_history(conn, &args));
                encode_result(
                    "get_question_history",
                    result,
                    |value| json!({"assetId":value.asset_id,"questionId":value.question_id,"observationCount":value.observations.len()}),
                )
            }
            "search_learning_events" => {
                let args: SearchLearningEventsQuery = match parse_or_reject(&call.arguments_json) {
                    Ok(args) => args,
                    Err(result) => return result,
                };
                let result = self
                    .db
                    .with_conn(|conn| ielts_db::search_learning_events(conn, &args));
                encode_result(
                    "search_learning_events",
                    result,
                    |value| json!({"eventCount":value.events.len(),"truncated":value.truncated}),
                )
            }
            _ => AgentToolExecution::rejected(
                "agent.unknown_tool",
                format!("unknown AttemptReview tool: {}", call.name),
                json!({"known":false}),
            ),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AttemptDetailArgs {
    attempt_id: String,
}

fn object_schema(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

fn parse<T: DeserializeOwned>(raw: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(raw)
}

fn parse_or_reject<T: DeserializeOwned>(raw: &str) -> Result<T, AgentToolExecution> {
    parse(raw).map_err(|error| {
        AgentToolExecution::rejected(
            "agent.invalid_tool_arguments",
            format!("tool arguments are invalid: {error}"),
            json!({"valid":false}),
        )
    })
}

fn encode_result<T: serde::Serialize>(
    tool_name: &str,
    result: ielts_db::DbResult<T>,
    audit: impl FnOnce(&T) -> Value,
) -> AgentToolExecution {
    let value = match result {
        Ok(value) => value,
        Err(error) => {
            return AgentToolExecution::failed(
                "agent.learning_read_failed",
                error.to_string(),
                false,
                json!({"tool":tool_name}),
            )
        }
    };
    let model_content = match serde_json::to_string(&value) {
        Ok(content) => content,
        Err(error) => {
            return AgentToolExecution::failed(
                "agent.tool_serialization_failed",
                error.to_string(),
                false,
                json!({"tool":tool_name}),
            )
        }
    };
    if model_content.len() > MAX_MODEL_RESULT_BYTES {
        return AgentToolExecution::rejected(
            "agent.tool_output_too_large",
            format!("tool output exceeds the {MAX_MODEL_RESULT_BYTES} byte limit"),
            json!({"tool":tool_name,"bytes":model_content.len(),"maxBytes":MAX_MODEL_RESULT_BYTES}),
        );
    }
    let mut audit_payload = audit(&value);
    if let Some(object) = audit_payload.as_object_mut() {
        object.insert("tool".into(), Value::String(tool_name.into()));
        object.insert("bytes".into(), json!(model_content.len()));
    }
    AgentToolExecution::succeeded(model_content, audit_payload)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use ielts_application::AgentToolStatus;
    use ielts_db::{
        append_learning_event, migrate, open_connection, DbOpenOptions, NewLearningEvent,
    };
    use ielts_domain::LearningEventType;

    use super::*;

    fn tools() -> (tempfile::TempDir, AppDb) {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("learning-tools.db");
        let mut connection = open_connection(&DbOpenOptions::create(path.clone())).unwrap();
        migrate(&mut connection).unwrap();
        (directory, AppDb::from_test_connection(connection, path))
    }

    fn call(name: &str, arguments: Value) -> AgentToolCall {
        AgentToolCall {
            id: "call-1".into(),
            name: name.into(),
            arguments_json: arguments.to_string(),
        }
    }

    #[test]
    fn registry_contains_only_four_learning_reads() {
        let (_directory, db) = tools();
        let executor = LearningReadTools::new(&db);
        let names = executor
            .definitions()
            .into_iter()
            .map(|definition| definition.name)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            names,
            BTreeSet::from([
                "compare_attempts_for_asset".into(),
                "get_attempt_detail".into(),
                "get_question_history".into(),
                "search_learning_events".into(),
            ])
        );
        assert!(!names.contains("write_file"));
        assert!(!names.contains("replace_in_file"));
    }

    #[tokio::test]
    async fn rejects_mutation_tools_and_unknown_arguments() {
        let (_directory, db) = tools();
        let executor = LearningReadTools::new(&db);
        let mutation = executor
            .execute(&call("write_file", json!({"path":"x","content":"y"})))
            .await;
        assert_eq!(mutation.status, AgentToolStatus::Rejected);
        assert_eq!(mutation.error.unwrap().code, "agent.unknown_tool");

        let invalid = executor
            .execute(&call(
                "get_attempt_detail",
                json!({"attemptId":"a1","extra":true}),
            ))
            .await;
        assert_eq!(invalid.status, AgentToolStatus::Rejected);
        assert_eq!(invalid.error.unwrap().code, "agent.invalid_tool_arguments");
    }

    #[tokio::test]
    async fn attempt_detail_excludes_raw_answers_and_audit_payloads_exclude_event_content() {
        let (_directory, db) = tools();
        db.with_conn(|conn| {
            conn.execute_batch(
                "INSERT INTO practice_assets(id,activity,source_kind,title,schema_version,fingerprint,pdf_only,created_at,updated_at)
                 VALUES ('asset-1','reading','imported','A',1,'fp',0,'2026-08-12T00:00:00Z','2026-08-12T00:00:00Z');
                 INSERT INTO attempts(id,activity,asset_id,mode,status,started_at,submitted_at,completed_at,duration_ms,score_value,score_scale,correct_count,question_count,schema_version,created_at,updated_at)
                 VALUES ('attempt-1','reading','asset-1','single','completed','2026-08-12T00:00:00Z','2026-08-12T00:01:00Z','2026-08-12T00:01:00Z',60000,1.0,'ratio',1,1,2,'2026-08-12T00:00:00Z','2026-08-12T00:01:00Z');
                 INSERT INTO attempt_answers(attempt_id,question_id,answer_json,is_correct,weight,change_count,visit_count,elapsed_ms,marked)
                 VALUES ('attempt-1','q1','\"RAW_ANSWER_MARKER\"',1,1,0,1,1000,0);",
            )?;
            append_learning_event(
                conn,
                NewLearningEvent {
                    event_type: LearningEventType::CoachQuestionAsked,
                    source_kind: "test".into(),
                    source_id: Some("private-marker".into()),
                    activity: None,
                    asset_id: None,
                    attempt_id: None,
                    question_id: None,
                    skill_key: None,
                    occurred_at: "2026-08-12T00:00:00Z".into(),
                    payload: json!({"marker":"PRIVATE_EVENT_MARKER"}),
                    schema_version: 1,
                    sensitivity: "private".into(),
                },
            )?;
            Ok(())
        })
        .unwrap();
        let executor = LearningReadTools::new(&db);
        let detail = executor
            .execute(&call(
                "get_attempt_detail",
                json!({"attemptId":"attempt-1"}),
            ))
            .await;
        assert_eq!(detail.status, AgentToolStatus::Succeeded);
        assert!(!detail.model_content.contains("RAW_ANSWER_MARKER"));
        assert!(!detail
            .audit_result
            .to_string()
            .contains("RAW_ANSWER_MARKER"));

        let search = executor
            .execute(&call("search_learning_events", json!({"limit":10})))
            .await;
        assert!(!search.model_content.contains("PRIVATE_EVENT_MARKER"));
        assert!(!search
            .audit_result
            .to_string()
            .contains("PRIVATE_EVENT_MARKER"));
    }

    #[tokio::test]
    async fn rejects_oversized_model_output_without_copying_it_to_audit() {
        let (_directory, db) = tools();
        let marker = "LARGE_PAYLOAD_MARKER".repeat(80);
        db.with_conn(|conn| {
            for index in 0..100 {
                append_learning_event(
                    conn,
                    NewLearningEvent {
                        event_type: LearningEventType::CoachQuestionAsked,
                        source_kind: "test".into(),
                        source_id: Some(format!("large-{index}")),
                        activity: None,
                        asset_id: None,
                        attempt_id: None,
                        question_id: None,
                        skill_key: None,
                        occurred_at: "2026-08-12T00:00:00Z".into(),
                        payload: json!({"marker":marker}),
                        schema_version: 1,
                        sensitivity: "normal".into(),
                    },
                )?;
            }
            Ok(())
        })
        .unwrap();
        let executor = LearningReadTools::new(&db);
        let result = executor
            .execute(&call("search_learning_events", json!({"limit":100})))
            .await;
        assert_eq!(result.status, AgentToolStatus::Rejected);
        assert_eq!(result.error.unwrap().code, "agent.tool_output_too_large");
        assert!(!result
            .audit_result
            .to_string()
            .contains("LARGE_PAYLOAD_MARKER"));
    }
}
