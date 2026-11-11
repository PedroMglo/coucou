//! Private projection of the documented headless NDJSON output.
//! https://antigravity.google/docs/cli/headless
//!
//! Tool payloads, reasoning, usage and native diagnostics never leave this parser.
use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

use super::types::{valid_uuid, Fault, MAX_LINE};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NativeStatus {
    Success,
    Error,
    Cancelled,
    Interrupted,
    Waiting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NativeEvent {
    Init {
        session_id: String,
        cwd: PathBuf,
        model: Option<String>,
        permission_mode: String,
    },
    UserObserved {
        session_id: String,
    },
    Delta {
        session_id: String,
        text: String,
    },
    Step {
        session_id: String,
    },
    /// Private proof candidate from the owned child's stdout, never a public
    /// chat event or approval on its own. Missing fields cannot mint authority.
    ToolProposed {
        session_id: String,
        step: u64,
        command: String,
        cwd: PathBuf,
    },
    ToolFinished {
        session_id: String,
        step: u64,
    },
    /// An ACTIVE update without a complete, consistent shell proof revokes any
    /// earlier proof for its step. It must not be hidden as a generic Step.
    ToolUnverified {
        session_id: String,
        step: Option<u64>,
    },
    Result {
        session_id: String,
        status: NativeStatus,
        response: String,
        num_turns: Option<u64>,
    },
}

// Typed fields reject duplicate known keys and invalid types. Extra native
// metadata is ignored rather than retained in a public/raw-event variant.
#[derive(Deserialize)]
#[serde(tag = "event")]
enum WireEvent {
    #[serde(rename = "init")]
    Init {
        conversation_id: String,
        init: InitPayload,
    },
    #[serde(rename = "step_update")]
    Step {
        #[serde(default, deserialize_with = "present_string")]
        conversation_id: Option<String>,
        step_update: StepPayload,
    },
    #[serde(rename = "result")]
    Result {
        #[serde(default, deserialize_with = "present_string")]
        conversation_id: Option<String>,
        result: ResultPayload,
    },
}

#[derive(Deserialize)]
struct InitPayload {
    cwd: String,
    permission_mode: String,
    #[serde(default, deserialize_with = "present_string")]
    model: Option<String>,
}

#[derive(Deserialize)]
struct StepPayload {
    conversation_id: String,
    state: String,
    step_type: String,
    #[serde(default, deserialize_with = "present_u64")]
    step_index: Option<u64>,
    #[serde(default, deserialize_with = "present_string")]
    text_delta: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    tool_info: Option<ToolInfo>,
}

#[derive(Deserialize)]
struct ToolInfo {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    parameters: Option<ToolParameters>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ToolParameters {
    Shell {
        #[serde(default, rename = "CommandLine")]
        command: Option<String>,
        #[serde(default, rename = "Cwd")]
        cwd: Option<String>,
    },
    // Other native schemas (or a shell schema with duplicate/invalid fields)
    // cannot prove a shell operation. Do not coerce them into approval data.
    Other(serde_json::Value),
}

#[derive(Deserialize)]
struct ResultPayload {
    conversation_id: String,
    status: String,
    #[serde(default, deserialize_with = "present_string")]
    response: Option<String>,
    #[serde(default, deserialize_with = "present_u64")]
    num_turns: Option<u64>,
}

// Missing optional fields are valid. Explicit null is not a valid string/count.
fn present_string<'de, D: Deserializer<'de>>(input: D) -> Result<Option<String>, D::Error> {
    String::deserialize(input).map(Some)
}

fn present_u64<'de, D: Deserializer<'de>>(input: D) -> Result<Option<u64>, D::Error> {
    u64::deserialize(input).map(Some)
}

fn bounded(text: &str) -> Result<(), Fault> {
    if text.len() > MAX_LINE {
        Err(Fault::Limit)
    } else {
        Ok(())
    }
}

fn required(text: &str) -> Result<(), Fault> {
    bounded(text)?;
    if text.is_empty() {
        Err(Fault::Protocol)
    } else {
        Ok(())
    }
}

fn identity(id: &str, envelope: Option<&str>) -> Result<(), Fault> {
    if !valid_uuid(id) || envelope.is_some_and(|other| other != id) {
        return Err(Fault::Protocol);
    }
    Ok(())
}

pub(super) fn parse(line: &[u8]) -> Result<NativeEvent, Fault> {
    if line.len() > MAX_LINE {
        return Err(Fault::Limit);
    }
    let wire: WireEvent = serde_json::from_slice(line).map_err(|_| Fault::Protocol)?;
    match wire {
        WireEvent::Init {
            conversation_id,
            init,
        } => {
            identity(&conversation_id, None)?;
            required(&init.cwd)?;
            required(&init.permission_mode)?;
            if let Some(model) = &init.model {
                required(model)?;
            }
            Ok(NativeEvent::Init {
                session_id: conversation_id,
                cwd: PathBuf::from(init.cwd),
                model: init.model,
                permission_mode: init.permission_mode,
            })
        }
        WireEvent::Step {
            conversation_id,
            step_update: step,
        } => {
            identity(&step.conversation_id, conversation_id.as_deref())?;
            required(&step.step_type)?;
            if !matches!(step.state.as_str(), "ACTIVE" | "DONE") {
                return Err(Fault::Protocol);
            }
            // A tool's index only correlates a private operation inside the
            // already verified owner/local turn. It is never a native turn ID.
            if let Some(text) = &step.text_delta {
                bounded(text)?;
            }
            if step.step_type == "tool" && step.state == "DONE" {
                return Ok(if let Some(index) = step.step_index {
                    NativeEvent::ToolFinished {
                        session_id: step.conversation_id,
                        step: index,
                    }
                } else {
                    NativeEvent::ToolUnverified {
                        session_id: step.conversation_id,
                        step: None,
                    }
                });
            }
            if step.step_type == "tool"
                && step.state == "ACTIVE"
                && step.tool_name.as_deref() == Some("run_command")
            {
                if let (
                    Some(index),
                    Some(ToolInfo {
                        name: Some(name),
                        parameters:
                            Some(ToolParameters::Shell {
                                command: Some(command),
                                cwd: Some(cwd),
                            }),
                    }),
                ) = (step.step_index, step.tool_info)
                {
                    if name == "run_command"
                        && !command.is_empty()
                        && !command.contains('\0')
                        && PathBuf::from(&cwd).is_absolute()
                        && !cwd.contains('\0')
                    {
                        return Ok(NativeEvent::ToolProposed {
                            session_id: step.conversation_id,
                            step: index,
                            command,
                            cwd: PathBuf::from(cwd),
                        });
                    }
                }
            }
            if step.step_type == "tool" && step.state == "ACTIVE" {
                return Ok(NativeEvent::ToolUnverified {
                    session_id: step.conversation_id,
                    step: step.step_index,
                });
            }
            if step.step_type == "user_input" && step.state == "DONE" {
                Ok(NativeEvent::UserObserved {
                    session_id: step.conversation_id,
                })
            } else if step.step_type == "agent_response" {
                match step.text_delta {
                    Some(text) => Ok(NativeEvent::Delta {
                        session_id: step.conversation_id,
                        text,
                    }),
                    None => Ok(NativeEvent::Step {
                        session_id: step.conversation_id,
                    }),
                }
            } else {
                Ok(NativeEvent::Step {
                    session_id: step.conversation_id,
                })
            }
        }
        WireEvent::Result {
            conversation_id,
            result,
        } => {
            identity(&result.conversation_id, conversation_id.as_deref())?;
            let status = match result.status.as_str() {
                "SUCCESS" => NativeStatus::Success,
                "ERROR" => NativeStatus::Error,
                "CANCELED" => NativeStatus::Cancelled,
                "INTERRUPTED" => NativeStatus::Interrupted,
                "WAITING" => NativeStatus::Waiting,
                _ => return Err(Fault::Protocol),
            };
            let response = match result.response {
                Some(text) => {
                    bounded(&text)?;
                    text
                }
                None if status == NativeStatus::Success => return Err(Fault::Protocol),
                None => String::new(),
            };
            Ok(NativeEvent::Result {
                session_id: result.conversation_id,
                status,
                response,
                num_turns: result.num_turns,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    const ID: &str = "019a734b-0000-7000-8000-000000000001";
    const OTHER: &str = "019a734b-0000-7000-8000-000000000002";

    fn read(value: Value) -> NativeEvent {
        match parse(&serde_json::to_vec(&value).unwrap()) {
            Ok(event) => event,
            Err(_) => panic!("synthetic native event was rejected"),
        }
    }

    fn protocol(value: Value) {
        assert!(matches!(
            parse(&serde_json::to_vec(&value).unwrap()),
            Err(Fault::Protocol)
        ));
    }

    fn init() -> Value {
        json!({"event":"init","conversation_id":ID,
            "init":{"cwd":"/synthetic/workspace","permission_mode":"request-review"}})
    }

    fn step(kind: &str, state: &str) -> Value {
        json!({"event":"step_update","step_update":{
            "conversation_id":ID,"step_index":3,"state":state,"step_type":kind}})
    }

    fn result(status: &str) -> Value {
        json!({"event":"result","result":{
            "conversation_id":ID,"status":status,"response":"public reply"}})
    }

    #[test]
    fn init_projects_only_identity_and_effective_configuration() {
        let mut value = init();
        value["init"]["tools"] = json!(["run_command", "synthetic-secret-tool"]);
        value["init"]["model"] = json!("synthetic-model");
        value["init"]["agent"] = json!("synthetic-private-agent");
        assert_eq!(
            read(value),
            NativeEvent::Init {
                session_id: ID.into(),
                cwd: PathBuf::from("/synthetic/workspace"),
                model: Some("synthetic-model".into()),
                permission_mode: "request-review".into(),
            }
        );
        assert!(matches!(
            read(init()),
            NativeEvent::Init { model: None, .. }
        ));
    }

    #[test]
    fn init_requires_metadata_and_rejects_invalid_optional_model() {
        for field in ["cwd", "permission_mode"] {
            let mut missing = init();
            missing["init"].as_object_mut().unwrap().remove(field);
            protocol(missing);
            for bad in [json!(null), json!(false), json!(7), json!(""), json!({})] {
                let mut value = init();
                value["init"][field] = bad;
                protocol(value);
            }
        }
        for bad in [json!(null), json!(false), json!(7), json!(""), json!([])] {
            let mut value = init();
            value["init"]["model"] = bad;
            protocol(value);
        }
    }

    #[test]
    fn only_completed_user_input_is_observed() {
        let mut value = step("user_input", "DONE");
        value["step_update"]["text_delta"] = json!("synthetic private prompt echo");
        assert_eq!(
            read(value),
            NativeEvent::UserObserved {
                session_id: ID.into()
            }
        );
        assert_eq!(
            read(step("user_input", "ACTIVE")),
            NativeEvent::Step {
                session_id: ID.into()
            }
        );
    }

    #[test]
    fn agent_deltas_are_exact_and_done_without_delta_is_valid() {
        for state in ["ACTIVE", "DONE"] {
            let text = "ação\n\u{0000}public tail";
            let mut value = step("agent_response", state);
            value["step_update"]["text_delta"] = json!(text);
            assert_eq!(
                read(value),
                NativeEvent::Delta {
                    session_id: ID.into(),
                    text: text.into()
                }
            );
            assert_eq!(
                read(step("agent_response", state)),
                NativeEvent::Step {
                    session_id: ID.into()
                }
            );
        }
        let mut empty = step("agent_response", "DONE");
        empty["step_update"]["text_delta"] = json!("");
        assert_eq!(
            read(empty),
            NativeEvent::Delta {
                session_id: ID.into(),
                text: String::new()
            }
        );
    }

    #[test]
    fn non_response_steps_never_project_reasoning_tools_or_raw_errors() {
        for kind in ["tool", "checkpoint", "reasoning", "future_native_step"] {
            let mut value = step(kind, "DONE");
            value["step_update"]["text_delta"] = json!("synthetic private reasoning");
            value["step_update"]["tool_info"] = json!({
                "parameters":{"CommandLine":"synthetic private command"},
                "output":"synthetic private output","error":{"message":"synthetic private failure"}});
            value["step_update"]["subagent_info"] = json!({"conversation_id":OTHER});
            assert_eq!(
                read(value),
                if kind == "tool" {
                    NativeEvent::ToolFinished {
                        session_id: ID.into(),
                        step: 3,
                    }
                } else {
                    NativeEvent::Step {
                        session_id: ID.into(),
                    }
                }
            );
        }
    }

    #[test]
    fn private_shell_operation_requires_exact_explicit_native_fields() {
        let mut value = step("tool", "ACTIVE");
        value["step_update"]["tool_name"] = json!("run_command");
        value["step_update"]["tool_info"] = json!({"name":"run_command","parameters":{"CommandLine":"pwd","Cwd":"/workspace"},"output":"private output"});
        assert_eq!(
            read(value.clone()),
            NativeEvent::ToolProposed {
                session_id: ID.into(),
                step: 3,
                command: "pwd".into(),
                cwd: "/workspace".into()
            }
        );
        for field in ["CommandLine", "Cwd"] {
            let mut missing = value.clone();
            missing["step_update"]["tool_info"]["parameters"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(matches!(read(missing), NativeEvent::ToolUnverified { .. }));
        }
        for field in ["tool_name", "step_index"] {
            let mut missing = value.clone();
            missing["step_update"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(matches!(read(missing), NativeEvent::ToolUnverified { .. }));
        }
        let mut relative = value.clone();
        relative["step_update"]["tool_info"]["parameters"]["Cwd"] = json!("relative");
        assert!(matches!(read(relative), NativeEvent::ToolUnverified { .. }));
        let mut other = value;
        other["step_update"]["tool_info"]["name"] = json!("other");
        assert!(matches!(read(other), NativeEvent::ToolUnverified { .. }));
    }

    #[test]
    fn duplicate_shell_parameters_cannot_mint_an_operation() {
        let line = format!(
            r#"{{"event":"step_update","step_update":{{"conversation_id":"{ID}","state":"ACTIVE","step_type":"tool","step_index":3,"tool_name":"run_command","tool_info":{{"name":"run_command","parameters":{{"CommandLine":"one","CommandLine":"two","Cwd":"/workspace"}}}}}}}}"#
        );
        assert!(matches!(
            parse(line.as_bytes()),
            Ok(NativeEvent::ToolUnverified { .. }) | Err(Fault::Protocol)
        ));
    }

    #[test]
    fn step_shape_and_present_optional_fields_are_checked() {
        for field in ["state", "step_type"] {
            let mut value = step("agent_response", "DONE");
            value["step_update"].as_object_mut().unwrap().remove(field);
            protocol(value);
        }
        for state in ["done", "WAITING", "", "RUNNING"] {
            protocol(step("agent_response", state));
        }
        protocol(step("", "DONE"));
        for bad in [json!(null), json!(false), json!(1), json!({})] {
            let mut value = step("agent_response", "DONE");
            value["step_update"]["text_delta"] = bad;
            protocol(value);
        }
        let mut absent_index = step("agent_response", "DONE");
        absent_index["step_update"]
            .as_object_mut()
            .unwrap()
            .remove("step_index");
        assert!(matches!(read(absent_index), NativeEvent::Step { .. }));
        for bad in [json!(null), json!(-1), json!(0.5), json!("1"), json!(false)] {
            let mut value = step("tool", "DONE");
            value["step_update"]["step_index"] = bad;
            protocol(value);
        }
    }

    #[test]
    fn result_maps_exact_native_statuses_and_preserves_cumulative_counter() {
        for (wire, status) in [
            ("SUCCESS", NativeStatus::Success),
            ("ERROR", NativeStatus::Error),
            ("CANCELED", NativeStatus::Cancelled),
            ("INTERRUPTED", NativeStatus::Interrupted),
            ("WAITING", NativeStatus::Waiting),
        ] {
            let mut value = result(wire);
            value["result"]["num_turns"] = json!(17);
            value["result"]["error"] = json!("synthetic raw failure must not escape");
            assert_eq!(
                read(value),
                NativeEvent::Result {
                    session_id: ID.into(),
                    status,
                    response: "public reply".into(),
                    num_turns: Some(17),
                }
            );
        }
        for count in [0, u64::MAX] {
            let mut value = result("SUCCESS");
            value["result"]["num_turns"] = json!(count);
            assert!(
                matches!(read(value), NativeEvent::Result { num_turns: Some(n), .. } if n == count)
            );
        }
    }

    #[test]
    fn success_requires_response_but_empty_success_is_not_rewritten_as_an_error() {
        let mut value = result("SUCCESS");
        value["result"]["response"] = json!("");
        assert_eq!(
            read(value),
            NativeEvent::Result {
                session_id: ID.into(),
                status: NativeStatus::Success,
                response: String::new(),
                num_turns: None,
            }
        );
        let mut missing = result("SUCCESS");
        missing["result"]
            .as_object_mut()
            .unwrap()
            .remove("response");
        protocol(missing);
        for status in ["ERROR", "CANCELED", "INTERRUPTED", "WAITING"] {
            let mut missing = result(status);
            missing["result"]
                .as_object_mut()
                .unwrap()
                .remove("response");
            missing["result"]["error"] = json!({"message":"synthetic sensitive diagnostic"});
            assert!(
                matches!(read(missing), NativeEvent::Result { response, .. } if response.is_empty())
            );
        }
    }

    #[test]
    fn invalid_status_response_and_counter_types_fail_closed() {
        for status in ["INVALID", "RUNNING", "CANCELLED", "success", "", "future"] {
            protocol(result(status));
        }
        for bad in [json!(null), json!(false), json!(3), json!([]), json!({})] {
            let mut value = result("ERROR");
            value["result"]["response"] = bad;
            protocol(value);
        }
        for bad in [
            json!(null),
            json!(-1),
            json!(1.5),
            json!("2"),
            json!(false),
            json!({}),
        ] {
            let mut value = result("SUCCESS");
            value["result"]["num_turns"] = bad;
            protocol(value);
        }
    }

    #[test]
    fn each_event_requires_its_documented_uuid_location() {
        for mut value in [init(), step("tool", "DONE"), result("SUCCESS")] {
            let key = match value["event"].as_str().unwrap() {
                "init" => None,
                "step_update" => Some("step_update"),
                _ => Some("result"),
            };
            let payload = if let Some(key) = key {
                &mut value[key]
            } else {
                &mut value
            };
            payload.as_object_mut().unwrap().remove("conversation_id");
            protocol(value);
        }
        for bad in [
            json!(""),
            json!("not-a-uuid"),
            json!(null),
            json!(false),
            json!(37),
        ] {
            let mut value = init();
            value["conversation_id"] = bad.clone();
            protocol(value);
            for mut nested in [step("tool", "DONE"), result("SUCCESS")] {
                let key = if nested["event"] == "result" {
                    "result"
                } else {
                    "step_update"
                };
                nested[key]["conversation_id"] = bad.clone();
                nested["conversation_id"] = json!(ID); // A root ID is never a fallback.
                protocol(nested);
            }
        }
    }

    #[test]
    fn contradictory_ids_are_rejected_and_id_bytes_are_preserved() {
        for mut value in [step("tool", "DONE"), result("SUCCESS")] {
            value["conversation_id"] = json!(OTHER);
            protocol(value);
        }
        let uppercase = "019A734B-0000-7000-8000-00000000000A";
        let mut value = init();
        value["conversation_id"] = json!(uppercase);
        assert!(
            matches!(read(value), NativeEvent::Init { session_id, .. } if session_id == uppercase)
        );
    }

    #[test]
    fn raw_control_unknown_and_invalid_json_never_become_events() {
        for event in [
            "raw",
            "user",
            "control_request",
            "control_response",
            "future",
            "",
        ] {
            protocol(
                json!({"event":event,"conversation_id":ID,"message":"synthetic private data"}),
            );
        }
        for value in [
            json!(null),
            json!([]),
            json!("text"),
            json!({}),
            json!({"event":1}),
        ] {
            protocol(value);
        }
        for line in [
            b"".as_slice(),
            b"not json",
            b"\xff",
            b"{}\n{}",
            b"{\"event\":\"init\"}",
        ] {
            assert!(matches!(parse(line), Err(Fault::Protocol)));
        }
    }

    #[test]
    fn duplicate_known_identity_and_public_text_fields_are_rejected() {
        let lines = [
            format!(
                r#"{{"event":"init","conversation_id":"{ID}","conversation_id":"{OTHER}","init":{{"cwd":"/synthetic","permission_mode":"request-review"}}}}"#
            ),
            format!(
                r#"{{"event":"step_update","step_update":{{"conversation_id":"{ID}","conversation_id":"{OTHER}","state":"DONE","step_type":"tool"}}}}"#
            ),
            format!(
                r#"{{"event":"result","result":{{"conversation_id":"{ID}","status":"SUCCESS","response":"one","response":"two"}}}}"#
            ),
        ];
        for line in lines {
            assert!(matches!(parse(line.as_bytes()), Err(Fault::Protocol)));
        }
    }

    #[test]
    fn byte_limit_is_checked_before_json_and_does_not_truncate() {
        let mut line = serde_json::to_vec(&init()).unwrap();
        assert!(line.len() < MAX_LINE);
        line.resize(MAX_LINE, b' ');
        assert!(matches!(parse(&line), Ok(NativeEvent::Init { .. })));
        line.push(b' ');
        assert!(matches!(parse(&line), Err(Fault::Limit)));
        assert!(matches!(
            parse(&vec![b'x'; MAX_LINE + 1]),
            Err(Fault::Limit)
        ));
    }
}
