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
type = "image"
source = "https://example.com/avatar.png"
size = 256

[avatars.local]
type = "image"
source = "avatar.png"
size = 256

[avatars.rocket]
type = "emoji"
emoji = "🚀"
background = "#5865F2"
size = 256
scale = 0.72

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

颜色接受 `#RRGGBB` 或 `#RRGGBBAA`。本地图片和字体相对路径均从 `config.toml` 的目录解析。没有显式 `font` 时，text avatar 会扫描系统字体并选择包含全部 glyph 的字体；找不到时会提示配置兼容字体。

`[channels]` 的 key 是大小写敏感的 alias，value 必须是带引号的 Discord 数字 channel ID。alias 可使用 ASCII 字母、数字、`-` 和 `_`，但不能全部由数字组成。channel selector 可以直接写数字 ID，也可以写 alias：

```console
pingme 'alert' --channel alerts
pingme 'alert' --channel 123456789012345678
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
> **🏠 `{{ runtime.user }}@{{ runtime.hostname }}`   📅 `{{ runtime.timestamp.local }}`**
{{ message }}
```

installer 只替换二进制，不修改模板；不带 `--force` 的初始化也拒绝覆盖已有文件。已有用户可以手动采用上面的模板，而不必变更 `config.toml` 或凭据。

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
notify-me-on-discord send 'deployed' --template release --var version=v1.2.3
notify-me-on-discord send --template alert --data alert.json --var severity=critical
```

## 一次性头像参数

命名 profile 是常用方式：

```console
pingme 'released' --avatar rocket
```

也可以通过 mutually-exclusive source argument 临时定义头像：

```console
pingme 'remote' --avatar-url https://example.com/avatar.png
pingme 'local' --avatar-file ./avatar.png --avatar-size 256
pingme 'emoji' --avatar-emoji '🚀' --avatar-background '#5865F2' --avatar-scale 0.72
pingme 'text' --avatar-text '告' --avatar-foreground '#FFFFFF' --avatar-background '#ED4245'
pingme 'icon' --avatar-icon U+F0F3 --avatar-font ./fa-solid-900.ttf
```

头像 source 为 `--avatar`、`--avatar-url`、`--avatar-file`、`--avatar-emoji`、`--avatar-text`、`--avatar-icon`。附加样式参数为 `--avatar-background`、`--avatar-foreground`、`--avatar-font`、`--avatar-size`、`--avatar-font-size` 和 `--avatar-scale`；不适用于所选类型的组合会直接报错。
CLI 中 `--avatar-file` 和 `--avatar-font` 的相对路径从当前工作目录解析；profile 中的相对路径仍从 `config.toml` 所在目录解析。

所有层都不指定头像时，payload 不包含 `avatar_url`，并使用 Discord webhook 的默认头像。如果 CLI 此前为该 webhook 应用了本地或生成头像，它会先 PATCH `avatar: null` 恢复默认状态，再发送消息。

## Secret 与 state

- `DISCORD_NOTIFICATION_WEBHOOK_URL` 覆盖 `discord.webhook_url`。
- `DISCORD_NOTIFICATION_BOT_TOKEN` 覆盖 `discord.bot_token`。
- Linux state 默认位于 `~/.local/share/discord-notification/state.toml`，使用 owner-only 权限。
- provisioned webhook URL 会按 channel ID 缓存到 state；它和 Bot token 一样应当视为秘密。
- 不要把真实 `config.toml` 提交到版本控制。本仓库的 `.gitignore` 已忽略根目录 `config.toml`。
