use std::path::Path;

use jiff::Zoned;
use serde::Serialize;

const UNKNOWN_USER: &str = "unknown-user";
const UNKNOWN_HOST: &str = "unknown-host";
const UNKNOWN_PROJECT: &str = "unknown-project";
const DIRECT_CLI_AGENT: &str = "CLI";
const INTERACTIVE_SESSION: &str = "interactive";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RuntimeMetadata {
    pub(crate) user: String,
    pub(crate) hostname: String,
    pub(crate) agent: RuntimeAgent,
    pub(crate) project: RuntimeProject,
    pub(crate) session: RuntimeSession,
    pub(crate) codex_thread_id: Option<String>,
    pub(crate) timestamp: RuntimeTimestamp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RuntimeAgent {
    pub(crate) name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RuntimeProject {
    pub(crate) name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RuntimeSession {
    pub(crate) id: Option<String>,
    pub(crate) name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RuntimeTimestamp {
    pub(crate) local: String,
    pub(crate) unix: i64,
    pub(crate) iso8601: String,
}

impl RuntimeMetadata {
    pub(crate) fn capture() -> Self {
        let generic_session_id = environment_value("PINGME_SESSION_ID");
        let claude_session_id = environment_value("CLAUDE_CODE_SESSION_ID");
        let codex_thread_id = environment_value("CODEX_THREAD_ID");
        let session_id = select_session_id(
            generic_session_id.as_deref(),
            claude_session_id.as_deref(),
            codex_thread_id.as_deref(),
        );
        let agent_name = select_agent_name(
            environment_value("PINGME_AGENT_NAME").as_deref(),
            claude_session_id.as_deref(),
            codex_thread_id.as_deref(),
        );
        let project_name = environment_value("PINGME_PROJECT_NAME").or_else(current_project_name);
        let session_name = environment_value("PINGME_SESSION_NAME");
        let now = Zoned::now();

        Self::from_zoned(
            whoami::username().ok(),
            whoami::hostname().ok(),
            agent_name,
            project_name,
            session_id,
            session_name,
            &now,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_zoned(
        user: Option<String>,
        hostname: Option<String>,
        agent_name: Option<String>,
        project_name: Option<String>,
        session_id: Option<String>,
        session_name: Option<String>,
        now: &Zoned,
    ) -> Self {
        let timestamp = now.timestamp();
        let session_id = normalize_optional_identity(session_id.as_deref());
        let session_name = normalize_optional_identity(session_name.as_deref())
            .unwrap_or_else(|| default_session_name(session_id.as_deref()));
        Self {
            user: normalize_identity(user.as_deref(), UNKNOWN_USER),
            hostname: normalize_identity(hostname.as_deref(), UNKNOWN_HOST),
            agent: RuntimeAgent {
                name: normalize_identity(agent_name.as_deref(), DIRECT_CLI_AGENT),
            },
            project: RuntimeProject {
                name: normalize_identity(project_name.as_deref(), UNKNOWN_PROJECT),
            },
            session: RuntimeSession {
                id: session_id.clone(),
                name: session_name,
            },
            codex_thread_id: session_id,
            timestamp: RuntimeTimestamp {
                local: now.strftime("%-m/%-d %H:%M:%S").to_string(),
                unix: timestamp.as_second(),
                iso8601: timestamp.to_string(),
            },
        }
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fixed(
        user: &str,
        hostname: &str,
        agent_name: &str,
        project_name: &str,
        session_id: Option<&str>,
        session_name: Option<&str>,
        local: &str,
        unix: i64,
        iso8601: &str,
    ) -> Self {
        let session_id = normalize_optional_identity(session_id);
        Self {
            user: normalize_identity(Some(user), UNKNOWN_USER),
            hostname: normalize_identity(Some(hostname), UNKNOWN_HOST),
            agent: RuntimeAgent {
                name: normalize_identity(Some(agent_name), DIRECT_CLI_AGENT),
            },
            project: RuntimeProject {
                name: normalize_identity(Some(project_name), UNKNOWN_PROJECT),
            },
            session: RuntimeSession {
                id: session_id.clone(),
                name: normalize_optional_identity(session_name)
                    .unwrap_or_else(|| default_session_name(session_id.as_deref())),
            },
            codex_thread_id: session_id,
            timestamp: RuntimeTimestamp {
                local: local.to_owned(),
                unix,
                iso8601: iso8601.to_owned(),
            },
        }
    }
}

pub(crate) fn current_agent_session_id() -> Option<String> {
    select_session_id(
        environment_value("PINGME_SESSION_ID").as_deref(),
        environment_value("CLAUDE_CODE_SESSION_ID").as_deref(),
        environment_value("CODEX_THREAD_ID").as_deref(),
    )
}

fn select_session_id(
    generic: Option<&str>,
    claude: Option<&str>,
    codex: Option<&str>,
) -> Option<String> {
    [generic, claude, codex]
        .into_iter()
        .find_map(normalize_optional_identity)
}

fn select_agent_name(
    explicit: Option<&str>,
    claude_session_id: Option<&str>,
    codex_thread_id: Option<&str>,
) -> Option<String> {
    normalize_optional_identity(explicit).or_else(|| {
        if normalize_optional_identity(claude_session_id).is_some() {
            Some("Claude Code".to_owned())
        } else if normalize_optional_identity(codex_thread_id).is_some() {
            Some("Codex".to_owned())
        } else {
            None
        }
    })
}

fn current_project_name() -> Option<String> {
    std::env::current_dir()
        .ok()
        .as_deref()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .and_then(|name| normalize_optional_identity(Some(name)))
}

fn environment_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .and_then(|value| normalize_optional_identity(Some(&value)))
}

fn default_session_name(session_id: Option<&str>) -> String {
    session_id.map_or_else(
        || INTERACTIVE_SESSION.to_owned(),
        |session_id| {
            let prefix = session_id.chars().take(8).collect::<String>();
            format!("session-{prefix}")
        },
    )
}

fn normalize_identity(value: Option<&str>, fallback: &str) -> String {
    let normalized = normalize_inline_code(value.unwrap_or_default());
    if normalized.is_empty() {
        fallback.to_owned()
    } else {
        normalized
    }
}

fn normalize_optional_identity(value: Option<&str>) -> Option<String> {
    let normalized = normalize_inline_code(value.unwrap_or_default());
    (!normalized.is_empty()).then_some(normalized)
}

fn normalize_inline_code(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| match character {
            '`' => '\'',
            character if character.is_control() => ' ',
            character => character,
        })
        .collect::<String>();
    sanitized.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use jiff::Zoned;

    use super::*;

    #[test]
    fn derives_runtime_context_and_timestamp_forms() {
        let now: Zoned = "2026-07-31T12:00:11+00:00[UTC]".parse().unwrap();
        let runtime = RuntimeMetadata::from_zoned(
            Some("mem".to_owned()),
            Some("vultr".to_owned()),
            Some("Codex".to_owned()),
            Some("ping-me-in-discord".to_owned()),
            Some("thread-123".to_owned()),
            Some("notification-skill-design".to_owned()),
            &now,
        );

        assert_eq!(runtime.agent.name, "Codex");
        assert_eq!(runtime.project.name, "ping-me-in-discord");
        assert_eq!(runtime.session.id.as_deref(), Some("thread-123"));
        assert_eq!(runtime.session.name, "notification-skill-design");
        assert_eq!(runtime.codex_thread_id.as_deref(), Some("thread-123"));
        assert_eq!(runtime.timestamp.local, "7/31 12:00:11");
        assert_eq!(runtime.timestamp.unix, now.timestamp().as_second());
        assert_eq!(runtime.timestamp.iso8601, now.timestamp().to_string());
    }

    #[test]
    fn session_id_precedence_is_generic_then_claude_then_codex() {
        assert_eq!(
            select_session_id(Some("generic"), Some("claude"), Some("codex")).as_deref(),
            Some("generic")
        );
        assert_eq!(
            select_session_id(None, Some("claude"), Some("codex")).as_deref(),
            Some("claude")
        );
        assert_eq!(
            select_session_id(None, None, Some("codex")).as_deref(),
            Some("codex")
        );
    }

    #[test]
    fn supported_agents_are_inferred_after_explicit_override() {
        assert_eq!(
            select_agent_name(Some("Custom Agent"), Some("claude"), Some("codex")).as_deref(),
            Some("Custom Agent")
        );
        assert_eq!(
            select_agent_name(None, Some("claude"), Some("codex")).as_deref(),
            Some("Claude Code")
        );
        assert_eq!(
            select_agent_name(None, None, Some("codex")).as_deref(),
            Some("Codex")
        );
    }

    #[test]
    fn normalizes_identity_for_single_line_inline_code() {
        assert_eq!(
            normalize_identity(Some("  mem\n ops\t`admin`  "), UNKNOWN_USER),
            "mem ops 'admin'"
        );
    }

    #[test]
    fn missing_identity_uses_safe_defaults_and_derives_session_name() {
        let now: Zoned = "2026-07-31T12:00:11+00:00[UTC]".parse().unwrap();
        let runtime = RuntimeMetadata::from_zoned(
            None,
            Some(" \n ".to_owned()),
            None,
            None,
            Some("1234567890".to_owned()),
            None,
            &now,
        );

        assert_eq!(runtime.user, UNKNOWN_USER);
        assert_eq!(runtime.hostname, UNKNOWN_HOST);
        assert_eq!(runtime.agent.name, DIRECT_CLI_AGENT);
        assert_eq!(runtime.project.name, UNKNOWN_PROJECT);
        assert_eq!(runtime.session.name, "session-12345678");
    }

    #[test]
    fn missing_session_id_uses_interactive_name_without_inventing_an_id() {
        let now: Zoned = "2026-07-31T12:00:11+00:00[UTC]".parse().unwrap();
        let runtime = RuntimeMetadata::from_zoned(
            Some("mem".to_owned()),
            Some("vultr".to_owned()),
            Some("CLI".to_owned()),
            Some("project".to_owned()),
            None,
            None,
            &now,
        );

        assert_eq!(runtime.session.id, None);
        assert_eq!(runtime.codex_thread_id, None);
        assert_eq!(runtime.session.name, INTERACTIVE_SESSION);
    }
}
