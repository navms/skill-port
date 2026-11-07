## 改了什么

<!-- 一段话说明问题与做法；涉及 UI 的附前后截图。 -->

## 怎么验证

- [ ] `pnpm typecheck && pnpm lint && pnpm test`
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] 手动跑过相关路径（macOS / Windows），用的是临时数据目录

## 风险与回滚

<!-- 会影响哪些数据 / 文件 / 权限？出问题怎么撤回？没有就写「无」。 -->
