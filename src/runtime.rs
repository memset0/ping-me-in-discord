use jiff::Zoned;
use serde::Serialize;

const UNKNOWN_USER: &str = "unknown-user";
const UNKNOWN_HOST: &str = "unknown-host";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RuntimeMetadata {
    pub(crate) user: String,
    pub(crate) hostname: String,
    pub(crate) codex_thread_id: Option<String>,
    pub(crate) timestamp: RuntimeTimestamp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RuntimeTimestamp {
    pub(crate) local: String,
    pub(crate) unix: i64,
    pub(crate) iso8601: String,
}

impl RuntimeMetadata {
    pub(crate) fn capture() -> Self {
        let now = Zoned::now();
        Self::from_zoned(
            whoami::username().ok(),
            whoami::hostname().ok(),
            current_codex_thread_id(),
            &now,
        )
    }

    fn from_zoned(
        user: Option<String>,
        hostname: Option<String>,
        codex_thread_id: Option<String>,
        now: &Zoned,
    ) -> Self {
        let timestamp = now.timestamp();
        Self {
            user: normalize_identity(user.as_deref(), UNKNOWN_USER),
            hostname: normalize_identity(hostname.as_deref(), UNKNOWN_HOST),
            codex_thread_id: normalize_optional_identity(codex_thread_id.as_deref()),
            timestamp: RuntimeTimestamp {
                local: now.strftime("%-m/%-d %H:%M:%S").to_string(),
                unix: timestamp.as_second(),
                iso8601: timestamp.to_string(),
            },
        }
    }

    #[cfg(test)]
    pub(crate) fn fixed(
        user: &str,
        hostname: &str,
        codex_thread_id: Option<&str>,
        local: &str,
        unix: i64,
        iso8601: &str,
    ) -> Self {
        Self {
            user: normalize_identity(Some(user), UNKNOWN_USER),
            hostname: normalize_identity(Some(hostname), UNKNOWN_HOST),
            codex_thread_id: normalize_optional_identity(codex_thread_id),
            timestamp: RuntimeTimestamp {
                local: local.to_owned(),
                unix,
                iso8601: iso8601.to_owned(),
            },
        }
    }
}

pub(crate) fn current_codex_thread_id() -> Option<String> {
    std::env::var("CODEX_THREAD_ID")
        .ok()
        .and_then(|value| normalize_optional_identity(Some(&value)))
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
    fn derives_all_timestamp_forms_from_one_zoned_instant() {
        let now: Zoned = "2026-07-31T12:00:11+00:00[UTC]".parse().unwrap();
        let runtime = RuntimeMetadata::from_zoned(
            Some("mem".to_owned()),
            Some("vultr".to_owned()),
            Some("thread-123".to_owned()),
            &now,
        );

        assert_eq!(runtime.codex_thread_id.as_deref(), Some("thread-123"));
        assert_eq!(runtime.timestamp.local, "7/31 12:00:11");
        assert_eq!(runtime.timestamp.unix, now.timestamp().as_second());
        assert_eq!(runtime.timestamp.iso8601, now.timestamp().to_string());
    }

    #[test]
    fn normalizes_identity_for_single_line_inline_code() {
        assert_eq!(
            normalize_identity(Some("  mem\n ops\t`admin`  "), UNKNOWN_USER),
            "mem ops 'admin'"
        );
    }

    #[test]
    fn missing_or_empty_identity_uses_explicit_fallbacks() {
        let now: Zoned = "2026-07-31T12:00:11+00:00[UTC]".parse().unwrap();
        let runtime = RuntimeMetadata::from_zoned(None, Some(" \n ".to_owned()), None, &now);

        assert_eq!(runtime.user, UNKNOWN_USER);
        assert_eq!(runtime.hostname, UNKNOWN_HOST);
        assert_eq!(runtime.codex_thread_id, None);
    }

    #[test]
    fn normalizes_optional_thread_id_without_inventing_one() {
        assert_eq!(
            normalize_optional_identity(Some("  thread\n`123`  ")).as_deref(),
            Some("thread '123'")
        );
        assert_eq!(normalize_optional_identity(Some(" \t ")), None);
        assert_eq!(normalize_optional_identity(None), None);
    }
}
