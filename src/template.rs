use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use minijinja::{Environment, UndefinedBehavior};
use serde::Serialize;
use serde_json::{Map, Value, json};
use url::Url;

use crate::{
    avatar::AvatarSelection,
    config::{validate_template_name, validate_template_selector},
    runtime::RuntimeMetadata,
};

const PAYLOAD_FIELDS: &[&str] = &[
    "username",
    "avatar_url",
    "tts",
    "embeds",
    "allowed_mentions",
    "components",
    "poll",
    "flags",
    "thread_name",
];
const LOCAL_FIELDS: &[&str] = &["avatar", "channel", "thread_id"];

#[derive(Clone, Debug, Serialize)]
pub struct RenderedMessage {
    pub template: String,
    pub payload: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<AvatarSelection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

pub fn build_context(
    message: Option<String>,
    data_path: Option<&Path>,
    variables: &[String],
    host_override: Option<&str>,
) -> Result<Value> {
    build_context_with_runtime(
        message,
        data_path,
        variables,
        RuntimeMetadata::capture(host_override)?,
    )
}

fn build_context_with_runtime(
    message: Option<String>,
    data_path: Option<&Path>,
    variables: &[String],
    runtime: RuntimeMetadata,
) -> Result<Value> {
    let mut context = if let Some(path) = data_path {
        let source = fs::read_to_string(path)
            .with_context(|| format!("could not read JSON data {}", path.display()))?;
        let value: Value = serde_json::from_str(&source)
            .with_context(|| format!("could not parse JSON data {}", path.display()))?;
        value
            .as_object()
            .cloned()
            .context("template JSON data must be an object")?
    } else {
        Map::new()
    };

    ensure!(
        !context.contains_key("runtime"),
        "template data key `runtime` is reserved for automatic runtime metadata"
    );

    if let Some(message) = message {
        context.insert("message".to_owned(), Value::String(message));
    }

    for variable in variables {
        let (key, value) = variable
            .split_once('=')
            .with_context(|| format!("template variable `{variable}` must use KEY=VALUE syntax"))?;
        ensure!(
            !key.is_empty()
                && key
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_'),
            "template variable name `{key}` may contain only ASCII letters, digits, and `_`"
        );
        ensure!(
            key != "runtime",
            "template variable `runtime` is reserved for automatic runtime metadata"
        );
        context.insert(key.to_owned(), Value::String(value.to_owned()));
    }

    context.insert(
        "runtime".to_owned(),
        serde_json::to_value(runtime).context("could not serialize automatic runtime metadata")?,
    );

    Ok(Value::Object(context))
}

pub fn render(templates_directory: &Path, name: &str, context: &Value) -> Result<RenderedMessage> {
    let path = template_path(templates_directory, name)?;
    let source = fs::read_to_string(&path)
        .with_context(|| format!("could not read template {}", path.display()))?;
    let rendered = render_source(&source, context)
        .with_context(|| format!("could not render template {}", path.display()))?;
    parse_rendered(name, &rendered)
        .with_context(|| format!("invalid rendered template {}", path.display()))
}

pub fn list(templates_directory: &Path) -> Result<Vec<String>> {
    let entries = fs::read_dir(templates_directory).with_context(|| {
        format!(
            "could not read template directory {}",
            templates_directory.display()
        )
    })?;
    let mut names = BTreeSet::new();
    for entry in entries {
        let entry = entry.with_context(|| {
            format!(
                "could not inspect template directory {}",
                templates_directory.display()
            )
        })?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|extension| extension.to_str()) == Some("md")
        {
            if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                if validate_template_name(stem).is_ok() {
                    names.insert(stem.to_owned());
                }
            }
        }
    }
    Ok(names.into_iter().collect())
}

pub fn validate_directory(templates_directory: &Path, default_template: &str) -> Result<()> {
    ensure!(
        templates_directory.is_dir(),
        "template directory does not exist: {}",
        templates_directory.display()
    );
    let default_path = template_path(templates_directory, default_template)?;
    ensure!(
        default_path.is_file(),
        "default template does not exist: {}",
        default_path.display()
    );
    validate_template_file(&default_path)?;

    for name in list(templates_directory)? {
        let path = template_path(templates_directory, &name)?;
        if path != default_path {
            validate_template_file(&path)?;
        }
    }
    Ok(())
}

fn template_path(directory: &Path, selector: &str) -> Result<PathBuf> {
    validate_template_selector(selector)?;
    let path = Path::new(selector);
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    Ok(directory.join(format!("{selector}.md")))
}

fn validate_template_file(path: &Path) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("could not read template {}", path.display()))?;
    compile_source(&source)
        .with_context(|| format!("template syntax is invalid in {}", path.display()))?;
    validate_frontmatter_delimiters(&source)
        .with_context(|| format!("frontmatter is invalid in {}", path.display()))
}

fn environment() -> Environment<'static> {
    let mut environment = Environment::new();
    environment.set_undefined_behavior(UndefinedBehavior::Strict);
    environment
}

fn compile_source(source: &str) -> Result<()> {
    environment()
        .template_from_str(source)
        .context("MiniJinja syntax error")?;
    Ok(())
}

fn render_source(source: &str, context: &Value) -> Result<String> {
    environment()
        .template_from_str(source)
        .context("MiniJinja syntax error")?
        .render(context)
        .context("MiniJinja rendering error")
}

fn validate_frontmatter_delimiters(source: &str) -> Result<()> {
    if starts_with_frontmatter(source) {
        split_frontmatter(source)?;
    }
    Ok(())
}

fn parse_rendered(name: &str, source: &str) -> Result<RenderedMessage> {
    let (frontmatter, body) = split_frontmatter(source)?;
    let mut fields = if let Some(frontmatter) = frontmatter {
        let yaml: serde_yaml::Value =
            serde_yaml::from_str(frontmatter).context("could not parse YAML frontmatter")?;
        let json = serde_json::to_value(yaml).context("could not convert YAML frontmatter")?;
        json.as_object()
            .cloned()
            .context("template frontmatter must be a YAML mapping")?
    } else {
        Map::new()
    };

    for key in fields.keys() {
        ensure!(
            PAYLOAD_FIELDS.contains(&key.as_str()) || LOCAL_FIELDS.contains(&key.as_str()),
            "unsupported frontmatter field `{key}`"
        );
    }

    let channel = take_optional_selector(&mut fields, "channel")?;
    let avatar =
        take_optional_string(&mut fields, "avatar")?.map(|name| AvatarSelection::Profile { name });
    let thread_id = take_optional_id(&mut fields, "thread_id")?;
    if avatar.is_some() && fields.contains_key("avatar_url") {
        bail!("frontmatter cannot set both `avatar` and `avatar_url`");
    }

    let content = body.trim_matches(['\r', '\n']);
    if !content.is_empty() {
        ensure!(
            content.chars().count() <= 2_000,
            "Discord message content exceeds 2000 characters"
        );
        fields.insert("content".to_owned(), Value::String(content.to_owned()));
    }

    if !fields.contains_key("allowed_mentions") {
        fields.insert("allowed_mentions".to_owned(), json!({ "parse": [] }));
    }

    validate_username(fields.get("username"))?;
    validate_avatar_url(fields.get("avatar_url"))?;
    validate_tts(fields.get("tts"))?;
    validate_thread_name(fields.get("thread_name"))?;
    normalize_embed_colors(fields.get_mut("embeds"))?;
    validate_embeds(fields.get("embeds"))?;

    let has_message = fields
        .get("content")
        .and_then(Value::as_str)
        .is_some_and(|content| !content.is_empty())
        || nonempty_array(fields.get("embeds"))
        || nonempty_array(fields.get("components"))
        || fields.get("poll").is_some_and(|value| !value.is_null());
    ensure!(
        has_message,
        "rendered template must contain content, embeds, components, or a poll"
    );

    Ok(RenderedMessage {
        template: name.to_owned(),
        payload: Value::Object(fields),
        channel,
        avatar,
        thread_id,
    })
}

fn starts_with_frontmatter(source: &str) -> bool {
    source.starts_with("---\n") || source.starts_with("---\r\n")
}

fn split_frontmatter(source: &str) -> Result<(Option<&str>, &str)> {
    if !starts_with_frontmatter(source) {
        return Ok((None, source));
    }

    let first_newline = source
        .find('\n')
        .context("frontmatter opening is incomplete")?
        + 1;
    let rest = &source[first_newline..];
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        let marker = line.trim_end_matches(['\r', '\n']);
        if marker == "---" {
            let frontmatter = &rest[..offset];
            let body = &rest[offset + line.len()..];
            return Ok((Some(frontmatter), body));
        }
        offset += line.len();
    }
    if rest.trim_end_matches('\r') == "---" {
        return Ok((Some(""), ""));
    }
    bail!("frontmatter opens with `---` but has no closing `---`")
}

fn take_optional_string(fields: &mut Map<String, Value>, key: &str) -> Result<Option<String>> {
    match fields.remove(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(Some(value)),
        Some(_) => bail!("frontmatter `{key}` must be a non-empty string"),
    }
}

fn take_optional_id(fields: &mut Map<String, Value>, key: &str) -> Result<Option<String>> {
    match fields.remove(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value))
            if !value.is_empty() && value.chars().all(|character| character.is_ascii_digit()) =>
        {
            Ok(Some(value))
        }
        Some(Value::Number(value)) if value.is_u64() => Ok(Some(value.to_string())),
        Some(_) => bail!("frontmatter `{key}` must be a Discord numeric ID"),
    }
}

fn take_optional_selector(fields: &mut Map<String, Value>, key: &str) -> Result<Option<String>> {
    match fields.remove(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(Some(value)),
        Some(Value::Number(value)) if value.is_u64() => Ok(Some(value.to_string())),
        Some(_) => bail!("frontmatter `{key}` must be a non-empty channel alias or numeric ID"),
    }
}

fn validate_username(value: Option<&Value>) -> Result<()> {
    if let Some(value) = value {
        let username = value
            .as_str()
            .context("frontmatter `username` must be a string")?;
        ensure!(
            (1..=80).contains(&username.chars().count()),
            "frontmatter `username` must contain between 1 and 80 characters"
        );
    }
    Ok(())
}

fn validate_avatar_url(value: Option<&Value>) -> Result<()> {
    if let Some(value) = value {
        let value = value
            .as_str()
            .context("frontmatter `avatar_url` must be a string")?;
        let url = Url::parse(value).context("frontmatter `avatar_url` must be a valid URL")?;
        ensure!(
            url.scheme() == "https",
            "frontmatter `avatar_url` must use HTTPS"
        );
    }
    Ok(())
}

fn validate_tts(value: Option<&Value>) -> Result<()> {
    if let Some(value) = value {
        ensure!(
            value.is_boolean(),
            "frontmatter `tts` must be true or false"
        );
    }
    Ok(())
}

fn validate_thread_name(value: Option<&Value>) -> Result<()> {
    if let Some(value) = value {
        let name = value
            .as_str()
            .context("frontmatter `thread_name` must be a string")?;
        ensure!(
            (1..=100).contains(&name.chars().count()),
            "frontmatter `thread_name` must contain between 1 and 100 characters"
        );
    }
    Ok(())
}

fn normalize_embed_colors(value: Option<&mut Value>) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    let embeds = value
        .as_array_mut()
        .context("frontmatter `embeds` must be an array")?;
    for embed in embeds {
        let object = embed
            .as_object_mut()
            .context("every embed must be a mapping")?;
        let Some(color) = object.get_mut("color") else {
            continue;
        };
        if let Some(text) = color.as_str() {
            let digits = text.strip_prefix('#').unwrap_or(text);
            ensure!(
                digits.len() == 6
                    && digits
                        .chars()
                        .all(|character| character.is_ascii_hexdigit()),
                "embed color `{text}` must be #RRGGBB"
            );
            *color = Value::Number(
                u64::from_str_radix(digits, 16)
                    .context("could not parse embed color")?
                    .into(),
            );
        }
    }
    Ok(())
}

fn validate_embeds(value: Option<&Value>) -> Result<()> {
    if let Some(value) = value {
        let embeds = value
            .as_array()
            .context("frontmatter `embeds` must be an array")?;
        ensure!(embeds.len() <= 10, "Discord supports at most 10 embeds");
    }
    Ok(())
}

fn nonempty_array(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_array)
        .is_some_and(|items| !items.is_empty())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn fixed_runtime() -> RuntimeMetadata {
        RuntimeMetadata::fixed(
            "mem",
            "vultr",
            "CLI",
            "ping-me-in-discord",
            None,
            None,
            "7/31 12:00:11",
            1_775_000_011,
            "2026-07-31T12:00:11Z",
        )
    }

    #[test]
    fn context_explicit_variables_override_message_and_json() {
        let root = TempDir::new().unwrap();
        let data = root.path().join("data.json");
        fs::write(&data, r#"{"message":"from-json","project":"api"}"#).unwrap();
        let context = build_context(
            Some("from-message".to_owned()),
            Some(&data),
            &["message=from-var".to_owned()],
            None,
        )
        .unwrap();

        assert_eq!(context["message"], "from-var");
        assert_eq!(context["project"], "api");
    }

    #[test]
    fn context_contains_the_complete_runtime_snapshot() {
        let context = build_context_with_runtime(None, None, &[], fixed_runtime()).unwrap();

        assert_eq!(
            context["runtime"],
            json!({
                "user": "mem",
                "hostname": "vultr",
                "host": "mem@vultr",
                "agent": {
                    "name": "CLI"
                },
                "project": {
                    "name": "ping-me-in-discord"
                },
                "session": {
                    "id": null,
                    "name": "interactive",
                    "title": null
                },
                "codex_thread_id": null,
                "timestamp": {
                    "local": "7/31 12:00:11",
                    "unix": 1_775_000_011_i64,
                    "iso8601": "2026-07-31T12:00:11Z"
                }
            })
        );
    }

    #[test]
    fn rejects_runtime_key_from_json_data() {
        let root = TempDir::new().unwrap();
        let data = root.path().join("data.json");
        fs::write(&data, r#"{"runtime":{"hostname":"spoofed"}}"#).unwrap();

        let error = build_context_with_runtime(None, Some(&data), &[], fixed_runtime())
            .unwrap_err()
            .to_string();

        assert!(error.contains("data key `runtime` is reserved"));
    }

    #[test]
    fn rejects_runtime_key_from_explicit_variables() {
        let error = build_context_with_runtime(
            None,
            None,
            &["runtime=spoofed".to_owned()],
            fixed_runtime(),
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("variable `runtime` is reserved"));
    }

    #[test]
    fn ordinary_host_variable_does_not_replace_runtime_host() {
        let context =
            build_context_with_runtime(None, None, &["host=worker-1".to_owned()], fixed_runtime())
                .unwrap();

        assert_eq!(context["host"], "worker-1");
        assert_eq!(context["runtime"]["host"], "mem@vultr");
    }

    #[test]
    fn starter_template_renders_exact_layout_and_preserves_markdown() {
        let context = build_context_with_runtime(
            Some("build **complete**".to_owned()),
            None,
            &[],
            fixed_runtime(),
        )
        .unwrap();
        let source = render_source(crate::config::STARTER_TEMPLATE, &context).unwrap();
        let rendered = parse_rendered("defaults", &source).unwrap();

        assert_eq!(
            rendered.payload["content"],
            "build **complete**\n-# 🏠 mem@vultr   📦 ping-me-in-discord   📅 7/31 12:00:11"
        );
    }

    #[test]
    fn starter_template_uses_an_explicit_complete_host_label() {
        let runtime = fixed_runtime()
            .with_host_override(Some("mukai-h20"))
            .unwrap();
        let context =
            build_context_with_runtime(Some("build complete".to_owned()), None, &[], runtime)
                .unwrap();
        let source = render_source(crate::config::STARTER_TEMPLATE, &context).unwrap();
        let rendered = parse_rendered("defaults", &source).unwrap();

        assert_eq!(
            rendered.payload["content"],
            "build complete\n-# 🏠 mukai-h20   📦 ping-me-in-discord   📅 7/31 12:00:11"
        );
    }

    #[test]
    fn starter_template_prefers_the_session_title_and_orders_all_context() {
        let runtime = RuntimeMetadata::fixed(
            "mem",
            "vultr",
            "Codex",
            "ping-me-in-discord",
            Some("019fb637"),
            Some("notification-skill-design"),
            "8/3 12:00:11",
            1_775_000_011,
            "2026-08-03T12:00:11Z",
        );
        let context =
            build_context_with_runtime(Some("build complete".to_owned()), None, &[], runtime)
                .unwrap();
        let source = render_source(crate::config::STARTER_TEMPLATE, &context).unwrap();
        let rendered = parse_rendered("defaults", &source).unwrap();

        assert_eq!(
            rendered.payload["content"],
            "build complete\n-# 🏠 mem@vultr   📦 ping-me-in-discord   🧵 notification-skill-design   🤖 Codex   📅 8/3 12:00:11"
        );
    }

    #[test]
    fn starter_template_falls_back_to_the_full_session_id() {
        let runtime = RuntimeMetadata::fixed(
            "mem",
            "vultr",
            "Codex",
            "ping-me-in-discord",
            Some("019fb637-full-session-id"),
            None,
            "8/3 12:00:11",
            1_775_000_011,
            "2026-08-03T12:00:11Z",
        );
        let context =
            build_context_with_runtime(Some("build complete".to_owned()), None, &[], runtime)
                .unwrap();
        let source = render_source(crate::config::STARTER_TEMPLATE, &context).unwrap();
        let rendered = parse_rendered("defaults", &source).unwrap();

        assert_eq!(
            rendered.payload["content"],
            "build complete\n-# 🏠 mem@vultr   📦 ping-me-in-discord   🧵 019fb637-full-session-id   🤖 Codex   📅 8/3 12:00:11"
        );
    }

    #[test]
    fn starter_template_omits_unavailable_optional_context() {
        let runtime = RuntimeMetadata::fixed(
            "unknown-user",
            "unknown-host",
            "CLI",
            "unknown-project",
            None,
            None,
            "8/3 12:00:11",
            1_775_000_011,
            "2026-08-03T12:00:11Z",
        );
        let context =
            build_context_with_runtime(Some("build complete".to_owned()), None, &[], runtime)
                .unwrap();
        let source = render_source(crate::config::STARTER_TEMPLATE, &context).unwrap();
        let rendered = parse_rendered("defaults", &source).unwrap();

        assert_eq!(
            rendered.payload["content"],
            "build complete\n-# 📅 8/3 12:00:11"
        );
    }

    #[test]
    fn starter_metadata_counts_toward_the_content_limit() {
        let context =
            build_context_with_runtime(Some("x".repeat(2_000)), None, &[], fixed_runtime())
                .unwrap();
        let source = render_source(crate::config::STARTER_TEMPLATE, &context).unwrap();
        let error = parse_rendered("defaults", &source).unwrap_err().to_string();

        assert!(error.contains("content exceeds 2000"));
    }

    #[test]
    fn renders_frontmatter_body_and_safe_mentions() {
        let source = r##"---
username: "{{ project }}"
avatar: rocket
embeds:
  - title: Deploy
    color: "#5865F2"
---
**{{ message }}**
"##;
        let context = json!({"project": "API", "message": "done"});
        let rendered =
            parse_rendered("defaults", &render_source(source, &context).unwrap()).unwrap();

        assert!(matches!(
            rendered.avatar,
            Some(AvatarSelection::Profile { ref name }) if name == "rocket"
        ));
        assert_eq!(rendered.payload["username"], "API");
        assert_eq!(rendered.payload["content"], "**done**");
        assert_eq!(rendered.payload["embeds"][0]["color"], 0x5865F2);
        assert_eq!(rendered.payload["allowed_mentions"]["parse"], json!([]));
        assert!(rendered.payload.get("avatar").is_none());
    }

    #[test]
    fn rejects_undefined_variables() {
        let error = render_source("{{ missing }}", &json!({}))
            .unwrap_err()
            .to_string();
        assert!(error.contains("MiniJinja rendering error"));
    }

    #[test]
    fn rejects_template_traversal() {
        let error = template_path(Path::new("templates"), "../secret")
            .unwrap_err()
            .to_string();
        assert!(error.contains("may contain only"));

        let root = TempDir::new().unwrap();
        let absolute = root.path().join("nested/../external.md");
        let error = template_path(Path::new("templates"), absolute.to_str().unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains("cannot contain parent-directory"));
    }

    #[test]
    fn resolves_named_and_absolute_markdown_templates() {
        assert_eq!(
            template_path(Path::new("templates"), "deploy").unwrap(),
            Path::new("templates/deploy.md")
        );

        let root = TempDir::new().unwrap();
        let absolute = root.path().join("external.md");
        assert_eq!(
            template_path(Path::new("templates"), absolute.to_str().unwrap()).unwrap(),
            absolute
        );
    }

    #[test]
    fn rejects_absolute_non_markdown_template_paths() {
        let root = TempDir::new().unwrap();
        let absolute = root.path().join("external.txt");
        let error = template_path(Path::new("templates"), absolute.to_str().unwrap())
            .unwrap_err()
            .to_string();

        assert!(error.contains("must end in `.md`"));
    }

    #[test]
    fn validates_an_absolute_default_template_outside_the_directory() {
        let root = TempDir::new().unwrap();
        let directory = root.path().join("templates");
        let absolute = root.path().join("external.md");
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("named.md"), "named").unwrap();
        fs::write(&absolute, "{{ message }}").unwrap();

        validate_directory(&directory, absolute.to_str().unwrap()).unwrap();

        fs::write(&absolute, "{{").unwrap();
        let error = validate_directory(&directory, absolute.to_str().unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains("template syntax is invalid"));
        assert!(error.contains(&absolute.display().to_string()));
    }

    #[test]
    fn rejects_empty_payload() {
        let error = parse_rendered("empty", "\n").unwrap_err().to_string();
        assert!(error.contains("must contain content"));
    }

    #[test]
    fn lists_only_safe_markdown_templates() {
        let root = TempDir::new().unwrap();
        fs::write(root.path().join("defaults.md"), "hello").unwrap();
        fs::write(root.path().join("deploy.md"), "hello").unwrap();
        fs::write(root.path().join("notes.txt"), "hello").unwrap();

        assert_eq!(
            list(root.path()).unwrap(),
            vec!["defaults".to_owned(), "deploy".to_owned()]
        );
    }
}
