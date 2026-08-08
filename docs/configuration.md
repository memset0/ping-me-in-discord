# 配置参考

## 完整示例

```toml
[discord]
# 二选一：直接 webhook URL
webhook_url = "https://discord.com/api/webhooks/WEBHOOK_ID/WEBHOOK_TOKEN"

# 或 Bot provisioning（需要 MANAGE_WEBHOOKS）
# bot_token = "YOUR_BOT_TOKEN"
webhook_name = "Notify Me"

[channels]
alerts = "123456789012345678"
releases = "234567890123456789"

[templates]
# 相对路径从 config.toml 所在目录开始解析
directory = "templates"

[defaults]
template = "defaults"
# 所有发送默认值均可省略。
# channel = "alerts"
# username = "Ping Me"
# avatar = "letter"
# avatar_url = "https://example.com/avatar.png"
# thread_id = "345678901234567890"
# thread_name = "New thread"
# tts = false

[emoji]
asset_base_url = "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/72x72/"

[avatars.remote]
description = "Use when a hosted image should identify the message"
type = "image"
source = "https://example.com/avatar.png"
size = 256

[avatars.local]
type = "image"
source = "avatar.png"
size = 256

[avatars.rocket]
description = "Use for launches and deployments"
type = "emoji"
emoji = "🚀"
background = "#5865F2"
size = 256
scale = 0.72

[avatars.started]
description = "Agent work started"
type = "emoji"
emoji = "🚀"
background = "#FFFFFF"
size = 256
scale = 0.72

[avatars.progress]
description = "Agent work in progress"
type = "emoji"
emoji = "🔄"
background = "#3B88C3"
size = 256
scale = 0.72

[avatars.success]
description = "Agent work completed successfully"
type = "emoji"
emoji = "✅"
background = "#77B255"
size = 256
scale = 0.72

[avatars.needs-input]
description = "Agent needs user input"
type = "emoji"
emoji = "❓"
background = "#F1C40F"
size = 256
scale = 0.72

[avatars.warning]
description = "Agent warning"
type = "emoji"
emoji = "⚠️"
background = "#E67E22"
size = 256
scale = 0.72

[avatars.error]
description = "Agent work or verification failed"
type = "emoji"
emoji = "❌"
foreground = "#FFFFFF"
background = "#DD2E44"
size = 256
scale = 0.576

[avatars.letter]
type = "text"
text = "N"
foreground = "#FFFFFF"
background = "#5865F2"
size = 256
font_size = 150.0

[avatars.chinese]
type = "text"
text = "告"
foreground = "#FFFFFF"
background = "#ED4245"
font = "/path/to/NotoSansCJK-Regular.ttc"
size = 256
font_size = 150.0

[avatars.bell]
type = "font-icon"
glyph = "U+F0F3"
font = "/path/to/fa-solid-900.ttf"
foreground = "#FFFFFF"
background = "#57F287"
size = 256
font_size = 140.0
```

每个头像 profile 都可以省略 `description`；配置后必须为 1–200 个非空白 Unicode 字符，只用于选择提示，不改变渲染或优先级。颜色接受 `#RRGGBB` 或 `#RRGGBBAA`。emoji 的 `foreground` 也可省略；设置后会用该颜色替换可见 artwork，同时保留原 alpha 轮廓与抗锯齿，省略时保留 emoji 原色。emoji 的 `scale` 省略时为 `0.72`，也可在每个 profile 中独立指定。本地图片和字体相对路径均从 `config.toml` 的目录解析。没有显式 `font` 时，text avatar 会扫描系统字体并选择包含全部 glyph 的字体；找不到时会提示配置兼容字体。

`[channels]` 的 key 是大小写敏感的 alias，value 必须是带引号的 Discord 数字 channel ID。alias 可使用 ASCII 字母、数字、`-` 和 `_`，但不能全部由数字组成。channel selector 可以直接写数字 ID，也可以写 alias：

```console
pingme 'alert' --channel alerts
pingme 'alert' --channel 123456789012345678
```

可以只查看可公开给 agent 的路由摘要；JSON 会给出 alias、ID、`is_default` 以及解析后的 default，但不会包含凭据或其它配置：

```console
pingme channels list
pingme channels list --json
```

指定 channel 后，CLI 会通过 Bot 为最终 channel 查找或创建 webhook，并按 channel ID 缓存。直接 webhook URL 固定绑定其原有 channel，只在所有层都不指定 channel 时使用。

## 参数覆盖

简单发送字段统一使用以下优先级：

```text
显式 CLI argument > template frontmatter > [defaults] > 未设置
```

| CLI argument | frontmatter / `[defaults]` | 说明 |
| --- | --- | --- |
| `--template` | `[defaults].template` | 模板名称；frontmatter 不递归选择模板 |
| `--channel` | `channel` | channel alias 或数字 ID |
| `--username` | `username` | 当前消息显示的 webhook 用户名 |
| `--avatar` | `avatar` | `[avatars.<name>]` profile |
| `--avatar-url` | `avatar_url` | HTTPS 远程头像 |
| `--thread-id` | `thread_id` | webhook channel 中已有 thread 的 ID |
| `--thread-name` | `thread_name` | forum/media channel 中要创建的 thread |
| `--tts` / `--no-tts` | `tts` | 显式启用或禁用 TTS |

`--avatar` 和其它头像 source 是一个整体覆盖项，不会把 CLI 的头像类型与下层 profile 的颜色或字体混合。
`thread_id` 和 `thread_name` 互斥，因为前者选择已有 thread，后者请求 Discord 创建新 thread。

## Template frontmatter

模板文件以可选 YAML frontmatter 开始，其余部分是 Discord Markdown `content`：

新初始化的 `templates/defaults.md` 使用以下精确布局；元信息 blockquote 在前，正文紧接下一行，没有额外空行：

```jinja
> **🏠 `{{ runtime.user }}@{{ runtime.hostname }}`   📅 `{{ runtime.timestamp.local }}`{% if runtime.codex_thread_id %}   🧵 `{{ runtime.codex_thread_id }}`{% endif %}**
{{ message }}
```

`CODEX_THREAD_ID` 不为空时模板追加 thread 字段，否则保持原来的两字段布局。installer 只替换二进制，不修改模板；不带 `--force` 的初始化也拒绝覆盖已有文件。已有用户可以手动采用上面的模板，而不必变更 `config.toml` 或凭据。

```markdown
---
channel: releases
username: Release Bot
avatar: rocket
tts: false
embeds:
  - title: "{{ title }}"
    description: "{{ description }}"
    color: "#5865F2"
allowed_mentions:
  parse: []
thread_id: "123456789012345678"
---
{{ message }}
```

支持字段：

| 字段 | 用途 |
| --- | --- |
| `channel` | channel alias 或数字 ID；仅在需要模板自己决定路由时填写 |
| `username` | 覆盖当前消息显示的 webhook 用户名 |
| `avatar` | 引用 `[avatars.<name>]`，仅在本地处理 |
| `avatar_url` | 直接使用 HTTPS 远程头像；不能与 `avatar` 同时使用 |
| `tts` | Discord TTS 标志 |
| `embeds` | 最多 10 个 Discord rich embed |
| `allowed_mentions` | Discord allowed mention object；省略时默认为 `parse: []` |
| `components` | Discord message components |
| `poll` | Discord poll request |
| `flags` | webhook execute 支持的 message flags |
| `thread_id` | 发送到指定 thread，作为 query 参数处理 |
| `thread_name` | 在 forum/media channel 创建 thread |

Embed 的 `color` 可以直接写 `#RRGGBB`，CLI 会转换成 Discord 所需的整数。模板必须在 `content`、`embeds`、`components` 或 `poll` 中至少产生一项。

模板名只允许 ASCII 字母、数字、`-` 和 `_`，因此不能通过模板名读取 `templates/` 外部文件。

## 变量

MiniJinja 使用严格 undefined 模式：

```jinja
{{ message }}
{{ project }}
```

每次渲染自动注入以下保留对象，三个 timestamp 字段都来自同一次时间采样：

| 变量 | 内容 |
| --- | --- |
| `runtime.user` | 当前系统用户；不可用时为 `unknown-user` |
| `runtime.hostname` | 当前 hostname；不可用时为 `unknown-host` |
| `runtime.codex_thread_id` | 可选的 `CODEX_THREAD_ID` 单行值；与 Discord `thread_id` 无关 |
| `runtime.timestamp.local` | 运行机器本地时间，格式 `M/D HH:mm:ss` |
| `runtime.timestamp.unix` | Unix 整数秒 |
| `runtime.timestamp.iso8601` | UTC ISO 8601 时间 |

顶层键 `runtime` 由 CLI 保留。`--data` 中包含该键或传入 `--var runtime=...` 时会在模板渲染和网络访问前报错。默认模板会把系统用户和 hostname 发送到 Discord；不希望暴露机器命名时，应编辑本机 `defaults.md` 删除该行或改用自己的标签。

变量来源按优先级由低到高为：

1. `--data event.json` 中的 JSON object
2. positional message 注入的 `message`
3. 重复的 `--var key=value`

示例：

```console
pingme 'hello'
ping-me-in-discord send 'deployed' --template release --var version=v1.2.3
ping-me-in-discord send --template alert --data alert.json --var severity=critical
```

## Agent 状态头像 profile

新初始化的 `config.toml` 提供 `started`、`progress`、`success`、`needs-input`、`warning` 和 `error` 六个普通 emoji profile。严格通知 skill 只选择同名 `--avatar <status>`，不会携带 emoji、颜色、尺寸或 scale；因此配置文件是视觉设定的唯一来源。前五个 starter profile 使用 `scale = 0.72`，`error` 使用已确认的 `scale = 0.576`。

升级和非强制初始化不会修改已有用户配置。现有用户需要从本页完整示例或 `examples/config.toml` 手动合入这些 profile；缺少所选 profile 时，严格通知会按既有 bounded failure 规则失败，不会临时合成 one-off 头像。

## 一次性头像参数

命名 profile 是常用方式：

```console
pingme 'released' --avatar rocket
```

安全列出 profile 名称、类型、description 和 default 标记：

```console
pingme avatar list
pingme avatar list --json
```

列表不会输出 image URL、本地图片路径或字体路径。

也可以通过 mutually-exclusive source argument 临时定义头像：

```console
pingme 'remote' --avatar-url https://example.com/avatar.png
pingme 'local' --avatar-file ./avatar.png --avatar-size 256
pingme 'emoji' --avatar-emoji '🚀' --avatar-background '#5865F2' --avatar-scale 0.72
pingme 'recolored emoji' --avatar-emoji '❌' --avatar-foreground '#FFFFFF' --avatar-background '#DD2E44'
pingme 'text' --avatar-text '告' --avatar-foreground '#FFFFFF' --avatar-background '#ED4245'
pingme 'icon' --avatar-icon U+F0F3 --avatar-font ./fa-solid-900.ttf
```

头像 source 为 `--avatar`、`--avatar-url`、`--avatar-file`、`--avatar-emoji`、`--avatar-text`、`--avatar-icon`。附加样式参数为 `--avatar-background`、`--avatar-foreground`、`--avatar-font`、`--avatar-size`、`--avatar-font-size` 和 `--avatar-scale`；不适用于所选类型的组合会直接报错。
CLI 中 `--avatar-file` 和 `--avatar-font` 的相对路径从当前工作目录解析；profile 中的相对路径仍从 `config.toml` 所在目录解析。

HTTPS 远程图片通过当前消息的 `avatar_url` 发送。本地图片、emoji、文字和 font icon 会渲染为 PNG，并按解析后的 channel ID 与图片摘要创建或复用独立 incoming webhook 身份；这要求配置 Bot token，且 Bot 在目标 channel 具有 `MANAGE_WEBHOOKS`。缺少 channel、token 或权限时，CLI 在发送正常消息前返回错误，不会退回默认头像。

所有层都不指定头像时，payload 不包含 `avatar_url`，并通过基础 webhook 使用 Discord 默认头像。若 state 中存在旧版本修改基础 webhook 时留下的 `avatar_digests` 记录，CLI 会对该基础 webhook 执行一次 `avatar: null` 恢复并删除记录；独立头像 webhook 不影响无头像消息。

## Agent 错误上报

`pingme report-error [--channel <alias-or-id>]` 构造固定的短消息，不读取模板、原错误详情或头像设置。存在 `CODEX_THREAD_ID` 时消息会带该 ID。指定 channel 无法解析或投递失败时，命令只再尝试一次不同的 `[defaults].channel`；默认目标也失败后立即本地返回非零状态，不会递归。

项目内两个 Codex skill 使用各自的安全 runner 包装每次 CLI 调用。普通用户调用不自动开启这项外部副作用。

## Secret 与 state

- `DISCORD_NOTIFICATION_WEBHOOK_URL` 覆盖 `discord.webhook_url`。
- `DISCORD_NOTIFICATION_BOT_TOKEN` 覆盖 `discord.bot_token`。
- Linux state 默认位于 `~/.local/share/discord-notification/state.toml`，使用 owner-only 权限。
- provisioned webhook URL 会按 channel ID 缓存到 state；它和 Bot token 一样应当视为秘密。
- 不要把真实 `config.toml` 提交到版本控制。本仓库的 `.gitignore` 已忽略根目录 `config.toml`。
