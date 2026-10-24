use serde_json::{json,Value};
pub(super) fn definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "pills",
            "description": "Add, remove, set as main or list the pills shown in the island at the top of the screen. At most 6 pills besides the main one: remove one first if needed.",
            "input_schema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "action": { "type": "string", "enum": ["add", "remove", "set_main", "list"] },
                    "names": { "type": "array", "maxItems": 32, "items": { "type": "string", "minLength": 1, "maxLength": 128 }, "description": "Pet names as listed. One or more for changes; empty or omitted for list." }
                },
                "required": ["action"]
            }
        }),
        json!({
            "name": "music",
            "description": "Request Spotify control. set_volume requires an explicit integer volume 0-100; search requires query; play_item requires a real URI. Other actions accept only action. A dispatched or accepted request is not a verified playback effect.",
            "input_schema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "action": { "type": "string", "enum": ["play_pause", "pause", "play", "next", "previous", "set_volume", "search", "play_item"] },
                    "query": { "type": "string", "minLength":1,"maxLength":512,"description": "Required only for search. Search requests results; it does not start a track." },
                    "uri": {"type":"string","maxLength":64,"description":"Required only for play_item: exact spotify:track/album/playlist/episode URI with its real 22-character alphanumeric ID. Never invent its ID."},
                    "volume": { "type": "integer", "minimum":0,"maximum":100,"description": "Required only for set_volume. There is no default volume." }
                },
                "required": ["action"]
            }
        }),
        json!({
            "name": "service_info",
            "description": "Read the user's real data that Coucou already has, as JSON: the active pills, the agent sessions (Claude Code, Codex…) and what they are doing, Claude or Codex plan usage, Spotify now playing, and the services' latest data (Stripe, GitHub, Vercel, Resend, n8n, Notion, Cal.com).",
            "input_schema": { "type": "object", "additionalProperties":false,"properties": {} }
        }),
        json!({
            "name": "open_app",
            "description": "Open an app on this computer by name (Firefox, VS Code, Notion, Spotify…).",
            "input_schema": { "type": "object", "additionalProperties":false,"properties": { "name": { "type": "string","minLength":1,"maxLength":128 } }, "required": ["name"] }
        }),
        json!({
            "name": "prepare_email",
            "description": "Open the user's mail app with an email filled in, for them to check and send (it is never sent automatically). The recipient must be an email address.",
            "input_schema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "recipient": { "type": "string", "minLength":1,"maxLength":320,"description": "One email address, without header fields or additional recipients." },
                    "subject": { "type": "string","maxLength":256 },
                    "body": { "type": "string", "maxLength":7000,"description": "The email body. This prepares a draft; the user sends it. The complete arguments must fit 8000 UTF-8 JSON bytes." }
                },
                "required": ["recipient", "subject", "body"]
            }
        }),
    ]
}
