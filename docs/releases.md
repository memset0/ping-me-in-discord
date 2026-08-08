# 发布说明

## Release targets

主要 Linux 产物：

- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-musl`

附加产物：

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

每个 archive 使用 `ping-me-in-discord-<tag>-<target>` 作为名称，包含 `notify-me-on-discord`、`pingme`、README 和第三方声明，并带有同名 `.sha256` 文件。

## 创建 release

1. 确保 `Cargo.toml` 中的 package version 是目标版本。
2. 确保以下命令全部通过：

   ```console
   cargo fmt --all -- --check
   cargo check --locked --all-targets
   cargo clippy --locked --all-targets -- -D warnings
   cargo test --locked --all-targets
   openspec validate --all --strict --no-interactive
   ```

3. 创建与 Cargo version 一致的 tag，例如 `v0.1.0`，并 push tag。
4. `.github/workflows/release.yml` 会先重复 quality gate，再构建、打包、计算 SHA-256 并创建 GitHub Release。

musl x86_64 和 ARM64 失败会阻止发布；GNU/Linux、macOS 和 Windows 是 best-effort 附加构建。

## Installer

`install.sh`：

- 自动检测 Linux/macOS 和 x86_64/ARM64。
- Linux 默认选择 musl archive。
- 从 GitHub latest release 或 `DISCORD_NOTIFICATION_VERSION` 下载。
- 在替换现有二进制前验证 SHA-256。
- 默认写入 `~/.local/bin`，从不调用 `sudo`。
- 只替换两个二进制，不修改 `config.toml`、`templates/` 或用户 data。

可用于测试 target detection：

```console
sh install.sh --print-target Linux x86_64
sh install.sh --print-target Linux aarch64
sh install.sh --print-target Darwin arm64
```
