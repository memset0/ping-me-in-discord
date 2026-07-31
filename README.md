# notify-me-on-discord

一个用 Rust 编写的 Discord 通知 CLI。它把 Markdown 模板渲染成 Discord webhook 消息，并提供两个等价入口：

```console
pingme 'message content'
notify-me-on-discord 'message content'
```

默认情况下，这段文字会作为 `message` 变量传给二进制同目录下的 `templates/defaults.md`，然后发送到配置的 Discord channel。模板、channel、用户名和头像均可在调用时覆盖。

## 安装

Release 提供预编译二进制，最终用户不需要 Rust，也不需要 root。建议先检查安装脚本，再执行：

```console
curl --proto '=https' --tlsv1.2 -fsSL \
  https://raw.githubusercontent.com/memset0/discord-notification/master/install.sh \
  -o /tmp/notify-me-on-discord-install.sh
less /tmp/notify-me-on-discord-install.sh
sh /tmp/notify-me-on-discord-install.sh
```

installer 默认安装到 `~/.local/bin`，同时安装 `notify-me-on-discord` 和 `pingme`。可通过 `DISCORD_NOTIFICATION_INSTALL_DIR` 指定其它用户目录：

```console
DISCORD_NOTIFICATION_INSTALL_DIR="$HOME/bin" sh /tmp/notify-me-on-discord-install.sh
```

如果 `~/.local/bin` 不在 `PATH` 中，需要把它加入 shell 的 `PATH`。installer 本身不会修改 shell 配置。

## 初始化

便携模式会把 `config.toml` 和 `templates/defaults.md` 放在二进制所在目录：

```console
notify-me-on-discord init --portable
```

普通用户模式遵循平台目录约定；Linux 下配置会写到 `~/.config/discord-notification`，运行状态和 emoji cache 位于 `~/.local/share/discord-notification`：

```console
notify-me-on-discord init
```

配置查找优先级如下：

1. `--config /path/to/config.toml`
2. `DISCORD_NOTIFICATION_CONFIG`
3. 二进制同目录下的 `config.toml`
4. 用户配置目录

可用以下命令确认实际路径和离线检查配置：

```console
notify-me-on-discord config path
notify-me-on-discord config validate
```

## Discord 凭据

### 直接使用 webhook URL

这是最简单、权限最小的方式。Discord incoming webhook URL 自身已包含 webhook token，不需要 Bot token：

```toml
[discord]
webhook_url = "https://discord.com/api/webhooks/WEBHOOK_ID/WEBHOOK_TOKEN"
webhook_name = "Notify Me"
```

### 使用 Bot token 自动创建 webhook

也可以配置 Bot token，并用 `[channels]` 给多个 channel ID 设置 alias。Bot 必须在每个目标 channel 拥有 `MANAGE_WEBHOOKS` 权限；首次发送时 CLI 会复用同名 incoming webhook，找不到时创建一个，并按 channel 缓存返回的 webhook URL。

```toml
[discord]
bot_token = "YOUR_BOT_TOKEN"
webhook_name = "Notify Me"

[channels]
alerts = "123456789012345678"
releases = "234567890123456789"

[defaults]
channel = "alerts"
```

随后既可以使用默认 channel，也可以覆盖为 alias 或数字 ID：

```console
pingme 'default destination'
pingme 'release completed' --channel releases
pingme 'one-off destination' --channel 345678901234567890
```

Discord incoming webhook 在执行时不能改投到任意 channel。因此配置了 `--channel`、frontmatter `channel` 或 `[defaults].channel` 时，CLI 使用 Bot 管理该 channel 对应的 webhook；单一 `discord.webhook_url` 只适用于完全不指定 channel 的固定目标用法。

更推荐通过环境变量注入秘密，它们会覆盖文件中的值：

```console
export DISCORD_NOTIFICATION_WEBHOOK_URL='https://discord.com/api/webhooks/...'
# 或
export DISCORD_NOTIFICATION_BOT_TOKEN='...'
```

CLI 不会在正常输出、dry-run 或 API 错误中打印这些秘密。

## 模板

不指定模板时使用 `[defaults].template`，其初始值为 `defaults`，对应 `templates/defaults.md`。新初始化的默认模板是：

```jinja
> **🏠 `{{ runtime.user }}@{{ runtime.hostname }}`   📅 `{{ runtime.timestamp.local }}`**
{{ message }}
```

因此：

```console
pingme 'build completed'
```

会先显示运行 CLI 的 `user@hostname` 和本地时间，下一行紧接 `build completed`。模板正文保留 Discord Markdown 语法，因此 message 自带的粗体、列表、链接和其它 Discord Markdown 不会被转义。

每次渲染都会自动提供一个保留的 `runtime` object：

- `runtime.user` 和 `runtime.hostname`：当前系统身份；读取失败时分别为 `unknown-user` 和 `unknown-host`。
- `runtime.timestamp.local`：运行机器本地时间，格式为 `M/D HH:mm:ss`。
- `runtime.timestamp.unix`：同一时刻的 Unix 秒数。
- `runtime.timestamp.iso8601`：同一时刻的 UTC ISO 8601 表示。

`runtime` 不能由 `--data` 或 `--var` 覆盖；发生冲突时 CLI 会在联网前报错。hostname 可能包含内部基础设施名称，不希望发送时可直接从自己的模板中删除元信息行。installer 和不带 `--force` 的初始化不会覆盖已有 `templates/defaults.md`，所以升级用户需要自行选择是否采用新版模板。

模板可在开头使用 YAML frontmatter：

```markdown
---
username: "{{ project }} Deploy"
avatar: rocket
embeds:
  - title: "Release {{ version }}"
    description: "{{ summary }}"
    color: "#5865F2"
---
Triggered by **{{ actor }}**.
```

发送命名模板：

```console
notify-me-on-discord send \
  --template deployment \
  --var project=API \
  --var version=v1.2.3 \
  --var summary=successful \
  --var actor=CI
```

也可以从 JSON object 读取变量；重复的 `--var` 拥有最高优先级：

```console
notify-me-on-discord send --template deployment --data event.json --var actor=manual
```

查看模板和最终 payload：

```console
notify-me-on-discord templates list
pingme 'preview only' --dry-run
```

dry-run 不会创建 webhook、更新头像或发起 Discord 请求。未定义的模板变量会直接报错。未声明 `allowed_mentions` 时，CLI 默认禁用 mention parsing，避免模板内容意外触发 `@everyone`。

发送选项统一按以下顺序解析：

```text
CLI argument > template frontmatter > config.toml [defaults] > 未设置
```

常用覆盖参数：

```console
pingme 'deploy completed' \
  --channel releases \
  --username 'Deploy Bot' \
  --avatar rocket \
  --no-tts
```

还支持 `--thread-id`、`--thread-name`、`--tts`、`--avatar-url` 以及下面的一次性头像参数。复杂的 embeds、components、poll 和 allowed mentions 继续由模板 frontmatter 管理。

frontmatter、全部头像类型和配置字段详见 [配置参考](docs/configuration.md)。

## 头像

`config.toml` 使用 `[avatars.<name>]` 定义可复用 profile；CLI `--avatar <name>`、模板 `avatar` 和 `[defaults].avatar` 都可以选择它：

- `image`：HTTPS 图片 URL 直接成为当前消息的 `avatar_url`；本地图片会居中裁剪成正方形 PNG。
- `emoji`：下载并缓存透明 Twemoji 图片，再渲染到指定背景色。
- `text`：把汉字、英文字母或短文本居中渲染，前景色和背景色均可配置。
- `font-icon`：从用户提供的 TTF/OTF/TTC 字体中渲染一个 glyph，例如 Font Awesome icon。

预览本地或生成头像：

```console
notify-me-on-discord avatar preview rocket --output rocket.png
```

也可以在单次调用中临时定义头像：

```console
pingme 'rocket launched' --avatar-emoji '🚀' --avatar-background '#5865F2'
pingme 'build completed' --avatar-text '构' --avatar-foreground '#FFFFFF' --avatar-background '#57F287'
pingme 'custom image' --avatar-file ./avatar.png --avatar-size 256
pingme 'remote image' --avatar-url https://example.com/avatar.png
```

一次只能使用 `--avatar`、`--avatar-url`、`--avatar-file`、`--avatar-emoji`、`--avatar-text`、`--avatar-icon` 中的一项。字体图标还需要 `--avatar-font`；可选样式参数包括 `--avatar-foreground`、`--avatar-background`、`--avatar-size`、`--avatar-font-size` 和 `--avatar-scale`。

Discord 的 `avatar_url` 必须是 Discord 能访问的 URL。本地图片、emoji、文字和 font icon 没有公网 URL，因此 CLI 会先用 webhook token 把 PNG 设置为该 webhook 的默认头像，再发送消息。若三层配置都没有指定头像，CLI 使用 Discord 默认头像；如果该 webhook 曾被本 CLI 设置过生成头像，会先将其重置为 `null`。多个主机并发改变同一个 webhook 时仍可能竞争，建议为不同身份使用不同 webhook。远程图片 URL 不受此限制。

## 开发与发布

```console
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --release
```

`vX.Y.Z` tag 会触发 release workflow。Linux x86_64/ARM64 musl archive 是主要产物，另有 GNU/Linux、macOS 和 Windows 构建；每个 archive 都附带 SHA-256 文件。更多信息见 [发布说明](docs/releases.md)。

Emoji artwork attribution 见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
