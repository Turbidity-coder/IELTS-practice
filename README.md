# IELTS Atlas

IELTS Atlas 是基于 **Tauri 2 + Rust + Vue + SQLite** 的本地优先 IELTS 学习桌面应用。当前 shipping 产品入口是 `apps/writing-vue`，由 `src-tauri` 承载，业务数据由 `crates/ielts-db` 持久化。

## 开发前置

Tauri 编译依赖一个冻结的 Python agent runtime sidecar。clean clone 或 GitHub source ZIP 必须先构建它，否则 `src-tauri/build.rs` 会因找不到 `src-tauri/binaries/ielts-agent-runtime-<target>[.exe]` 而直接 panic。

```bash
# 1. 安装 Python 3.12 与冻结依赖
python -m pip install -r agent-runtime-python/requirements-build.lock
python -m pip install -r agent-runtime-python/requirements.lock

# 2. 构建并 smoke 冻结 sidecar（Windows x64 为默认 target）
python developer/tests/ci/build_agent_runtime_sidecar.py
```

非 Windows 目标需显式指定 `--target`，例如 macOS arm64：

```bash
python developer/tests/ci/build_agent_runtime_sidecar.py --target aarch64-apple-darwin
```

构建产物包含可执行文件和 `.sha256` manifest，`src-tauri/build.rs` 会在每次 Tauri 编译前校验二者。

## 开发

```bash
npm run prepare:writing
npm run build:writing
cargo tauri dev
```

常用检查：

```bash
cargo check --workspace --locked
cargo test --workspace --locked
python developer/tests/ci/run_static_suite.py
python developer/tests/e2e/suite_practice_flow.py
```

以上任一 Tauri 编译或 Rust 测试命令都要求 sidecar 已按「开发前置」构建完成。

## 目录

- `apps/writing-vue/`：阅读、写作和 Agent 工作台 Vue 界面。
- `src-tauri/`：Tauri commands、AI runtime、权限和桌面宿主。
- `crates/ielts-domain/`：领域合同和序列化类型。
- `crates/ielts-db/`：SQLite schema、迁移、业务查询和学习事件账本。
- `assets/resource-pack/`：运行时题目资源包。
- `developer/tests/ci/`、`developer/tests/e2e/`：当前静态门禁和打包 E2E。

## 文档

- [Agent 自进化总任务书](developer/docs/IELTS_Atlas_Agent_Self_Evolution_Engineering_Plan.md)
- [Tauri Phase 10 发布运行手册](developer/docs/phase10-release-runbook.md)
- [架构 ADR](docs/architecture/)
- [M0 评估](docs/evaluations/agent-m0-baseline-eval.md)
- [Tauri cutover 迁移记录](docs/rewrite/phase10-cutover.md)

总任务书是 Agent 自进化路线的唯一权威计划。`docs/rewrite/` 中的 Phase 文档仅作为迁移历史和数据合同记录，不代表当前开发状态。

## 产品边界

Electron、Fastify、根目录静态 HTML 和旧 `js/` 运行时已经从 shipping 树移除。当前前端通过 Tauri invoke 调用 Rust commands；SQLite 是练习、历史、评估、Coach 和 Agent 运行数据的持久化真相源。

## 许可证

代码采用 GNU GPL v3，详见 [LICENSE](LICENSE)。题库内容仅供学习使用，版权归原作者所有。
