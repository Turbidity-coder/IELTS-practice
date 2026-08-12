# IELTS Atlas Agent 辅助、自进化与长期个性化系统

## Deep Research 与工程落地总计划

- **文档版本**：1.0
- **日期**：2026-08-10
- **目标分支**：`IELTS-WRITING-FEAT`
- **当前分支基线**：`5c9fd7c6e9d89cc2fd4f7b4ef4cb34f71335c9ce`
- **核心后端基线**：`93e4ed4bbf80105876af5c6830f9c7ad9748b9c2`
- **文档性质**：架构设计、产品机制设计、数据设计、接口设计、分阶段工程任务书、测试与验收规范
- **产品定位**：本地优先、证据驱动、Agent 辅助的 IELTS 学习桌面产品

---

## 目录

1. [执行摘要](#1-执行摘要)
2. [范围、非目标与核心术语](#2-范围非目标与核心术语)
3. [当前分支最新实现审计](#3-当前分支最新实现审计)
4. [发散调研：行业与研究领域的可复用模式](#4-发散调研行业与研究领域的可复用模式)
5. [收敛决策：IELTS Atlas 应采用的总体模型](#5-收敛决策ielts-atlas-应采用的总体模型)
6. [目标架构](#6-目标架构)
7. [Soul、User、Memory、Diary、Skill 的边界](#7-soulusermemorydiaryskill-的边界)
8. [数据架构与 SQLite Schema 设计](#8-数据架构与-sqlite-schema-设计)
9. [学习事件账本与证据模型](#9-学习事件账本与证据模型)
10. [长期记忆生命周期](#10-长期记忆生命周期)
11. [Daily Journal 与 Dream 离线整合机制](#11-daily-journal-与-dream-离线整合机制)
12. [学习者模型与重复练习分析](#12-学习者模型与重复练习分析)
13. [Context Engineering 与上下文编译器](#13-context-engineering-与上下文编译器)
14. [Agent Runtime、状态、工具与权限](#14-agent-runtime状态工具与权限)
15. [AI Coach 个性化与教学策略演化](#15-ai-coach-个性化与教学策略演化)
16. [产品级 Prompt、Skill 与工具描述自进化](#16-产品级-promptskill-与工具描述自进化)
17. [Rust Application API 与 Tauri 接口设计](#17-rust-application-api-与-tauri-接口设计)
18. [Vue 产品界面与交互设计](#18-vue-产品界面与交互设计)
19. [安全、隐私与记忆投毒防护](#19-安全隐私与记忆投毒防护)
20. [评测体系、指标与发布门禁](#20-评测体系指标与发布门禁)
21. [逐阶段工程实施计划](#21-逐阶段工程实施计划)
22. [建议目录结构](#22-建议目录结构)
23. [关键伪代码](#23-关键伪代码)
24. [风险清单与反模式](#24-风险清单与反模式)
25. [最终验收标准](#25-最终验收标准)
26. [参考资料](#26-参考资料)

---

# 1. 执行摘要

## 1.1 最终产品不应只是“带工具调用的聊天框”

IELTS Atlas 的核心价值不应停留在：

- 用户在 Agent 工作台输入一句话；
- Agent 调用若干工具；
- Agent 返回一段回答；
- 对话结束后系统只留下 transcript。

真正有价值的产品闭环应是：

```text
用户练习、写作、复盘、提问、修正 Agent
                    ↓
形成可验证的学习事件和交互证据
                    ↓
日内快速记录：Diary / Candidate Memory
                    ↓
夜间或空闲期离线整合：Dream / Reflection
                    ↓
更新可解释的学习者模型、用户画像和教学策略
                    ↓
下一次请求时按任务动态编译最小高价值上下文
                    ↓
Agent 的讲解、练习建议和工具选择更贴近用户
                    ↓
通过后续练习结果验证“是否真的更好”
```

因此，本计划将系统拆成三种不同的“自进化”：

### A. 用户记忆演化

目标是让系统越来越准确地理解：

- 用户稳定偏好；
- 用户当前目标；
- 用户在哪些题型、技能和语言点上存在稳定困难；
- 哪些讲解方式对该用户有效；
- 哪些旧判断已被新证据推翻。

它是**用户级、可查看、可编辑、可撤销**的派生数据演化。

### B. 个性化教学策略演化

目标是让系统逐渐掌握：

- 对该用户应优先使用例证、反例、逐步推理还是直接结论；
- 阅读错题复盘中应强调原文证据、干扰项、定位策略还是时间分配；
- 写作反馈应先给结构、句法、词汇还是任务回应；
- 哪些建议在后续练习中确实产生了正向效果。

它是**用户级程序性记忆**，必须有学习结果证据，不能只根据用户一次“感觉不好”就永久改变。

### C. 产品 Prompt、Skill 与工具描述演化

目标是改进全体用户使用的：

- System Prompt 模块；
- Coach Prompt；
- Memory Extractor Prompt；
- Dream Consolidator Prompt；
- Tool Description；
- 可复用 Skill。

它必须是**开发者控制的离线优化流程**：候选生成、离线评测、保留集、影子运行、人工审批、版本发布、可回滚。线上 Agent 不得直接修改当前生效的核心 Soul 或全局 Prompt。

---

## 1.2 十二条不可破坏的架构原则

1. **练习事实与 Agent 推断分离**
   `attempts`、答案、分数、耗时、Coach 原始消息等是事实；Memory、画像和趋势是派生结论。

2. **SQLite 中的学习事实默认为只读证据**
   Agent 可读取和分析，但不得篡改历史得分、答案或原始练习记录。

3. **Soul 是产品政策，不是可自由学习的用户记忆**
   Agent 不能因某次对话自行改写“自己是谁、允许做什么、禁止做什么”。

4. **显式用户偏好与模型推断画像分表存储**
   “用户明确说喜欢逐步讲解”和“系统推断用户可能喜欢逐步讲解”必须有不同可信等级。

5. **原始事件追加，长期记忆有界**
   Diary 可追加；Active Memory 必须有容量、去重、合并、替代、过期和归档机制。

6. **记忆不能无来源**
   每条自动记忆都必须链接到 attempt、question、Coach message、Agent run 或用户显式输入。

7. **冲突通过 supersede 解决，不通过静默覆盖解决**
   新结论替代旧结论时保留审计链；UI 应能解释何时、因何改变。

8. **上下文按需编译，不把全部历史塞入 Prompt**
   Context 是有限资源，应优先放入当前任务证据、明确偏好和高置信记忆。

9. **先建立只读学习工具，再建立写工具**
   第一阶段 Agent 只读学习数据；任何计划、词汇、复习队列等写入均需明确权限和可撤销性。

10. **自进化必须先评测后生效**
    不允许生产 Agent 通过“我觉得这次表现不错”直接提升自己的 Prompt 或 Skill。

11. **本地优先与最小披露**
    数据存 SQLite；发送给远程模型的内容必须是当前任务所需的最小 Context Pack。

12. **逐模块纵向切片，不做全栈同时重写**
    每个阶段都必须能独立发布、回滚和量化收益。

---

## 1.3 推荐的最终系统形态

```mermaid
flowchart TB
    UI[Vue 产品界面] --> Tauri[Tauri Commands / Events]
    Tauri --> App[ielts-application Use Cases]

    App --> Learning[Learning Domain Services]
    App --> Agent[Agent Runtime]
    App --> Memory[Memory & Context Services]
    App --> Dream[Background Dream Services]
    App --> Eval[Evolution & Evaluation Services]

    Learning --> DB[(SQLite Learning Truth)]
    Memory --> DB
    Dream --> DB
    Eval --> DB

    Agent --> Context[Context Compiler]
    Agent --> Tools[Tool Registry + Guardrails]
    Agent --> Model[LLM Gateway]

    Context --> DB
    Tools --> Learning
    Tools --> Memory
    Model --> Provider[OpenAI-compatible / Future Providers]

    Scheduler[SQLite-backed Job Scheduler] --> Dream
    Scheduler --> Eval
    Scheduler --> Memory
```

推荐继续维持 **Rust modular monolith**，不引入外部 Agent 框架作为运行时依赖。LangGraph、LangMem、OpenAI Agents SDK、Hermes、OpenClaw 等只作为设计参考；核心运行逻辑仍由现有 Rust Application 层、SQLite 和 Tauri 适配器控制。

---

# 2. 范围、非目标与核心术语

## 2.1 本计划包含

- 长期记忆分层；
- User Profile 与 Learner Model；
- Daily Journal；
- Dream / Offline Consolidation；
- Context Compiler；
- Agent 学习数据只读工具；
- Agent run、thread、checkpoint 与后台 job；
- Coach 个性化；
- 学习策略演化；
- Prompt/Skill 自进化评测管线；
- Memory Center、Dream Report、Profile 等 UI；
- SQLite schema、Rust trait、Tauri command 和 Vue repository 设计；
- 安全、隐私、评测和发布门禁。

## 2.2 本计划当前不包含

- 多 Agent 社会或角色群；
- 自主联网替用户执行高风险行为；
- 模型权重在线训练；
- 未经评测自动修改 Rust 代码；
- 直接复刻 Claude Code、OpenClaw、Hermes 或 WorkBuddy；
- 与 `opensource` 分支的数据或接口兼容；
- 将全部题库正文强制存入数据库；
- 第一阶段即引入图数据库、独立向量数据库或云端记忆服务。

## 2.3 术语定义

| 术语 | 本文中的准确含义 |
|---|---|
| Learning Truth | 练习、答案、分数、耗时、原始对话等不可由 Agent 改写的事实数据 |
| Observation | 从事实中提取的单条观察，例如“用户在三次 Matching Headings 中均受干扰项影响” |
| Memory Candidate | 尚未成为长期记忆的候选结论 |
| Semantic Memory | 关于用户、学习状态或领域的相对稳定事实 |
| Episodic Memory | 一次具体经历及其结果，例如某种讲解方式在某次复盘中有效 |
| Procedural Memory | “如何对该用户教学”或“如何完成某类任务”的策略 |
| Diary / Journal | 以日期为单位、较详细、允许冗余的工作层记录 |
| Dream | 在后台跨会话聚合证据、压缩、冲突解析和生成候选长期记忆的过程 |
| Soul | 产品定义的 Agent 身份、价值边界、安全规则和不可越权原则 |
| User Profile | 用户明确提供或系统推断的稳定偏好、背景、目标；显式与推断必须分离 |
| Learner Model | 对技能掌握、错误类型、稳定性、遗忘和复习需求的量化模型 |
| Context Pack | 每次模型调用前按预算动态选出的最小高价值上下文 |
| Self-Evolution | 通过证据、评测、候选版本和发布门禁改善记忆、策略或 Prompt，而非无约束自改 |

---

# 3. 当前分支最新实现审计

## 3.1 最新提交关系

当前分支 tip 为：

- `5c9fd7c6e9d89cc2fd4f7b4ef4cb34f71335c9ce`
- Commit：`feat: migrate opensource visual continuity to Vue`

其父提交：

- `93e4ed4bbf80105876af5c6830f9c7ad9748b9c2`
- Commit：`refactor: add application and agent backend layers`

这两个提交共同构成当前 Agent 基线：父提交建立后端、application ports、Agent loop 和工具审计；tip 提交增加 Agent 工作台和 Vue 视觉层。

## 3.2 当前已经具备的正确基础

### 3.2.1 Application 层已经建立

`crates/ielts-application` 已经存在，并包含：

```text
agent.rs
coach.rs
error.rs
lib.rs
ports.rs
writing_evaluation.rs
```

它不直接依赖 Tauri、Keyring 或原始 SQLite 连接，而通过 port trait 表达：

- `LanguageModel`
- `AgentModel`
- `WritingEvaluationStore`
- `CoachStore`
- `AgentStore`
- `AgentToolExecutor`
- `EventSink`

这是后续 Memory、Dream 和 Learner Model 最合适的扩展位置。

### 3.2.2 已有真实的 Agent loop

当前 `AgentService::run` 已具备：

- System/User/Assistant/ToolResult 消息；
- 模型原生 tool call；
- 工具定义；
- 多轮执行；
- 最大轮数与最大工具调用数；
- token usage 聚合；
- 每次 tool call 的 begin/finish 审计；
- Agent run begin/finish；
- 无效、空 ID、重复 tool call ID 防护；
- 最终回答或限制终止。

默认限制为有限轮数和有限 tool call 数，这一思路应保留，并进一步扩展为不同 run type 的策略配置。

### 3.2.3 Agent run 和 tool call 已落 SQLite

当前 migration `0011_agent_runs_tool_calls.sql` 已建立：

- `agent_runs`
- `agent_tool_calls`

并记录：

- provider、model；
- status；
- rounds；
- result/error；
- tool arguments/result/error；
- 运行和工具调用时间。

重启恢复会把未完成的运行标记为 interrupted。该审计层不应被新 Memory 系统替代，而应扩展为所有 Agent、Dream、Evolution 运行的 trace 主链。

### 3.2.4 LLM runtime 已从 commands 中抽离

`src-tauri/src/ai/runtime.rs` 同时实现：

- 普通结构化 completion；
- Agent 原生 tools 协议；
- usage、latency、provider request ID；
- OpenAI-compatible 请求；
- 限定重试。

这已经比此前“AI 编排直接属于 Tauri command”成熟得多。

### 3.2.5 当前文件工具安全性较好

现有 workspace Agent 具备：

- 15 分钟临时 workspace grant；
- 进程内授权；
- canonical path containment；
- `read_file`、`write_file`、`replace_in_file`；
- 1 MiB 文件限制；
- UTF-8 限制；
- SHA-256 乐观并发控制；
- 原子写；
- 相对路径和敏感路径拦截；
- 工具参数最小审计。

这些模式可直接复用于未来的学习工具权限框架。

## 3.3 当前 Agent 工作台的真实定位

当前 `AgentWorkspacePage.vue` 仍然主要是 UI 原型：

- 文件列表是固定演示数据；
- 运行通过 `setTimeout` 模拟；
- 输出不是由 `agent_run` 返回；
- 未展示真实 tool call；
- 未展示 trace、memory、context 或 approval；
- 未承担学习领域核心流程。

因此，应保留其视觉和交互骨架，但不能把它视为 Agent 产品主架构已经完成。

## 3.4 当前缺口矩阵

| 能力 | 当前状态 | 结论 |
|---|---|---|
| Application ports | 已有 | 保留并扩展 |
| LLM tool protocol | 已有 | 保留 |
| 通用 Agent loop | 已有 | 扩展 run type、checkpoint 和权限 |
| Workspace 文件工具 | 已有 | 保留为独立 workspace Agent |
| 学习领域只读工具 | 缺失 | 第一优先级 |
| Agent conversation thread | 不完整 | `agent_runs` 是执行审计，不等于长期对话线程 |
| Learning Event Ledger | 缺失 | 必须增加 |
| User Profile | 缺失 | 必须显式/推断分离 |
| Long-term Memory | 缺失 | 必须增加类型、证据与生命周期 |
| Daily Journal | 缺失 | 必须增加 |
| Dream scheduler | 缺失 | 必须增加 SQLite job + 后台执行 |
| Context Compiler | 缺失 | 必须增加，不能由 Vue 拼 Prompt |
| Learner Model | 缺失 | 必须增加可解释模型 |
| Repeated Attempt Analysis | 缺失统一服务 | 必须增加 |
| Teaching Strategy Memory | 缺失 | 中后期增加 |
| Prompt/Skill Registry | 部分 Prompt 表存在 | 扩展为统一版本与评测模型 |
| Evolution Eval Harness | 缺失 | 自进化前置条件 |
| Memory Security | 缺失 | 必须在自动写入前完成 |
| Memory UI | 缺失 | 用户信任所必需 |

---

# 4. 发散调研：行业与研究领域的可复用模式

## 4.1 Claude Code / Anthropic：Context 是有限资源

Anthropic 的 Context Engineering 核心结论不是“尽可能多地提供上下文”，而是：

> 选择能够最大化任务成功概率的最小高信号 token 集合。

可复用模式：

- System Prompt 保持适当抽象高度；
- 工具说明要紧凑且准确；
- 采用 just-in-time retrieval；
- 长任务使用 compaction、结构化 memory 和 handoff artifact；
- Agent 评测必须包含 task、trial、grader、trace 和真实 outcome；
- 生成者与评估者应尽量分离；
- 工具返回和持久化 memory 都是潜在注入面。

对 IELTS Atlas 的启示：

- 不把完整练习历史塞进 Coach Prompt；
- Context Compiler 必须可审计；
- 评测不能只看最终文字，应看后续学习结果、工具行为和数据库 outcome；
- Memory 写入前需要安全扫描和 provenance。

## 4.2 Claude Code Memory：人类规则与自动记忆分离

Claude Code 的 `CLAUDE.md` 更接近人类显式维护的规则、命令和项目约定，而自动 memory 用于模型发现的经验。两者用途不同。

对 IELTS Atlas 的映射：

- `Soul` / Product Policy：开发者显式规则；
- `Explicit User Preferences`：用户显式规则；
- `Inferred Memory`：系统基于证据生成；
- 不允许把三者混成一张自由文本表。

## 4.3 OpenClaw：Daily、Memory、User、Dream 四层

OpenClaw 的公开设计将：

- `USER.md`：用户模型；
- `MEMORY.md`：精炼长期记忆；
- `memory/YYYY-MM-DD.md`：每日工作层；
- `DREAMS.md`：梦境和整合报告；

分开管理。Daily 不会全部注入每次 Prompt；长期 Memory 有上下文预算；Dreaming 通过 light、REM、deep 阶段整合，并只有 deep 阶段写入长期记忆。

最值得采用的不是 Markdown 文件格式，而是：

1. 工作层与精选层分离；
2. 自动整合有阶段；
3. 长期层有容量；
4. 用户可以审阅 Dream 报告；
5. promotion 前重读原始证据，避免从过期索引提升。

## 4.4 Hermes：有界 Memory、冻结快照、FTS5 历史搜索

Hermes 采用小型、强约束、始终注入的 `MEMORY.md` 和 `USER.md`，同时把完整 Session 存 SQLite 并使用 FTS5 按需搜索。

关键模式：

- Active Memory 必须有严格容量；
- 写满时要求合并或替代，而不是无限追加；
- memory snapshot 在会话开始冻结，避免中途热替换导致缓存和行为不稳定；
- 完整历史不进入 Active Memory，而通过 FTS5 查询；
- memory 条目有重复防护和注入安全扫描；
- 外部 memory provider 是增强层，不替代本地核心 memory。

对 IELTS Atlas 的建议：

- SQLite 为 canonical store；
- Active Context Profile 是运行时快照；
- Coach 历史使用 FTS5 检索；
- 不需要一开始引入外部 vector DB。

## 4.5 WorkBuddy：夜间更新、用户可管理的画像

WorkBuddy 官方文档说明：

- 从会话提取事实、偏好、关系和跟进项；
- 每晚整理当天会话；
- 记忆摘要会重新生成；
- 用户可以查看、编辑、删除和关闭。

这一产品模式说明：长期个性化不是隐藏的内部状态，而是一项用户可管理的产品能力。

对 IELTS Atlas 的建议：

- Memory Center 必须和后台 Dream 同期建设；
- 用户可关闭某类自动学习；
- 用户可更正错误画像；
- 日更应是“重新整理有效集合”，而不是继续在尾部追加。

## 4.6 memU：Agent 负责判断，存储层负责可读、索引和检索

memU 当前强调：

- Agent 从 session/tool history 决定是否创建、修改或不写 memory/skill；
- Memory service 本身不调用 LLM；
- 可读 Markdown 是 Agent-facing artifact；
- SQLite/Postgres 是 durable store；
- segment embedding 用于检索，最终返回完整可读文件；
- memory 关联来源，可跨 Agent 使用。

对 IELTS Atlas 的建议：

- 不把 Memory 写入逻辑藏在不透明向量服务里；
- Memory 应能导出为可读 Markdown，但 canonical 仍在 SQLite；
- Dream Agent 产出 proposal，Memory Store 只执行经过验证的 mutation；
- 长期可加入 skill track，但第一阶段只做 memory track。

## 4.7 Auto-Dreamer：在线快速记录，离线跨会话整合

Auto-Dreamer 提出将：

- fast online acquisition；
- slow offline consolidation；

分离。整合器读取一个有类型的 memory 区域和来源轨迹，以只读证据为基础，生成一个新的紧凑替代集合，并 supersede 原始区域。

本计划采用其最重要的工程思想：

- Dream 的输入证据只读；
- Dream 不能就地逐条任意改写；
- Dream 生成 replacement proposal set；
- mutation 通过确定性校验器；
- 原 memory 通过 supersession 保留追踪链。

注意：Auto-Dreamer 是 2026 年预印本，应采用其架构思想，而不是把其实验结果直接当作生产保证。

## 4.8 Generative Agents：Observation、Reflection、Retrieval、Planning

Generative Agents 将完整经历记录为 memory stream，周期性生成更高层 reflection，并基于 recency、relevance 和 importance 动态检索。

可复用点：

- Raw observation 与 high-level reflection 分层；
- 反思可由累计重要度而非固定时间单独触发；
- Reflection 仍是 memory，并有来源；
- 反思结果只在相关任务中检索。

对 IELTS Atlas 可增加两个触发条件：

```text
固定时间触发：每日、每周
证据阈值触发：累计重要学习事件超过阈值
```

## 4.9 LangMem / LangGraph：语义、情景、程序性记忆

LangMem 和 LangGraph 区分：

- Semantic memory：事实；
- Episodic memory：经历与有效方法；
- Procedural memory：规则和系统指令；
- Hot-path formation：当前交互内写入；
- Background formation：后台提取、合并和更新；
- Thread checkpoint：当前会话状态；
- Long-term store：跨 thread 数据。

这非常适合 IELTS Atlas：

| 类型 | IELTS 示例 |
|---|---|
| Semantic | 用户长期在 Matching Headings 上不稳定 |
| Episodic | 用“先排除主题范围不匹配的 heading”讲解后，用户下一次同类题正确 |
| Procedural | 对该用户讲 Heading 时先让其复述段落主旨，再看选项 |
| Thread checkpoint | 当前 Coach 对话和工具调用进度 |
| Long-term store | 用户画像、技能状态、长期记忆 |

## 4.10 Letta：Always-visible Memory Block 必须有边界

Letta 的 memory block 是始终注入的结构化区块，并支持 read-only block。

对 IELTS Atlas 的启示：

- Soul、用户明确偏好和少量核心教学策略可以是 always-visible block；
- 大量题目历史和细节绝不能做 always-visible block；
- 每个 block 需要明确 label、description、字符/token 上限和写权限。

## 4.11 OpenAI Agents SDK：Session、Guardrail、Tracing

OpenAI Agents SDK 的可借鉴工程模式包括：

- Session history 与长期 memory 分离；
- SQLite 可作为本地 session backend；
- session input callback 可在模型调用前裁剪历史；
- tool guardrail 在执行前和执行后校验；
- 高风险工具支持 approval；
- trace 覆盖整个 run、model call、tool call 和 guardrail。

当前 Rust Agent loop 已有良好起点，但需要补充：

- thread/session；
- context snapshot；
- tool input/output guardrail；
- approval state；
- model invocation trace；
- run cancellation 与 checkpoint。

## 4.12 Hermes Self-Evolution / GEPA：优化必须在生产运行之外

Hermes Self-Evolution 的正确模式是：

```text
读取当前 Skill/Prompt
        ↓
生成或整理 Eval Dataset
        ↓
从真实 execution trace 分析失败原因
        ↓
生成候选版本
        ↓
训练集 / 验证集 / Holdout 评测
        ↓
尺寸、语义、测试、缓存等约束
        ↓
人工 Review / PR
        ↓
新会话生效
```

同时，相关开源项目已经出现版本兼容和优化目标没有真正被 mutation 的公开问题。这说明不应直接把一个第三方优化器当作黑盒生产能力。

对 IELTS Atlas 的建议：

- 自进化管线放在 `developer/evolution/`；
- Rust 生产运行时只加载已发布版本；
- GEPA 是可选候选生成器，不是架构核心；
- 所有候选必须经过本项目自己的 evaluator 和 holdout；
- 不允许 mid-session 热替换。

## 4.13 学习科学：记忆系统必须优化真实学习，而不是只优化满意度

检索练习和间隔练习研究表明：

- 主动回忆比单纯重复阅读更有利于延迟保持；
- 最佳间隔与目标保持周期相关；
- 学习者常高估重复阅读带来的掌握程度；
- 系统应帮助用户识别“以为会”与“真正能提取”之间的差异。

因此，Agent 个性化不应只追求：

- 回答更像用户喜欢的风格；
- 用户短期更满意；

还应追求：

- 后续同类新题表现改善；
- 间隔后仍能保持；
- 解释减少错误而不是泄露答案；
- 用户的自我判断更准确。

## 4.14 知识追踪：先可解释，再深度模型

Knowledge Tracing 的目标是估计学习者随时间变化的知识状态。深度模型可获得较强预测能力，但需要足够大、稳定和正确标注的数据，同时可解释性较弱。

IELTS Atlas 初期应采用：

- Beta-Bernoulli / EWMA；
- 时间衰减；
- 跨题目证据多样性；
- 重复同题降权；
- 显式错误 taxonomy；
- 置信区间或 uncertainty；

而不是立即实现 DKT。只有在匿名化、用户同意且数据量足够后，才评估更复杂模型。

## 4.15 调研收敛矩阵

| 行业模式 | 采用 | 暂缓 | 拒绝 |
|---|---:|---:|---:|
| 有界 Active Memory | ✅ |  |  |
| Daily 工作层 + Long-term 精选层 | ✅ |  |  |
| Background Dream | ✅ |  |  |
| USER / SOUL / MEMORY 分离 | ✅ |  |  |
| SQLite + FTS5 | ✅ |  |  |
| Hybrid retrieval | ✅，先 FTS5 后 embedding |  |  |
| Temporal knowledge graph |  | ✅ |  |
| 独立向量数据库 |  | ✅ |  |
| 多 Agent 群体 |  | ✅ |  |
| 在线 Agent 直接改 System Prompt |  |  | ❌ |
| 在线 Agent 直接改代码并发布 |  |  | ❌ |
| Memory 无来源自由写入 |  |  | ❌ |
| 全历史每轮注入 |  |  | ❌ |
| 用户无法查看和删除画像 |  |  | ❌ |
| 只用满意度作为自进化奖励 |  |  | ❌ |


---

# 5. 收敛决策：IELTS Atlas 应采用的总体模型

## 5.1 产品核心不是“通用 Agent”，而是“学习证据操作系统”

推荐将产品定义为：

> Agent 负责理解目标、选择工具、组织解释和提出行动；SQLite 中的学习事实、Memory 系统和 Learner Model 负责让 Agent 对用户形成连续、可验证且可治理的理解。

产品价值排序应为：

1. **准确记录学习事实**；
2. **从跨时间证据中识别变化**；
3. **为当前任务检索最相关的用户上下文**；
4. **用适合该用户的方式解释和建议**；
5. **验证建议是否在未来产生学习收益**；
6. **在评测门禁下改善策略与 Prompt**。

Agent 工作台是一个入口，但不应成为所有智能能力唯一入口。大量个性化应在阅读页、写作页、历史页和每日总结中自然发生。

## 5.2 四条相互独立的数据链

### 5.2.1 Learning Truth Chain

```text
用户行为
  → attempts / answers / evaluations / annotations / coach_messages
  → learning_events
  → learner_skill_observations
```

规则：只追加、可纠错但不可被 Agent 静默修改。

### 5.2.2 Memory Chain

```text
learning_events / messages
  → memory candidates
  → journal
  → dream consolidation
  → active semantic / episodic / procedural memory
  → supersession / archive
```

规则：所有结论有来源、置信度和生命周期。

### 5.2.3 Context Chain

```text
当前请求
  → task classifier
  → retrieval query plan
  → memory + learner state + current evidence retrieval
  → ranking / dedup / budget packing
  → Context Pack
  → model call
```

规则：Context Pack 是每次运行的可审计产物。

### 5.2.4 Evolution Chain

```text
生产 traces + 用户反馈 + 后续学习结果
  → eval dataset candidates
  → baseline / candidate execution
  → deterministic + LLM + human graders
  → shadow / canary
  → prompt or skill version promotion
```

规则：不得从生产 trace 直接跳到线上生效版本。

## 5.3 三种不同的时间尺度

| 时间尺度 | 机制 | 目标 |
|---|---|---|
| 实时 / 单次请求 | Context Compiler、Agent tools | 完成当前学习任务 |
| 会话结束 / 当日 | candidate extraction、daily journal | 不丢失有价值的新证据 |
| 日 / 周 / 月 | dream、reflection、compaction、eval | 跨会话抽象、删除冗余、验证策略 |

## 5.4 本地优先的技术选择

初期推荐：

- Canonical store：现有 SQLite；
- Full-text：SQLite FTS5；
- Background jobs：SQLite queue + Tauri/Tokio worker；
- Embedding：不作为第一阶段硬依赖；
- Vector index：后续可选，优先内嵌 SQLite 方案；
- Graph：只有当多跳关系检索被评测证明明显不足时再引入；
- Export：Memory 和 Diary 可导出为 Markdown；
- Remote LLM：只接收编译后的最小 Context Pack；
- Scheduler：应用启动、空闲、指定时间窗口触发，不依赖云端常驻服务。

## 5.5 关于“SQL 是只读还是 Agent 可写”

应避免把“整个 SQL 都只读”理解为技术限制。正确做法是按数据类别授权：

| 数据 | Agent 权限 |
|---|---|
| 原始练习记录、分数、答案 | 只读 |
| 用户显式偏好 | 通过专用工具写入，用户可编辑 |
| Memory candidate | Agent 可提议，不直接生效 |
| Active inferred memory | Dream service 按门禁变更 |
| Diary | 后台服务可写，用户可编辑/删除 |
| 学习计划 | 初期提议；后期用户批准后写入 |
| 词汇收藏 | 用户确认或低风险明确动作后写入 |
| Soul / 安全政策 | 生产 Agent 只读 |
| Prompt/Skill active version | 生产 Agent 只读；开发管线发布 |

---

# 6. 目标架构

## 6.1 建议的 Rust 模块边界

```text
crates/
  ielts-domain/
    src/
      agent/
      memory/
      learner/
      prompt/
      learning_event/

  ielts-application/
    src/
      agent/
        run_service.rs
        thread_service.rs
        approval_service.rs
      context/
        compiler.rs
        ranking.rs
        budget.rs
      memory/
        extraction.rs
        mutation.rs
        retrieval.rs
      dream/
        daily.rs
        weekly.rs
        compaction.rs
      learner/
        observation.rs
        mastery.rs
        repeated_attempt.rs
      evolution/
        candidate.rs
        evaluation.rs
        promotion.rs
      ports.rs

  ielts-db/
    src/
      learning_events/
      agent/
      memory/
      dream/
      learner/
      prompts/
      jobs/
      search/

src-tauri/
  src/
    ai/
    agent/
      learning_tools.rs
      memory_tools.rs
      action_tools.rs
      tool_guardrails.rs
    background/
      scheduler.rs
      worker.rs
      recovery.rs
    app/
      application_store.rs
      memory_store.rs
      learner_store.rs
      job_store.rs
    commands/
      agent.rs
      memory.rs
      profile.rs
      journal.rs
      dream.rs
      insights.rs

apps/writing-vue/src/
  modules/
    agent/
    memory/
    learner-profile/
    dream-report/
    study-plan/
```

这里不要求一次性创建所有目录。目录结构是最终收敛目标，必须按第 21 节的里程碑逐步增加。

## 6.2 核心服务

### 6.2.1 LearningEventService

职责：

- 从已提交 attempt、Coach message、Agent run、用户反馈生成标准事件；
- 确保 idempotency；
- 规范化 question、skill、asset、时间和来源；
- 不做高层推断。

### 6.2.2 LearnerModelService

职责：

- 把 learning event 映射为 skill observation；
- 更新技能状态；
- 分析同题重复、跨题型重复和时间间隔；
- 输出 uncertainty 和证据解释。

### 6.2.3 MemoryExtractionService

职责：

- 从一次 session 或事件窗口提取候选 memory；
- 区分显式偏好、推断事实、具体 episode、教学策略候选；
- 只创建 candidate，不越权提升。

### 6.2.4 DreamService

职责：

- 读取一个固定时间窗内未整合事件；
- 读取相关 Active Memory；
- 生成 mutation proposal；
- 执行去重、冲突、证据、安全和容量校验；
- 低风险自动提升或进入用户审阅；
- 生成 Daily/Weekly Dream Report。

### 6.2.5 ContextCompiler

职责：

- 决定当前请求需要哪些类型的上下文；
- 检索事实、画像、Learner State、Memory 和 Diary；
- 排名、去重、截断和格式化；
- 写入 context snapshot；
- 返回供模型调用的固定结构。

### 6.2.6 AgentRunService

职责：

- 创建 thread/run；
- 加载 Context Pack；
- 选择 tool set；
- 执行 Agent loop；
- 处理 approval、cancel、checkpoint；
- 持久化 trace；
- 生成 run result。

### 6.2.7 EvolutionService

职责：

- 管理 Prompt/Skill 候选；
- 从失败 trace 生成 eval case 候选；
- 调用离线 evaluator；
- 记录 baseline/candidate 指标；
- 决定是否允许进入 shadow/canary；
- 不直接修改源码或 active version。

## 6.3 请求路径：个性化阅读 Coach

```mermaid
sequenceDiagram
    participant U as User
    participant V as Vue
    participant T as Tauri
    participant A as AgentRunService
    participant C as ContextCompiler
    participant D as SQLite
    participant M as LLM

    U->>V: 针对 Q18 提问
    V->>T: agent_run(threadId, request, scope)
    T->>A: run(request)
    A->>C: compile(scope=reading_coach)
    C->>D: 当前 attempt / question / passage evidence
    C->>D: explicit profile / learner state / relevant memory
    C->>D: recent Coach episodes / teaching strategy
    C-->>A: ContextPack + provenance
    A->>D: persist context snapshot
    A->>M: system + context + tools + question
    M-->>A: tool call / final answer
    A->>D: persist run, tool calls, model usage
    A-->>T: stream events
    T-->>V: answer + citations + why-this-style
```

## 6.4 后台路径：每日 Dream

```mermaid
sequenceDiagram
    participant S as Scheduler
    participant J as JobQueue
    participant D as DreamService
    participant DB as SQLite
    participant L as LLM

    S->>J: enqueue daily_dream(dedupe=user+date)
    J->>D: claim job
    D->>DB: load unconsolidated events
    D->>DB: load active related memories
    D->>L: bounded read-only evidence + mutation schema
    L-->>D: memory mutation proposals + journal
    D->>D: deterministic validation
    D->>DB: save candidates, report, provenance
    alt low risk and policy allows
        D->>DB: atomic promote / supersede / archive
    else review required
        D->>DB: status=pending_review
    end
    D->>J: complete job + checkpoint
```

## 6.5 不采用全量 Event Sourcing

虽然引入 `learning_events`，但不建议把整个产品改造成严格 Event Sourcing：

- 现有 attempts 等表继续是当前状态和查询主模型；
- event ledger 用于 Agent、Dream、Learner Model 的分析和增量处理；
- 不要求所有页面通过 replay events 重建状态；
- 避免一次架构重写。

---

# 7. Soul、User、Memory、Diary、Skill 的边界

## 7.1 Soul

Soul 表达：

- Agent 的产品身份；
- 教育目标；
- 安全边界；
- 对成绩、事实和用户自主权的原则；
- 不允许做的事情；
- 在不确定时如何表达。

示例：

```markdown
# IELTS Atlas Learning Agent

- 你是学习辅助者，不是考试成绩的唯一裁判。
- 不得修改原始成绩、答案或历史事实。
- 讲解必须优先引用当前题目证据。
- 个性化画像是可纠正的推断，不得作为对用户能力的绝对标签。
- 不得通过泄露答案制造“学习改善”的假象。
- 任何长期用户画像变化必须可解释、可查看、可删除。
```

写权限：

- 开发者发布管线：可写；
- 产品管理员：可切换已发布版本；
- 生产 Agent：只读；
- Dream Agent：只读；
- 用户：不可直接改安全核心，但可配置可选风格参数。

## 7.2 User Profile

必须分成两个来源：

### Explicit Profile

用户明确提供：

- 目标分数；
- 考试日期；
- 每日可用时间；
- 偏好语言；
- 偏好讲解风格；
- 禁止记忆的内容；
- 是否允许后台 Dream；
- 是否允许使用远程模型处理练习内容。

可信等级最高，除非用户修改，否则自动推断不能覆盖。

### Inferred Profile

系统推断：

- 用户可能偏好先结论后推理；
- 用户对术语解释接受度；
- 用户通常在长回答中途重新提问；
- 用户更容易从例子、对比或反例中理解。

必须包含：

- confidence；
- evidence；
- first/last observed；
- active/superseded；
- 可被用户纠正。

## 7.3 Learner Model

Learner Model 不等于 User Profile。

```text
User Profile：用户是谁、偏好什么、目标是什么
Learner Model：用户目前会什么、不稳定在哪里、证据有多强
```

禁止把“当前某题型掌握度低”写成稳定人格标签。

## 7.4 Memory

### Semantic Memory

例：

- “用户在过去 21 天的 5 个不同文章中，Matching Headings 的主要失误是过早根据局部关键词选项。”

### Episodic Memory

例：

- “2026-08-03 复盘 `asset-X/Q14` 时，使用先概括段落主旨再比较选项的讲解后，用户能够自行解释错误原因，并在两天后的不同文章同类题中答对。”

### Procedural Memory

例：

- “对该用户讲解 Matching Headings：先要求一句话概括段落，再显示候选 heading；避免一开始给答案。”

Procedural Memory 的生效门槛应高于 Semantic Memory，因为它会直接改变 Agent 行为。

## 7.5 Diary / Journal

Diary 是详细工作层，允许：

- 多条观察；
- 当日总结；
- 尚未证实的猜测；
- 待验证问题；
- 当日 Agent 交互摘要；
- 当前学习计划进展。

Diary 不等于 Active Memory。它不应全部进入每次 Prompt。

## 7.6 Skill

Skill 是跨用户可复用的程序化说明，例如：

- 如何复盘 Matching Headings；
- 如何分析 IELTS Task 2 论证结构；
- 如何从连续练习中判断“偶然错误”和“稳定错误”；
- 如何创建一周学习计划。

Skill 不应保存用户私有数据。用户私有的教学策略引用 Skill，并通过参数进行个性化。

## 7.7 权限矩阵

| 对象 | 用户 | 在线 Agent | Dream Agent | 开发 Evolution |
|---|---:|---:|---:|---:|
| Soul active | 只读 | 只读 | 只读 | 候选、评测、发布 |
| Explicit Profile | 查看/编辑/删除 | 读取；仅明确指令时写 | 不覆盖 | 不访问内容 |
| Inferred Profile | 查看/纠正/删除 | 读取 | 提议/更新 | 只使用脱敏 eval |
| Raw Learning Truth | 查看 | 只读 | 只读 | 仅测试 fixture |
| Diary | 查看/编辑/删除 | 可追加 session summary | 生成/重写当日摘要 | 不访问生产内容 |
| Semantic Memory | 查看/管理 | 读取、提议 | 合并/替代/归档 | 不直接访问生产内容 |
| Episodic Memory | 查看/管理 | 检索 | 生成/压缩 | 可用脱敏案例 |
| Procedural Memory | 查看/关闭 | 读取 | 生成候选 | 评测全局 Skill |
| Prompt/Skill active | 只读 | 只读 | 只读 | 发布 |

## 7.8 推荐的 Always-visible token 预算

不建议复制某一产品的固定字符数，而应从配置开始：

| Block | 初始预算建议 |
|---|---:|
| Soul + 安全规则 | 800–1,500 tokens |
| Explicit User Profile | 250–500 tokens |
| Inferred Profile 精选摘要 | 250–500 tokens |
| 当前 scope 核心教学策略 | 200–400 tokens |
| 其余 Memory | 按需检索，不始终注入 |

预算是初始假设，必须通过 token、质量和延迟评测调整。

---

# 8. 数据架构与 SQLite Schema 设计

## 8.1 数据层级

```text
Level 0: Immutable / canonical learning truth
  attempts, attempt_answers, evaluations, coach_messages, annotations ...

Level 1: Append-only normalized evidence
  learning_events, learner_skill_observations

Level 2: Derived mutable projections
  learner_skill_state, inferred_profile, daily_journals

Level 3: Curated memory
  memory_items, memory_evidence, memory_mutations

Level 4: Agent execution and context
  agent_threads, agent_messages, agent_runs, agent_tool_calls,
  agent_checkpoints, agent_context_items

Level 5: Self-evolution governance
  prompt_versions, eval_suites, eval_cases, eval_runs, eval_results
```

## 8.2 Migration 策略

当前 schema version 为 11。建议后续按能力边界增加：

```text
0012_learning_event_ledger.sql
0013_agent_threads_checkpoints.sql
0014_memory_profile_core.sql
0015_journal_dream_jobs.sql
0016_learner_model.sql
0017_prompt_evolution_evals.sql
0018_memory_fts.sql
```

不要把所有表塞进一个 migration。每个 migration 必须：

- 可在当前 v11 数据库升级；
- 单事务；
- 有 rollback fixture 或恢复说明；
- 有 fresh DB 和 upgrade DB 测试；
- 纳入 backup canonical tables；
- 明确 retention 和 privacy delete 行为。

## 8.3 `learning_events`

```sql
CREATE TABLE learning_events (
  id TEXT PRIMARY KEY NOT NULL,
  user_id TEXT NOT NULL DEFAULT 'local',
  event_type TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  source_id TEXT,
  idempotency_key TEXT NOT NULL UNIQUE,

  activity TEXT,
  asset_id TEXT,
  attempt_id TEXT,
  question_id TEXT,
  skill_key TEXT,

  occurred_at TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  schema_version INTEGER NOT NULL DEFAULT 1,

  consolidation_state TEXT NOT NULL DEFAULT 'pending'
    CHECK (consolidation_state IN ('pending','processed','ignored','quarantined')),
  sensitivity TEXT NOT NULL DEFAULT 'normal'
    CHECK (sensitivity IN ('normal','private','restricted')),

  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,

  FOREIGN KEY (attempt_id) REFERENCES attempts(id) ON DELETE CASCADE
);

CREATE INDEX idx_learning_events_pending
  ON learning_events(consolidation_state, occurred_at);
CREATE INDEX idx_learning_events_attempt
  ON learning_events(attempt_id, occurred_at);
CREATE INDEX idx_learning_events_asset
  ON learning_events(asset_id, occurred_at);
CREATE INDEX idx_learning_events_skill
  ON learning_events(skill_key, occurred_at);
CREATE INDEX idx_learning_events_type_time
  ON learning_events(event_type, occurred_at);
```

事件示例：

```json
{
  "eventType": "reading.question_submitted",
  "sourceKind": "attempt_answer",
  "activity": "reading",
  "assetId": "p2-high-120",
  "attemptId": "attempt-...",
  "questionId": "q18",
  "skillKey": "reading.matching_headings.main_idea",
  "occurredAt": "2026-08-10T14:30:00Z",
  "payload": {
    "correct": false,
    "answer": "iv",
    "correctAnswer": "vii",
    "changeCount": 3,
    "visitCount": 4,
    "elapsedMs": 91000,
    "marked": true,
    "attemptOrdinalForAsset": 3,
    "daysSincePreviousAttempt": 2
  }
}
```

## 8.4 Agent thread、message 与 checkpoint

现有 `agent_runs` 是一次执行审计，不足以表示长期对话。

```sql
CREATE TABLE agent_threads (
  id TEXT PRIMARY KEY NOT NULL,
  user_id TEXT NOT NULL DEFAULT 'local',
  thread_kind TEXT NOT NULL,
  scope_json TEXT,
  title TEXT,
  status TEXT NOT NULL DEFAULT 'active'
    CHECK (status IN ('active','archived','deleted')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  last_run_id TEXT,
  FOREIGN KEY (last_run_id) REFERENCES agent_runs(id) ON DELETE SET NULL
);

CREATE TABLE agent_messages (
  id TEXT PRIMARY KEY NOT NULL,
  thread_id TEXT NOT NULL,
  sequence INTEGER NOT NULL,
  role TEXT NOT NULL
    CHECK (role IN ('system','user','assistant','tool','summary')),
  content TEXT NOT NULL,
  structured_payload TEXT,
  source_run_id TEXT,
  created_at TEXT NOT NULL,
  UNIQUE(thread_id, sequence),
  FOREIGN KEY (thread_id) REFERENCES agent_threads(id) ON DELETE CASCADE,
  FOREIGN KEY (source_run_id) REFERENCES agent_runs(id) ON DELETE SET NULL
);

CREATE TABLE agent_checkpoints (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  step_index INTEGER NOT NULL,
  state_json TEXT NOT NULL,
  state_hash TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(run_id, step_index),
  FOREIGN KEY (run_id) REFERENCES agent_runs(id) ON DELETE CASCADE
);
```

建议扩展 `agent_runs`：

```sql
ALTER TABLE agent_runs ADD COLUMN thread_id TEXT;
ALTER TABLE agent_runs ADD COLUMN run_type TEXT NOT NULL DEFAULT 'workspace';
ALTER TABLE agent_runs ADD COLUMN parent_run_id TEXT;
ALTER TABLE agent_runs ADD COLUMN context_snapshot_id TEXT;
ALTER TABLE agent_runs ADD COLUMN cancel_requested_at TEXT;
ALTER TABLE agent_runs ADD COLUMN waiting_reason TEXT;
```

如 SQLite migration 不便直接加入复杂 FK，可先加入列和索引，再在新表层建立引用校验。

## 8.5 显式用户偏好

```sql
CREATE TABLE explicit_user_preferences (
  user_id TEXT NOT NULL DEFAULT 'local',
  preference_key TEXT NOT NULL,
  scope TEXT NOT NULL DEFAULT 'global',
  value_json TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active'
    CHECK (status IN ('active','disabled','deleted')),
  source TEXT NOT NULL DEFAULT 'user'
    CHECK (source IN ('user','import','product_default')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (user_id, preference_key, scope)
);
```

示例 key：

```text
communication.language
communication.answer_length
teaching.show_answer_timing
teaching.prefer_passage_evidence
privacy.allow_background_dream
privacy.allow_remote_llm
privacy.remember_coach_conversations
learning.target_band
learning.exam_date
learning.daily_minutes
```

## 8.6 `memory_items`

```sql
CREATE TABLE memory_items (
  id TEXT PRIMARY KEY NOT NULL,
  user_id TEXT NOT NULL DEFAULT 'local',
  scope TEXT NOT NULL,
  memory_type TEXT NOT NULL
    CHECK (memory_type IN (
      'semantic',
      'episodic',
      'procedural',
      'inferred_profile',
      'goal',
      'constraint'
    )),

  canonical_key TEXT,
  subject_key TEXT,
  title TEXT,
  content TEXT NOT NULL,
  structured_json TEXT,

  status TEXT NOT NULL DEFAULT 'candidate'
    CHECK (status IN (
      'candidate',
      'pending_review',
      'active',
      'superseded',
      'archived',
      'rejected',
      'quarantined',
      'deleted'
    )),

  confidence REAL NOT NULL DEFAULT 0
    CHECK (confidence >= 0 AND confidence <= 1),
  importance REAL NOT NULL DEFAULT 0
    CHECK (importance >= 0 AND importance <= 1),
  source_trust REAL NOT NULL DEFAULT 0
    CHECK (source_trust >= 0 AND source_trust <= 1),
  sensitivity TEXT NOT NULL DEFAULT 'normal'
    CHECK (sensitivity IN ('normal','private','restricted')),

  valid_from TEXT,
  valid_to TEXT,
  first_observed_at TEXT,
  last_observed_at TEXT,
  last_recalled_at TEXT,
  recall_count INTEGER NOT NULL DEFAULT 0,
  successful_use_count INTEGER NOT NULL DEFAULT 0,
  contradicted_count INTEGER NOT NULL DEFAULT 0,

  version INTEGER NOT NULL DEFAULT 1,
  supersedes_id TEXT,
  created_by TEXT NOT NULL,
  created_run_id TEXT,
  content_hash TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,

  FOREIGN KEY (supersedes_id) REFERENCES memory_items(id) ON DELETE SET NULL,
  FOREIGN KEY (created_run_id) REFERENCES agent_runs(id) ON DELETE SET NULL
);

CREATE INDEX idx_memory_active_scope
  ON memory_items(user_id, scope, memory_type, status);
CREATE INDEX idx_memory_subject
  ON memory_items(subject_key, status);
CREATE INDEX idx_memory_canonical
  ON memory_items(canonical_key, status);
CREATE INDEX idx_memory_recency
  ON memory_items(last_observed_at DESC);

CREATE UNIQUE INDEX uq_memory_active_canonical
  ON memory_items(user_id, scope, canonical_key)
  WHERE status = 'active' AND canonical_key IS NOT NULL;
```

`canonical_key` 用于表达一个可替代 slot，例如：

```text
profile.communication.explanation_style
learner.reading.matching_headings.primary_error
strategy.reading.matching_headings.teaching_sequence
```

## 8.7 Memory evidence 与 mutation audit

```sql
CREATE TABLE memory_evidence (
  memory_id TEXT NOT NULL,
  event_id TEXT NOT NULL,
  evidence_role TEXT NOT NULL DEFAULT 'support'
    CHECK (evidence_role IN ('support','contradict','context','outcome')),
  weight REAL NOT NULL DEFAULT 1,
  excerpt TEXT,
  created_at TEXT NOT NULL,
  PRIMARY KEY (memory_id, event_id, evidence_role),
  FOREIGN KEY (memory_id) REFERENCES memory_items(id) ON DELETE CASCADE,
  FOREIGN KEY (event_id) REFERENCES learning_events(id) ON DELETE CASCADE
);

CREATE TABLE memory_mutations (
  id TEXT PRIMARY KEY NOT NULL,
  memory_id TEXT,
  operation TEXT NOT NULL
    CHECK (operation IN (
      'create',
      'promote',
      'merge',
      'supersede',
      'archive',
      'reject',
      'quarantine',
      'restore',
      'delete'
    )),
  actor_type TEXT NOT NULL
    CHECK (actor_type IN ('user','agent','dream','system','developer')),
  actor_id TEXT,
  run_id TEXT,
  before_json TEXT,
  after_json TEXT,
  reason TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (memory_id) REFERENCES memory_items(id) ON DELETE SET NULL,
  FOREIGN KEY (run_id) REFERENCES agent_runs(id) ON DELETE SET NULL
);
```

## 8.8 Daily Journal 与 Dream

```sql
CREATE TABLE daily_journals (
  id TEXT PRIMARY KEY NOT NULL,
  user_id TEXT NOT NULL DEFAULT 'local',
  journal_date TEXT NOT NULL,
  scope TEXT NOT NULL DEFAULT 'global',
  version INTEGER NOT NULL DEFAULT 1,
  status TEXT NOT NULL DEFAULT 'draft'
    CHECK (status IN ('draft','final','superseded','deleted')),
  summary_markdown TEXT NOT NULL,
  structured_json TEXT NOT NULL,
  coverage_start TEXT NOT NULL,
  coverage_end TEXT NOT NULL,
  source_event_count INTEGER NOT NULL,
  dream_run_id TEXT,
  content_hash TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(user_id, journal_date, scope, version)
);

CREATE TABLE dream_runs (
  id TEXT PRIMARY KEY NOT NULL,
  user_id TEXT NOT NULL DEFAULT 'local',
  dream_kind TEXT NOT NULL
    CHECK (dream_kind IN ('session_close','daily','weekly','monthly','manual')),
  status TEXT NOT NULL
    CHECK (status IN ('queued','running','completed','failed','cancelled','interrupted')),
  coverage_start TEXT NOT NULL,
  coverage_end TEXT NOT NULL,
  input_event_count INTEGER NOT NULL DEFAULT 0,
  input_memory_count INTEGER NOT NULL DEFAULT 0,
  candidate_count INTEGER NOT NULL DEFAULT 0,
  promoted_count INTEGER NOT NULL DEFAULT 0,
  rejected_count INTEGER NOT NULL DEFAULT 0,
  quarantined_count INTEGER NOT NULL DEFAULT 0,
  provider TEXT,
  model TEXT,
  prompt_version_id TEXT,
  usage_json TEXT,
  checkpoint_json TEXT,
  error_json TEXT,
  started_at TEXT,
  completed_at TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE dream_candidates (
  id TEXT PRIMARY KEY NOT NULL,
  dream_run_id TEXT NOT NULL,
  candidate_type TEXT NOT NULL,
  operation TEXT NOT NULL,
  target_memory_id TEXT,
  canonical_key TEXT,
  proposed_content TEXT NOT NULL,
  proposed_json TEXT,
  confidence REAL NOT NULL,
  novelty REAL NOT NULL,
  risk_level TEXT NOT NULL
    CHECK (risk_level IN ('low','medium','high','blocked')),
  validation_json TEXT,
  status TEXT NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending','accepted','rejected','auto_promoted','quarantined')),
  reviewed_by TEXT,
  reviewed_at TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (dream_run_id) REFERENCES dream_runs(id) ON DELETE CASCADE,
  FOREIGN KEY (target_memory_id) REFERENCES memory_items(id) ON DELETE SET NULL
);
```

## 8.9 Background Job Queue

```sql
CREATE TABLE background_jobs (
  id TEXT PRIMARY KEY NOT NULL,
  job_type TEXT NOT NULL,
  dedupe_key TEXT UNIQUE,
  payload_json TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'queued'
    CHECK (status IN (
      'queued','running','waiting','completed','failed','cancelled','interrupted'
    )),
  priority INTEGER NOT NULL DEFAULT 0,
  attempts INTEGER NOT NULL DEFAULT 0,
  max_attempts INTEGER NOT NULL DEFAULT 3,
  scheduled_at TEXT NOT NULL,
  locked_at TEXT,
  locked_by TEXT,
  heartbeat_at TEXT,
  checkpoint_json TEXT,
  result_json TEXT,
  error_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_jobs_claim
  ON background_jobs(status, scheduled_at, priority DESC);
```

对于单机桌面应用，不需要 Redis 或外部消息队列。应使用：

- SQLite 原子 claim；
- 单个后台 worker；
- heartbeat；
- 应用重启后 recover stale running jobs；
- dedupe key 防止同一天重复 Dream。

## 8.10 Learner Model 表

```sql
CREATE TABLE skill_catalog (
  skill_key TEXT PRIMARY KEY NOT NULL,
  activity TEXT NOT NULL,
  parent_key TEXT,
  label TEXT NOT NULL,
  description TEXT NOT NULL,
  taxonomy_version INTEGER NOT NULL,
  active INTEGER NOT NULL DEFAULT 1,
  FOREIGN KEY (parent_key) REFERENCES skill_catalog(skill_key)
);

CREATE TABLE question_skill_map (
  asset_id TEXT NOT NULL,
  question_id TEXT NOT NULL,
  skill_key TEXT NOT NULL,
  weight REAL NOT NULL DEFAULT 1,
  mapping_source TEXT NOT NULL,
  mapping_version INTEGER NOT NULL,
  PRIMARY KEY (asset_id, question_id, skill_key),
  FOREIGN KEY (skill_key) REFERENCES skill_catalog(skill_key)
);

CREATE TABLE learner_skill_observations (
  id TEXT PRIMARY KEY NOT NULL,
  user_id TEXT NOT NULL DEFAULT 'local',
  event_id TEXT NOT NULL,
  skill_key TEXT NOT NULL,
  outcome REAL NOT NULL CHECK (outcome >= 0 AND outcome <= 1),
  evidence_weight REAL NOT NULL CHECK (evidence_weight >= 0),
  novelty_weight REAL NOT NULL CHECK (novelty_weight >= 0 AND novelty_weight <= 1),
  time_weight REAL NOT NULL CHECK (time_weight >= 0 AND time_weight <= 1),
  error_type TEXT,
  context_json TEXT,
  observed_at TEXT NOT NULL,
  UNIQUE(event_id, skill_key),
  FOREIGN KEY (event_id) REFERENCES learning_events(id) ON DELETE CASCADE,
  FOREIGN KEY (skill_key) REFERENCES skill_catalog(skill_key)
);

CREATE TABLE learner_skill_state (
  user_id TEXT NOT NULL DEFAULT 'local',
  skill_key TEXT NOT NULL,
  alpha REAL NOT NULL DEFAULT 1,
  beta REAL NOT NULL DEFAULT 1,
  mastery_mean REAL NOT NULL DEFAULT 0.5,
  uncertainty REAL NOT NULL DEFAULT 1,
  evidence_count INTEGER NOT NULL DEFAULT 0,
  distinct_asset_count INTEGER NOT NULL DEFAULT 0,
  recent_error_rate REAL,
  stability_days REAL,
  last_practiced_at TEXT,
  next_review_at TEXT,
  model_version TEXT NOT NULL,
  explanation_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (user_id, skill_key),
  FOREIGN KEY (skill_key) REFERENCES skill_catalog(skill_key)
);
```

## 8.11 Context snapshot 与检索审计

```sql
CREATE TABLE agent_context_snapshots (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  compiler_version TEXT NOT NULL,
  scope TEXT NOT NULL,
  query_plan_json TEXT NOT NULL,
  token_budget INTEGER NOT NULL,
  used_tokens INTEGER NOT NULL,
  rendered_context TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (run_id) REFERENCES agent_runs(id) ON DELETE CASCADE
);

CREATE TABLE agent_context_items (
  snapshot_id TEXT NOT NULL,
  item_type TEXT NOT NULL,
  item_id TEXT NOT NULL,
  rank INTEGER NOT NULL,
  score REAL NOT NULL,
  estimated_tokens INTEGER NOT NULL,
  inclusion_reason TEXT NOT NULL,
  provenance_json TEXT,
  PRIMARY KEY (snapshot_id, item_type, item_id),
  FOREIGN KEY (snapshot_id) REFERENCES agent_context_snapshots(id) ON DELETE CASCADE
);
```

## 8.12 Prompt、Skill 和评测版本

```sql
CREATE TABLE prompt_artifacts (
  id TEXT PRIMARY KEY NOT NULL,
  artifact_key TEXT NOT NULL UNIQUE,
  artifact_type TEXT NOT NULL
    CHECK (artifact_type IN ('soul','system_section','feature_prompt','skill','tool_description')),
  owner_scope TEXT NOT NULL,
  description TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE prompt_versions (
  id TEXT PRIMARY KEY NOT NULL,
  artifact_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  parent_version_id TEXT,
  status TEXT NOT NULL
    CHECK (status IN ('draft','candidate','shadow','canary','active','retired','rejected')),
  content TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  schema_json TEXT,
  optimizer_json TEXT,
  evaluation_run_id TEXT,
  created_by TEXT NOT NULL,
  created_at TEXT NOT NULL,
  activated_at TEXT,
  UNIQUE(artifact_id, version),
  FOREIGN KEY (artifact_id) REFERENCES prompt_artifacts(id) ON DELETE CASCADE,
  FOREIGN KEY (parent_version_id) REFERENCES prompt_versions(id) ON DELETE SET NULL
);

CREATE UNIQUE INDEX uq_prompt_active
  ON prompt_versions(artifact_id)
  WHERE status = 'active';

CREATE TABLE eval_suites (
  id TEXT PRIMARY KEY NOT NULL,
  suite_key TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE eval_cases (
  id TEXT PRIMARY KEY NOT NULL,
  suite_id TEXT NOT NULL,
  case_key TEXT NOT NULL,
  input_json TEXT NOT NULL,
  expected_json TEXT,
  grader_spec_json TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  source_ref TEXT,
  split TEXT NOT NULL CHECK (split IN ('train','validation','holdout','regression')),
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  UNIQUE(suite_id, case_key),
  FOREIGN KEY (suite_id) REFERENCES eval_suites(id) ON DELETE CASCADE
);

CREATE TABLE eval_runs (
  id TEXT PRIMARY KEY NOT NULL,
  suite_id TEXT NOT NULL,
  baseline_version_id TEXT,
  candidate_version_id TEXT,
  status TEXT NOT NULL,
  trial_count INTEGER NOT NULL,
  config_json TEXT NOT NULL,
  metrics_json TEXT,
  started_at TEXT,
  completed_at TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (suite_id) REFERENCES eval_suites(id) ON DELETE CASCADE
);

CREATE TABLE eval_results (
  id TEXT PRIMARY KEY NOT NULL,
  eval_run_id TEXT NOT NULL,
  case_id TEXT NOT NULL,
  trial_index INTEGER NOT NULL,
  candidate_kind TEXT NOT NULL CHECK (candidate_kind IN ('baseline','candidate')),
  outcome_json TEXT NOT NULL,
  trace_ref TEXT,
  grader_results_json TEXT NOT NULL,
  score REAL NOT NULL,
  latency_ms INTEGER,
  token_usage_json TEXT,
  created_at TEXT NOT NULL,
  UNIQUE(eval_run_id, case_id, trial_index, candidate_kind),
  FOREIGN KEY (eval_run_id) REFERENCES eval_runs(id) ON DELETE CASCADE,
  FOREIGN KEY (case_id) REFERENCES eval_cases(id) ON DELETE CASCADE
);
```

## 8.13 FTS5 起步方案

```sql
CREATE VIRTUAL TABLE memory_fts USING fts5(
  memory_id UNINDEXED,
  title,
  content,
  subject_key,
  tokenize = 'unicode61'
);

CREATE VIRTUAL TABLE agent_message_fts USING fts5(
  message_id UNINDEXED,
  thread_id UNINDEXED,
  content,
  tokenize = 'unicode61'
);

CREATE VIRTUAL TABLE journal_fts USING fts5(
  journal_id UNINDEXED,
  summary_markdown,
  tokenize = 'unicode61'
);
```

FTS 同步可通过 repository 显式写入，初期不建议使用复杂 trigger 隐藏副作用。

## 8.14 是否立即使用 Embedding

第一阶段不要求 embedding，原因：

- 学习数据天然有 activity、asset、question、skill、date 等强结构过滤；
- FTS5 对明确题型、术语、题号和用户措辞很有效；
- 本地 embedding 会增加模型分发、跨平台、版本和存储复杂度；
- 远程 embedding 会引入额外隐私与成本。

启用条件：

- FTS5 + metadata 在 Memory Retrieval eval 上明显不足；
- 有至少 100–300 条高质量 memory；
- 已有 retrieval precision/recall 指标；
- 能固定 embedding model version；
- 有重建索引和回滚方案。


---

# 9. 学习事件账本与证据模型

## 9.1 为什么不能让 Dream 直接扫描所有业务表

Dream 每次直接查询 `attempts`、`attempt_answers`、`coach_messages`、`writing_evaluations` 等表会导致：

- 每个 Dream 版本重新实现数据拼接；
- 表结构变化直接影响 Prompt；
- 难以判断哪些记录已经处理；
- 难以做 idempotency；
- 难以把同一业务事实统一映射为学习证据；
- 难以进行增量处理和失败恢复。

因此需要一个轻量的标准化 `learning_events` 层。它不是第二事实源，而是分析事件账本。

## 9.2 事件分类

### 练习事件

```text
reading.attempt_started
reading.answer_changed
reading.question_submitted
reading.attempt_submitted
reading.attempt_reviewed
reading.attempt_repeated
writing.draft_saved
writing.attempt_submitted
writing.evaluation_completed
writing.revision_created
vocab.item_reviewed
vocab.session_completed
```

### Coach / Agent 交互事件

```text
coach.user_message
coach.assistant_message
coach.response_rated
coach.response_corrected
coach.question_rephrased
coach.answer_abandoned
agent.tool_called
agent.tool_failed
agent.run_completed
```

### 用户显式配置事件

```text
profile.preference_set
profile.preference_removed
memory.user_pinned
memory.user_corrected
memory.user_deleted
privacy.setting_changed
```

### 后续结果事件

```text
learning.recommendation_followed
learning.recommendation_skipped
learning.skill_retested
learning.strategy_success_candidate
learning.strategy_failure_candidate
```

## 9.3 事件 payload 必须保存什么

事件 payload 应保存分析所需信息，但不能复制整张业务表。

原则：

- 保存稳定 ID；
- 保存当时快照中不可重建的少量字段；
- 大文本通过 source ID 读取；
- 敏感内容标 sensitivity；
- 使用 schema version；
- 计算 content hash。

例如 `coach.response_rated`：

```json
{
  "threadId": "coach-thread-1",
  "userMessageId": "msg-u",
  "assistantMessageId": "msg-a",
  "rating": "negative",
  "reasonCodes": ["too_generic", "did_not_use_passage"],
  "userCorrectionMessageId": "msg-u2",
  "responseStrategyVersionId": "strategy-v3"
}
```

## 9.4 事件产生点

不得让 Vue 自己构造权威学习事件。事件应由 Rust use case 在业务事务成功后产生。

示例：阅读提交事务：

```rust
fn submit_reading_attempt(
    conn: &Connection,
    cmd: &ReadingSubmitCommand,
) -> DbResult<ReadingSubmitResult> {
    let tx = conn.unchecked_transaction()?;

    let result = submit_reading_attempt_inner(&tx, cmd)?;

    for answer in &result.answers {
        append_learning_event(
            &tx,
            LearningEvent::reading_question_submitted(
                &result.attempt,
                answer,
                cmd.idempotency_key.as_str(),
            ),
        )?;
    }

    append_learning_event(
        &tx,
        LearningEvent::reading_attempt_submitted(&result),
    )?;

    tx.commit()?;
    Ok(result)
}
```

事件与业务结果在同一事务写入，避免“提交成功但分析事件丢失”。

## 9.5 当前 v11 数据的迁移

本计划不要求兼容旧 Electron 或 `opensource` 数据，但必须保护当前 AI 产品已经产生的数据。

建议：

- migration 只建表，不在 schema migration 中执行大规模 LLM 或复杂 backfill；
- 增加一次性 deterministic backfill job；
- 从现有 completed/submitted attempts 生成基础事件；
- backfill event 使用固定 idempotency key；
- 不重新触发 Coach LLM 或 Dream；
- 用户可选择“从现有记录构建学习画像”。

```rust
fn backfill_learning_events(batch_size: usize) -> JobStepResult {
    let cursor = load_checkpoint_cursor();
    let attempts = load_attempts_after(cursor, batch_size);

    for attempt in attempts {
        emit_attempt_events_if_missing(attempt)?;
    }

    save_checkpoint(last_attempt_id);
    if attempts.len() < batch_size {
        JobStepResult::Completed
    } else {
        JobStepResult::Continue
    }
}
```

## 9.6 证据可信等级

| 来源 | 初始 source trust |
|---|---:|
| 用户显式设置 | 1.00 |
| 数据库确定性练习结果 | 0.95 |
| 用户明确纠正 Agent | 0.95 |
| 多次跨资产一致行为 | 0.85–0.95 |
| 单次用户点赞/点踩 | 0.50–0.70 |
| 模型从一次对话推断 | 0.30–0.55 |
| 无来源模型总结 | 0，禁止 promotion |

具体数值应作为配置和评测对象，不应硬编码在 Prompt 中。

## 9.7 Evidence Diversity

不能因为用户在同一套题反复做对而高估能力。

定义：

```text
Evidence diversity =
  distinct assets
  + distinct dates
  + distinct question variants
  + delayed retrieval evidence
  - repeated exact item penalty
```

同题第三次作答的 mastery 权重应显著低于不同文章的同题型正确。

## 9.8 事件处理状态

```text
pending      尚未进入任何 consolidation window
processed    已完成候选提取或确定性模型更新
ignored      明确无长期价值
quarantined  检测到异常、注入或敏感风险
```

处理状态只代表下游处理，不代表删除原始事件。

---

# 10. 长期记忆生命周期

## 10.1 生命周期状态机

```mermaid
stateDiagram-v2
    [*] --> Candidate
    Candidate --> PendingReview: medium/high risk
    Candidate --> Active: low-risk promotion gate passed
    Candidate --> Rejected: validation failed
    Candidate --> Quarantined: security failed
    PendingReview --> Active: user approves
    PendingReview --> Rejected: user rejects
    Active --> Superseded: newer conflicting memory accepted
    Active --> Archived: stale/low-use/low-value
    Active --> Quarantined: later security finding
    Archived --> Active: restored
    Superseded --> Archived: retention compaction
    Rejected --> [*]
```

## 10.2 Memory Candidate 提取规则

候选必须回答：

1. 这是事实、经历、策略还是偏好？
2. 它对未来哪些任务有价值？
3. 来源证据是什么？
4. 是新信息、补充信息还是冲突信息？
5. 置信度和风险是什么？
6. 应新建、合并、替代还是忽略？

建议的模型输出 schema：

```json
{
  "candidates": [
    {
      "memoryType": "semantic",
      "scope": "reading",
      "canonicalKey": "learner.reading.matching_headings.primary_error",
      "subjectKey": "reading.matching_headings",
      "content": "用户近期主要错误是根据局部关键词选择 heading，而未先概括段落主旨。",
      "evidenceEventIds": ["evt-1", "evt-2", "evt-3"],
      "confidence": 0.84,
      "importance": 0.76,
      "proposedOperation": "supersede_or_create",
      "reason": "三篇不同文章、两周内出现同类错误",
      "risks": []
    }
  ]
}
```

## 10.3 Promotion Gate

候选进入 Active Memory 前必须经过确定性校验：

```rust
struct PromotionDecision {
    allowed: bool,
    requires_review: bool,
    reasons: Vec<String>,
}

fn validate_candidate(c: &MemoryCandidate, evidence: &[LearningEvent])
    -> PromotionDecision
{
    deny_if(c.evidence_event_ids.is_empty(), "no evidence");
    deny_if(!all_evidence_exists(c), "missing evidence");
    deny_if(c.content.trim().is_empty(), "empty content");
    deny_if(c.confidence < MIN_STORE_CONFIDENCE, "low confidence");
    deny_if(detect_prompt_injection(c.content), "injection pattern");
    deny_if(contains_secret(c.content), "secret-like content");
    deny_if(c.memory_type == Procedural && evidence.len() < MIN_STRATEGY_EVIDENCE,
            "insufficient procedural evidence");

    review_if(c.sensitivity != Normal, "sensitive");
    review_if(c.proposed_operation == Delete, "destructive mutation");
    review_if(conflicts_with_explicit_profile(c), "explicit preference conflict");
    review_if(c.memory_type == InferredProfile && c.confidence < PROFILE_AUTO_THRESHOLD,
              "profile inference");

    allow()
}
```

## 10.4 合并与替代

### Merge

适用于同一结论的证据增加：

```text
旧：用户在 Matching Headings 中容易受关键词干扰。
新证据：另外两篇文章中出现相同错误。
结果：内容可轻微更新，confidence、evidence 和 last_observed 增加。
```

### Supersede

适用于结论改变：

```text
旧 Active：用户偏好先看完整答案再听解释。
新显式偏好：用户明确要求先自己思考，不要提前显示答案。
结果：旧条目 superseded，新显式偏好 active。
```

### Archive

适用于：

- 长期未检索；
- 已被更稳定高层总结覆盖；
- 只与已结束短期目标相关；
- 置信度低且无新证据；
- 用户关闭某类记忆。

Archive 不是立即删除，仍可用于审计和重新验证。

## 10.5 Active Memory 容量

按 scope 和 memory type 建立预算，而不是全局只看条数：

```text
inferred_profile:        20–40 个活跃 slot
semantic_reading:        40–80 条
semantic_writing:        40–80 条
episodic_coach:          30–60 条
procedural_user:         10–25 条
goals_constraints:       10–20 条
```

预算值需要通过真实使用调整。达到上限时：

1. 合并重复；
2. supersede 冲突；
3. 将低复用 episode 归档；
4. 提升高层 reflection；
5. 绝不静默丢弃用户 pinned memory。

## 10.6 Memory Utility

建议记录每条 memory 的使用结果：

```text
retrieved_count
included_count
successful_use_count
contradicted_count
user_corrected_count
last_recalled_at
```

但“回答得到点赞”不能直接证明某条 memory 正确。成功使用可由多信号组成：

- 用户未纠正且完成后续任务；
- 后续同类练习表现改善；
- 人工标注认为上下文相关；
- Agent 回答有正确 evidence grounding；
- memory 未造成不必要的过度个性化。

## 10.7 Recall Feedback

每次 Context Compiler 选择 memory 后写入 `agent_context_items`。运行结束可生成使用反馈：

```json
{
  "memoryId": "mem-1",
  "runId": "run-1",
  "retrieved": true,
  "included": true,
  "modelReferenced": true,
  "userCorrected": false,
  "outcomeSignal": "unknown"
}
```

不要要求模型自行判断“这条 memory 对我非常有用”并直接增加权重；模型自评只能作为弱信号。

## 10.8 冲突规则

优先级：

```text
用户最新显式设置
  > 用户明确纠正
  > 多次确定性练习事实
  > 跨会话高置信推断
  > 单次对话推断
  > 模型无来源陈述
```

冲突时不得把两个相反条目都 active。

## 10.9 时间衰减

不同 memory 类型使用不同衰减：

| 类型 | 衰减 |
|---|---|
| 显式偏好 | 不自动衰减，等待用户修改 |
| 稳定背景 | 很慢 |
| 学习目标 | 按截止日期和状态 |
| 技能弱点 | 随新证据动态更新 |
| Coach episode | 中等速度，低复用后归档 |
| 临时计划 | 快速衰减 |
| 安全政策 | 不衰减 |

示例：

```text
recency_weight = exp(-ln(2) * age_days / half_life_days)
```

但最终评分不能只按新旧，稳定多证据事实即使较旧也应保留。

## 10.10 记忆删除

用户删除分两种：

### 普通删除

- status=`deleted`；
- 不再检索；
- 保留最小 mutation audit；
- audit 不保存被删除的完整正文。

### 隐私彻底删除

- 删除正文、embedding、FTS、evidence excerpt；
- 按配置级联删除相关 derived profile；
- 保留不可反推出内容的 tombstone/hash；
- 后续 Dream 不得从仍保留的原始 Coach 内容再次恢复，除非用户明确允许。

因此需要同时处理 source retention 和 derived memory retention。

---

# 11. Daily Journal 与 Dream 离线整合机制

## 11.1 设计目标

Dream 不是模拟人类意识，而是一个工程化后台整合器：

- 扫描时间窗口内的新证据；
- 生成当日可读日志；
- 找出跨会话模式；
- 合并、替代和归档长期记忆；
- 更新 Learner Model 解释；
- 生成教学策略候选；
- 不直接改 Soul；
- 不直接改全局 Prompt；
- 不修改原始练习事实。

## 11.2 四级整合

### Level 0：Hot Capture

发生时间：业务事务完成时。

工作：

- 写 `learning_events`；
- 对用户明确“记住/不要记住”立即写 explicit preference；
- 不调用高成本模型。

### Level 1：Session Close Reflection

发生时间：Coach thread 结束、阅读复盘结束、写作评估查看完成。

工作：

- 总结本 session；
- 提取 0–5 个 candidate；
- 提取待验证问题；
- 写入当日 journal draft；
- 不自动改变高风险 Active Memory。

### Level 2：Daily Dream

发生时间：每日指定窗口、应用空闲或下次启动补跑。

工作：

- 汇总当天所有 scope；
- 跨 session 去重；
- 与 Active Memory 比较；
- 生成 daily journal final；
- 处理低风险 memory mutation；
- 产生 pending review。

### Level 3：Weekly Reflection

工作：

- 跨日期、跨资产分析稳定趋势；
- 更新技能模型解释；
- 识别有效/无效教学策略；
- 生成一周学习摘要；
- 提议下一阶段复习方向。

### Level 4：Monthly Compaction

工作：

- 清理重复 episode；
- 合并低层 memory 为高层 reflection；
- 归档已结束目标；
- 重新计算 active context budget；
- 生成用户可审阅的变更报告。

## 11.3 Light / REM / Deep 的工程化映射

可以借用 OpenClaw 命名，但不必暴露为神秘概念：

### Light

确定性预处理：

- 事件去重；
- 敏感分类；
- 按 activity/skill/asset/date 分组；
- 计算重复题间隔；
- 加载相关 active memory；
- 生成 evidence packet。

### REM

LLM 发现模式：

- 提出 high-level questions；
- 生成候选总结；
- 识别冲突；
- 提出 memory mutation 和 strategy candidate；
- 生成 journal narrative。

REM 结果只写 candidate。

### Deep

确定性治理和提交：

- schema validation；
- provenance validation；
- safety scan；
- conflict resolution；
- capacity check；
- atomic mutation；
- audit；
- mark events processed。

只有 Deep 可以改变 Active Memory。

## 11.4 Dream 触发

```rust
fn should_schedule_daily_dream(now: DateTime, state: &DreamScheduleState) -> bool {
    let has_pending = state.pending_event_count >= MIN_DAILY_EVENTS;
    let past_window = now.local_time() >= configured_dream_time;
    let not_done = state.last_completed_date < now.local_date();
    has_pending && past_window && not_done
}
```

附加触发：

```text
重要度阈值：最近未整合事件 importance 总和 > threshold
数量阈值：pending event > N
用户手动：立即生成今日学习总结
应用启动补跑：上次到今天有未处理事件
```

## 11.5 不应依赖应用持续运行

桌面应用可能夜间关闭。因此：

- “夜间 Dream”是逻辑窗口，不是必须常驻；
- Job 可在下次启动后补跑；
- UI 显示“最后整合时间”；
- 大型任务在应用空闲时执行；
- 用户可暂停、取消；
- 低电量或省电模式可延迟。

## 11.6 Dream 输入包

```rust
struct DreamEvidencePack {
    window: TimeRange,
    events: Vec<LearningEventView>,
    current_memories: Vec<MemoryView>,
    learner_state_before: Vec<SkillStateView>,
    explicit_preferences: Vec<PreferenceView>,
    unresolved_questions: Vec<OpenQuestion>,
    budgets: DreamBudgets,
}
```

必须确保：

- 原始工具输出以 data block 包裹；
- 外部或用户输入中的指令不进入 system role；
- 不发送无关完整文章；
- passage evidence 只发送必要片段；
- 所有 evidence 带 ID。

## 11.7 Dream 输出 contract

```rust
struct DreamOutput {
    journal: JournalDraft,
    memory_proposals: Vec<MemoryMutationProposal>,
    strategy_proposals: Vec<TeachingStrategyProposal>,
    learner_explanations: Vec<LearnerStateExplanation>,
    open_questions: Vec<OpenQuestion>,
}

struct MemoryMutationProposal {
    operation: MutationKind,
    target_memory_id: Option<String>,
    canonical_key: Option<String>,
    memory_type: MemoryType,
    scope: String,
    content: String,
    structured: Value,
    evidence_ids: Vec<String>,
    confidence: f32,
    novelty: f32,
    reason: String,
    risk_flags: Vec<String>,
}
```

## 11.8 Daily Journal 格式

```markdown
# 2026-08-10 学习日志

## 今日完成
- 阅读 P2 一篇，得分 9/13
- 复盘 Matching Headings 3 题
- 写作 Task 2 完成一次评估

## 新观察
- Matching Headings 仍容易根据局部关键词选择，但在被要求先概括主旨后能自行纠正。
- 写作中的主要问题由词汇准确性转向段落论证连接。

## 与过去相比
- 同一阅读文章第三次练习，错误数下降，但重复题熟悉度较高，不能视为跨材料掌握。
- 不同文章中的同类题首次出现迁移成功证据。

## 有效的讲解方式
- 先要求用户给出段落一句话主旨，再比较选项。

## 待验证
- 用户是否普遍更适合“先做一步，再给下一步”的讲解，而非一次展示完整流程？

## 记忆变更建议
- 更新 `learner.reading.matching_headings.primary_error`（待自动门禁）
- 新建 episodic memory 1 条（低风险）
```

## 11.9 Weekly Reflection

Weekly 不能只是 7 篇 daily 的拼接。它应回答：

- 哪些弱点在不同材料中重复？
- 哪些弱点已恢复？
- 哪些改善只来自同题熟悉？
- 哪些教学策略在多个 session 中有效？
- 哪些记忆已冲突或过期？
- 下一周应增加什么类型的 retrieval practice？

## 11.10 Monthly Compaction 算法

```rust
fn compact_memory(scope: &str) -> CompactionPlan {
    let active = load_active_memories(scope);
    let clusters = cluster_by_canonical_subject(active);

    for cluster in clusters {
        if cluster.has_conflict() {
            propose_supersession(cluster);
        } else if cluster.is_redundant() {
            propose_summary_replacement(cluster);
        } else if cluster.low_utility_and_stale() {
            propose_archive(cluster);
        }
    }

    enforce_budget();
    preserve_user_pinned();
    return plan;
}
```

## 11.11 自动 promotion 分级

| 风险 | 示例 | 默认行为 |
|---|---|---|
| Low | 多次练习形成的题型弱点；一条具体 episode | 可自动 active |
| Medium | 推断用户偏好；改变 Coach 教学顺序 | pending review 或高阈值自动 |
| High | 敏感个人信息；影响长期目标；强能力判断 | 必须用户确认 |
| Blocked | 凭证、注入指令、外部恶意内容 | quarantine |

## 11.12 Dream 成本控制

- 按 event group 而不是逐事件调用 LLM；
- 先确定性聚合，再用 LLM；
- 无足够新证据时跳过；
- Daily 使用较低成本模型，Weekly 可使用高质量模型；
- 输出严格 schema；
- 记录 token、latency 和 candidate yield；
- 同一 evidence window 使用 content hash 防重复执行；
- 设置每日 token/cost hard limit；
- 超限时只生成 deterministic journal，不做 LLM consolidation。

---

# 12. 学习者模型与重复练习分析

## 12.1 目标

Learner Model 应回答：

- 用户目前在哪些技能上稳定、波动或薄弱？
- 结论来自多少不同题目和日期？
- 是否只是同题熟悉而非能力迁移？
- 最近是否出现恢复或退步？
- 何时适合再次检索练习？
- 哪类错误重复出现？
- Agent 推荐是否带来了后续改善？

## 12.2 技能 taxonomy

第一版必须由人工维护并版本化，不让 LLM 自由创建无限 skill。

### Reading 顶层

```text
reading.comprehension.main_idea
reading.comprehension.detail
reading.comprehension.inference
reading.evidence.localization
reading.paraphrase.recognition
reading.distractor.resistance
reading.time_management
reading.answer_format
```

### 题型技能

```text
reading.matching_headings.main_idea
reading.matching_headings.scope_match
reading.matching_headings.distractor
reading.tfng.evidence_strength
reading.tfng.not_given_boundary
reading.multiple_choice.option_elimination
reading.summary_completion.grammar_fit
reading.summary_completion.paraphrase
reading.matching_information.localization
```

### Writing 顶层

```text
writing.task_response
writing.coherence
writing.lexical_resource
writing.grammar_accuracy
writing.argument_structure
writing.example_relevance
writing.revision_skill
```

## 12.3 Error Taxonomy

错误类型应区分：

```text
knowledge_gap
misread_question
keyword_matching
scope_mismatch
unsupported_inference
evidence_localization_failure
distractor_attraction
answer_format_error
time_pressure
late_change_from_correct_to_wrong
random_guess
language_paraphrase_gap
strategy_not_applied
```

LLM 可建议 error type，但最终必须通过规则、题型 schema 或人工修订约束。

## 12.4 第一版 Mastery 模型

推荐 Beta-Bernoulli 累积：

```text
alpha = prior_success + Σ(correct_i × weight_i)
beta  = prior_failure + Σ((1-correct_i) × weight_i)
mastery_mean = alpha / (alpha + beta)
uncertainty ≈ 1 / sqrt(alpha + beta)
```

单条 evidence 权重：

```text
weight =
  question_skill_weight
  × time_decay
  × novelty_weight
  × completion_quality
  × evidence_trust
```

### novelty_weight

```text
new asset, delayed attempt             1.00
new asset, same day                    0.85
same asset, delayed                    0.50–0.70
same asset, immediate repeat           0.20–0.40
same exact question after answer shown 0.05–0.20
```

### completion_quality

可考虑：

- 是否使用 hint；
- 是否先看到答案；
- 是否超时；
- 是否中途改变答案；
- 是否 review mode；
- 是否真实考试模式。

## 12.5 时间衰减

不是简单把旧正确答案忘掉，而是降低其对“当前掌握”的影响：

```rust
fn time_weight(age_days: f64, half_life_days: f64) -> f64 {
    (-std::f64::consts::LN_2 * age_days / half_life_days).exp()
}
```

half-life 可按 skill 和用户稳定性调整。

## 12.6 同一题重复练习分析

对同一 `asset_id + question_id` 的序列：

```rust
struct RepeatedQuestionTimeline {
    attempts: Vec<QuestionAttemptPoint>,
    transitions: Vec<QuestionTransition>,
}

struct QuestionTransition {
    from_attempt_id: String,
    to_attempt_id: String,
    gap_days: f64,
    from_correct: bool,
    to_correct: bool,
    answer_changed: bool,
    elapsed_delta_ms: i64,
    explanation_seen_between: bool,
}
```

分类：

| 模式 | 判断 |
|---|---|
| Persistent misconception | 间隔后多次错误，错误答案或理由相似 |
| Unstable knowledge | 对错交替，跨题也不稳定 |
| Recovered | 先错后对，且在不同 asset 同 skill 上保持 |
| Familiarity gain | 同题变对，但新题无迁移证据 |
| Regression | 过去跨题稳定，近期多次错误 |
| Speed-accuracy tradeoff | 更快但错误增加，或更慢且稳定改善 |
| Strategy adoption | Coach 建议后，过程指标和新题结果均改善 |

## 12.7 跨题型重复错误

通过 `skill_key` 和 `error_type` 聚合：

```sql
SELECT
  skill_key,
  error_type,
  COUNT(*) AS n,
  COUNT(DISTINCT json_extract(context_json, '$.assetId')) AS assets,
  MIN(observed_at),
  MAX(observed_at)
FROM learner_skill_observations
WHERE outcome < 0.5
GROUP BY skill_key, error_type;
```

只有满足最低 distinct asset、时间跨度和 evidence count，才形成稳定弱点 memory。

## 12.8 掌握状态分层

建议 UI 不直接展示伪精确百分比，而展示：

```text
Insufficient evidence
Emerging
Unstable
Developing
Stable
Needs refresh
```

同时提供：

- mastery mean；
- uncertainty；
- evidence count；
- distinct assets；
- last practiced；
- 解释文本。

## 12.9 Learner State 更新伪代码

```rust
fn apply_observation(
    state: SkillState,
    obs: SkillObservation,
) -> SkillState {
    let w = obs.evidence_weight
        * obs.novelty_weight
        * obs.time_weight;

    let mut next = state;
    next.alpha += obs.outcome * w;
    next.beta += (1.0 - obs.outcome) * w;
    next.mastery_mean = next.alpha / (next.alpha + next.beta);
    next.uncertainty = 1.0 / (next.alpha + next.beta).sqrt();
    next.evidence_count += 1;
    next.last_practiced_at = Some(obs.observed_at);
    next.explanation = build_deterministic_explanation(&next, &obs);
    next
}
```

## 12.10 推荐复习时间

初期可采用解释性规则，而非让 LLM 自由安排：

```text
高错误 + 高置信：短间隔重练，但使用不同题目
不稳定 + 高 uncertainty：尽快补充诊断题
稳定 + 最近练习：延长间隔
稳定但长期未练：安排 retrieval refresh
同题熟悉度高、跨题证据低：优先新材料迁移题
```

## 12.11 Agent 如何使用 Learner Model

Agent 只能把 learner state 当作概率性证据：

错误表达：

> 你不擅长 Matching Headings。

正确表达：

> 最近 14 天里，你在 4 篇不同文章的 Matching Headings 中有 3 次出现“先根据关键词选项”的模式；目前证据显示这一策略还不稳定。我们可以用一篇新材料验证，而不是重复原题。

## 12.12 后续 Knowledge Tracing 升级条件

只有满足以下条件才评估 BKT/DKT：

- taxonomy 稳定；
- question-skill mapping 质量可评测；
- 用户样本和交互序列足够；
- 有 train/validation/holdout；
- 可解释模型已作为 baseline；
- 复杂模型在预测、推荐和真实学习结果上显著优于 baseline；
- 能解释数据隐私和模型更新策略。


---

# 13. Context Engineering 与上下文编译器

## 13.1 为什么 Context Compiler 是整个产品的核心

长期记忆系统最常见的失败不是“没有保存数据”，而是：

- 保存了太多低价值数据；
- 每轮都把所有历史塞进 Prompt；
- 画像、事实、推断和当前题目相互冲突；
- 相同结论被多种表述重复注入；
- 过期记忆仍以高权重影响回答；
- Agent 无法说明某个判断来自哪里；
- 上下文过长导致当前问题和工具结果反而被淹没。

因此，Memory Store 只解决“存什么”，Context Compiler 才解决：

> 在当前任务、当前页面、当前用户目标和当前 token 预算下，哪些信息应该进入模型，按什么顺序进入，以什么可信度和证据标签进入。

这应当成为 `ielts-application` 中的正式用例，而不是散落在 Vue 页面、Coach Prompt 和 Agent command 中的字符串拼接。

## 13.2 Context Pack 的输入

建议定义统一请求：

```rust
pub struct BuildContextRequest {
    pub request_id: String,
    pub thread_id: Option<String>,
    pub user_query: String,
    pub task: AgentTaskKind,
    pub surface: ProductSurface,
    pub locale: Locale,
    pub current_asset_id: Option<String>,
    pub current_attempt_id: Option<String>,
    pub selected_question_ids: Vec<String>,
    pub selected_text: Option<String>,
    pub requested_capabilities: Vec<String>,
    pub token_budget: ContextBudget,
}
```

其中：

```rust
pub enum AgentTaskKind {
    GeneralDialogue,
    ReadingQuestionExplanation,
    ReadingAttemptReview,
    RepeatedAttemptComparison,
    WritingEvaluationExplanation,
    WritingRevision,
    VocabularyReview,
    StudyPlanning,
    MemoryReview,
    DreamReview,
}

pub enum ProductSurface {
    AgentWorkspace,
    ReadingPractice,
    ReadingResult,
    WritingCompose,
    WritingResult,
    History,
    MemoryCenter,
    DailyBrief,
}
```

`task` 不应只由 LLM 分类。建议使用：

1. 页面和按钮产生的显式 `task hint`；
2. 当前 route 和实体 ID；
3. 确定性规则；
4. 只有在仍不明确时才调用轻量分类模型。

## 13.3 Context Source 分层

每次编译可访问以下来源，但不是每次全部使用：

| Source | 内容 | 默认可信度 | 是否 always-visible |
|---|---|---:|---:|
| Soul | 产品身份、安全与教学边界 | 1.00 | 是，严格有界 |
| Explicit User | 用户明确设置、目标和偏好 | 1.00 | 少量 |
| Current Task Evidence | 当前题目、原文、答案、当前作文 | 1.00 | 当前任务必须 |
| Canonical Learning Facts | attempt、answer、score、timeline | 1.00 | 按需 |
| Learner State | 聚合技能状态和 uncertainty | 0.70–0.95 | 按需 |
| Semantic Memory | 稳定画像和结论 | 条目自身 confidence | 按需/少量核心 |
| Episodic Memory | 有代表性的学习事件 | 条目自身 confidence | 按需 |
| Procedural Memory | 对该用户有效的教学方法 | 条目自身 confidence | 少量 |
| Recent Thread | 当前对话历史 | 1.00 | 最近窗口 |
| Search Results | FTS/embedding 返回 | 动态 | 按需 |
| Daily Journal | 当日工作摘要 | 0.60–0.90 | 按需 |
| Dream Report | 离线整合说明 | 不直接作为事实 | 通常不注入 |

重要约束：Dream 报告本身是解释和审计材料，真正被注入的是 Dream 批准后的 active memory 和 learner state，而不是整篇 Dream 文本。

## 13.4 Context Budget

建议不要只给一个总 token 数，而按区块预算：

```rust
pub struct ContextBudget {
    pub total_tokens: u32,
    pub soul_tokens: u32,
    pub user_tokens: u32,
    pub task_evidence_tokens: u32,
    pub learner_state_tokens: u32,
    pub memory_tokens: u32,
    pub thread_tokens: u32,
    pub tool_reserve_tokens: u32,
    pub output_reserve_tokens: u32,
}
```

推荐初始比例：

| 区块 | 占输入预算 | 说明 |
|---|---:|---|
| Soul + hard policy | 5% | 稳定、极短 |
| Explicit User | 5% | 只放强相关偏好和目标 |
| Current Task Evidence | 35–50% | 当前题目和原始事实优先 |
| Learner State | 8–12% | 聚合而非全历史 |
| Retrieved Memory | 10–18% | 有来源、有去重 |
| Recent Thread | 15–25% | 动态压缩 |
| Tool reserve | 单独保留 | 避免工具结果挤爆上下文 |

在 Reading Review 中，当前题目证据优先级必须高于用户画像；在 General Dialogue 中，用户目标和近期记忆可提高比例。

## 13.5 Retrieval Query Plan

Context Compiler 先生成一个结构化查询计划：

```rust
pub struct ContextQueryPlan {
    pub exact_entities: Vec<EntityRef>,
    pub skill_ids: Vec<String>,
    pub memory_kinds: Vec<MemoryKind>,
    pub keywords: Vec<String>,
    pub time_range: Option<TimeRange>,
    pub include_recent_thread: bool,
    pub include_repeated_attempts: bool,
    pub include_teaching_preferences: bool,
    pub max_candidates: u32,
}
```

示例：用户在阅读结果页询问“为什么我又在第 14 题选错？”：

```json
{
  "exactEntities": [
    {"kind":"attempt","id":"attempt-123"},
    {"kind":"question","id":"q14"}
  ],
  "skillIds": ["reading.tfng.qualifier_scope"],
  "memoryKinds": ["semantic", "episodic", "procedural"],
  "keywords": ["qualifier", "scope", "true false not given"],
  "timeRange": {"days": 90},
  "includeRecentThread": true,
  "includeRepeatedAttempts": true,
  "includeTeachingPreferences": true,
  "maxCandidates": 40
}
```

## 13.6 检索顺序

第一阶段建议采用：

```text
1. 精确实体查询
2. skill_id / asset_id / question_id / attempt_id 元数据查询
3. FTS5 lexical search
4. 时间和状态过滤
5. 候选评分、去重、证据检查
6. token budget packing
```

不要第一阶段就把 embedding 作为主检索。IELTS 数据具有大量明确 ID、题型和技能 taxonomy，结构化过滤通常比纯向量相似度更可靠。

后续启用 embedding 时采用 hybrid retrieval：

```text
structured score
+ FTS5/BM25 score
+ embedding similarity
+ recency
+ confidence
+ evidence diversity
- redundancy
- staleness
- contradiction penalty
```

## 13.7 Memory Ranking 公式

建议初版确定性评分：

```text
rank =
    0.28 * task_relevance
  + 0.18 * entity_match
  + 0.14 * skill_match
  + 0.12 * confidence
  + 0.10 * recency
  + 0.08 * evidence_diversity
  + 0.06 * user_confirmed
  + 0.04 * pedagogical_value
  - 0.12 * redundancy
  - 0.10 * staleness
  - 0.20 * contradiction_risk
```

其中：

- `task_relevance`：与当前 task type 的映射；
- `entity_match`：attempt/question/asset 精确匹配；
- `skill_match`：当前问题技能与 memory skill tag 匹配；
- `confidence`：memory 的置信度；
- `evidence_diversity`：是否来自多个不同题目；
- `user_confirmed`：是否由用户明确确认；
- `contradiction_risk`：是否存在更新或相反证据。

这些权重应进入配置和 eval，而不是散落在 SQL 中。

## 13.8 去重和冲突处理

### 去重

两条 memory 若满足：

```text
相同 subject + predicate
且 normalized value 相同
且 evidence 高度重叠
```

只保留：

- 更高 confidence；
- 更近 verified_at；
- 更多 distinct evidence；
- 用户确认优先。

### 冲突

出现以下情况时，不应自动选择一条并隐藏另一条：

```text
Memory A: 用户偏好先给结论
Memory B: 用户偏好先逐步推理
```

Context Pack 应呈现：

```json
{
  "kind": "preference_conflict",
  "status": "uncertain",
  "candidates": [...],
  "instruction": "Do not assume; infer from current request or ask briefly."
}
```

如果新证据明确取代旧结论，应由 Memory Mutation 将旧条目标记为 superseded，而不是在检索阶段永久忽略。

## 13.9 Context Pack 数据结构

```rust
pub struct ContextPack {
    pub request_id: String,
    pub snapshot_id: String,
    pub task: AgentTaskKind,
    pub soul: Vec<ContextItem>,
    pub explicit_user: Vec<ContextItem>,
    pub current_evidence: Vec<ContextItem>,
    pub learner_state: Vec<ContextItem>,
    pub memories: Vec<ContextItem>,
    pub thread: Vec<ContextItem>,
    pub warnings: Vec<ContextWarning>,
    pub token_estimate: u32,
    pub compiler_version: String,
}

pub struct ContextItem {
    pub id: String,
    pub source_kind: ContextSourceKind,
    pub source_id: String,
    pub content: String,
    pub confidence: Option<f32>,
    pub evidence_ids: Vec<String>,
    pub relevance_score: f32,
    pub token_estimate: u32,
    pub sensitivity: SensitivityClass,
}
```

模型不必看到所有内部字段。Compiler 将其渲染为清晰分区：

```text
[Product Policy]
...

[User-confirmed Preferences]
...

[Current Task Evidence — canonical]
...

[Learner State — probabilistic, do not overstate]
...

[Relevant Past Experiences]
...

[Current Conversation]
...
```

## 13.10 Context Snapshot

每个重要模型调用都应保存：

- compiler version；
- query plan；
- 选中的 source IDs；
- 每项 score；
- 截断原因；
- token 估计；
- Prompt version；
- model/provider；
- 输出结果或调用 trace ID。

默认不必重复保存完整敏感正文，可保存：

- source reference；
- content hash；
- redacted preview；
-必要时 encrypted snapshot。

这样可以回答：

> 为什么 Agent 这次认为用户在 Heading 上存在问题？

而不是只能猜测当时模型看到了什么。

## 13.11 Context Compiler 伪代码

```rust
pub async fn build_context(
    req: BuildContextRequest,
    stores: &ContextStores,
    tokenizer: &dyn TokenEstimator,
) -> Result<ContextPack, ApplicationError> {
    let task = resolve_task(&req)?;
    let plan = build_query_plan(&req, task, stores)?;

    let current = stores.learning.load_current_evidence(&plan)?;
    let explicit_user = stores.profile.load_explicit_relevant(&plan)?;
    let learner = stores.learner.load_skill_states(&plan.skill_ids)?;
    let memory_candidates = stores.memory.search(&plan)?;
    let thread = stores.thread.load_recent(req.thread_id.as_deref(), &plan)?;

    let normalized = normalize_candidates(
        current,
        explicit_user,
        learner,
        memory_candidates,
        thread,
    );

    let safe = normalized
        .into_iter()
        .filter(memory_is_active)
        .filter(no_unresolved_security_quarantine)
        .map(mark_epistemic_status)
        .collect::<Vec<_>>();

    let ranked = rank_and_deduplicate(safe, &req, &plan);
    let packed = pack_by_section_budget(ranked, &req.token_budget, tokenizer)?;
    let warnings = detect_conflicts_and_missing_evidence(&packed);

    let snapshot = stores.context.persist_snapshot(
        &req,
        &plan,
        &packed,
        &warnings,
    )?;

    Ok(ContextPack::from_snapshot(snapshot, packed, warnings))
}
```

## 13.12 Compaction

当前 thread 超过预算时，不应直接截断最老消息。建议：

```text
最近 N 轮原文
+ 当前未完成工具调用
+ 当前任务目标
+ 已确认结论
+ 未解决问题
+ 必须保留的实体 ID
+ 其余历史的结构化摘要
```

Compaction 结果必须是 thread checkpoint，不应自动成为长期 Memory。只有后续 Memory Extractor 判断其有跨会话价值时，才形成 memory candidate。

## 13.13 Context Compiler 的初始验收指标

| 指标 | 初始目标 |
|---|---:|
| 当前题目关键证据召回率 | ≥ 99% |
| 用户明确偏好在相关任务召回率 | ≥ 95% |
| 不相关长期记忆注入率 | ≤ 10% |
| 已 superseded memory 注入率 | 0% |
| 无来源推断注入率 | 0% |
| Context token 超预算率 | 0% |
| 相同内容重复占用率 | ≤ 5% |
| 对同输入的确定性 pack 差异 | 0，除非数据版本变化 |

---

# 14. Agent Runtime、状态、工具与权限

## 14.1 保留现有 Agent Loop

当前 `AgentService` 已经具备：

- model/tool 多轮循环；
- run 和 tool call 持久化；
- round/tool call 上限；
- tool call ID 验证；
- usage 汇总；
- interrupted recovery；
- tool result 回注模型。

因此本计划不建议引入 LangGraph、OpenAI Agents SDK 或其他运行时替代当前 Rust loop。它们只作为设计参考。

下一步应在现有 loop 上增加：

1. thread/session；
2. context snapshot；
3. checkpoint 和 resume；
4. cancellation；
5. approval；
6. tool policy；
7. model invocation trace；
8. background run；
9. response streaming；
10. memory hooks。

## 14.2 Run 与 Thread 分离

当前一个 `agent_run` 基本对应一次用户请求。需要增加：

```text
agent_thread
  1 ── * agent_run
agent_run
  1 ── * model_invocation
  1 ── * tool_call
  1 ── * context_snapshot
```

Thread 负责：

- 持续对话身份；
- 最近消息；
- thread summary；
-当前 task context；
- 当前页面/attempt/asset 绑定。

Run 负责：

- 一次明确执行；
- 使用的模型、Prompt 和 Context；
- tool calls；
- 结果、费用、错误和完成状态。

## 14.3 建议状态

```rust
pub enum AgentRunStatus {
    Queued,
    BuildingContext,
    Running,
    WaitingApproval,
    Cancelling,
    Completed,
    Failed,
    LimitExceeded,
    Interrupted,
    Cancelled,
}
```

不需要第一阶段实现复杂 DAG 状态机，但状态必须足够表示：

- 还没开始；
- 正在构建 Context；
- 正在调用模型或工具；
- 等待用户授权；
- 用户要求取消；
- 进程重启中断；
- 可以安全重试。

## 14.4 Checkpoint

建议每轮模型调用后、每个有副作用工具前后保存 checkpoint：

```rust
pub struct AgentCheckpoint {
    pub id: String,
    pub run_id: String,
    pub round: u32,
    pub phase: CheckpointPhase,
    pub message_state: Vec<StoredAgentMessageRef>,
    pub pending_tool_calls: Vec<AgentToolCall>,
    pub context_snapshot_id: String,
    pub usage: TokenUsage,
    pub created_at: DateTime<Utc>,
}
```

初期恢复策略可保守：

- 进程重启后将 in-flight network/tool call 标为 interrupted；
- 用户可以“从上一个安全 checkpoint 重试”；
- 不自动重放写工具；
- read-only 工具允许重新执行；
- mutation 工具必须依赖 idempotency key 和 expected version。

## 14.5 Tool Registry

工具不应由一个巨大的 `match tool_name` 长期扩张。建议：

```rust
pub trait AgentTool: Send + Sync {
    fn spec(&self) -> AgentToolSpec;
    fn policy(&self) -> ToolPolicy;
    async fn validate(
        &self,
        ctx: &ToolExecutionContext,
        arguments: Value,
    ) -> Result<ValidatedArguments, ToolRejection>;
    async fn execute(
        &self,
        ctx: &ToolExecutionContext,
        arguments: ValidatedArguments,
    ) -> Result<ToolOutput, ToolError>;
}

pub struct AgentToolRegistry {
    tools: HashMap<ToolName, Arc<dyn AgentTool>>,
}
```

## 14.6 Tool Permission Class

```rust
pub enum ToolEffect {
    ReadOnly,
    ProposalOnly,
    ReversibleWrite,
    IrreversibleWrite,
    ExternalSideEffect,
}

pub struct ToolPolicy {
    pub effect: ToolEffect,
    pub requires_user_approval: ApprovalRule,
    pub allowed_run_kinds: Vec<AgentRunKind>,
    pub max_result_bytes: usize,
    pub timeout_ms: u64,
    pub sensitivity: SensitivityClass,
    pub idempotency_required: bool,
}
```

建议初始批准矩阵：

| 工具类型 | 默认行为 |
|---|---|
| 读取学习记录 | 自动 |
| 读取用户画像/Memory | 自动，但记录 trace |
| 搜索 Coach 历史 | 自动 |
| 生成 memory proposal | 自动 |
| 激活/删除 memory | 用户可配置：初期必须确认 |
| 创建学习计划 proposal | 自动 |
| 修改正式学习计划 | 确认 |
| 修改题库、答案或分数 | 永不允许 Agent |
| 修改 Soul / 全局 Prompt | 生产 Agent 永不允许 |
| 外部网络或文件写入 | 明确授权 |

## 14.7 第一批 Learning Read Tools

### `get_attempt_detail`

输入：

```json
{"attemptId":"attempt-123"}
```

输出应为紧凑的 canonical view，而不是整张表 dump：

```json
{
  "attempt": {...},
  "questions": [...],
  "score": {...},
  "timelineSummary": {...},
  "evidenceVersion": 1
}
```

### `compare_attempts_for_asset`

```json
{
  "assetId":"p2-high-14",
  "limit":5,
  "minimumGapHours":12
}
```

返回：

- attempt timeline；
- question-level transitions；
- first-try correctness；
- repeat familiarity warning；
- corrected/newly wrong/still wrong；
- answer change metrics；
-不直接输出自由文本结论。

### `get_question_history`

按 canonical question key 查询跨 attempt 证据。

### `get_skill_state`

返回 mastery、uncertainty、evidence count、distinct assets、last observed 和解释。

### `search_learning_events`

支持：

- event type；
- skill；
- activity；
- date；
- asset；
- attempt；
- result limit。

### `search_coach_history`

使用 FTS5 + thread/date/attempt filter。

### `search_memory`

只能返回 active、非隔离、满足 sensitivity policy 的 memory。

### `get_daily_journal`

用于用户询问“今天我主要学到了什么”。

## 14.8 第一批 Memory Proposal Tools

不要直接提供通用 `write_memory`。建议工具语义化：

```text
propose_memory_create
propose_memory_replace
propose_memory_merge
propose_memory_archive
propose_user_preference_update
```

每个 proposal 必须包含：

```json
{
  "kind":"semantic",
  "subject":"user:default",
  "predicate":"teaching.preference",
  "value": {...},
  "reason":"...",
  "evidenceIds":["event-1","coach-message-7"],
  "confidence":0.78,
  "expectedCurrentVersion":3
}
```

真正的 mutation 由 Memory Service 完成确定性校验。

## 14.9 Tool Result 最小化

工具返回给模型和保存到审计表中的内容要分开：

```rust
pub struct ToolOutput {
    pub model_payload: Value,
    pub audit_payload: Value,
    pub ui_payload: Option<Value>,
    pub sensitivity: SensitivityClass,
    pub truncated: bool,
}
```

例如 `read_file` 可以给模型正文，但审计只保存 hash、路径、字节数；学习工具同理，避免把大量作文、题目原文或用户隐私重复复制进 `agent_tool_calls.result_json`。

## 14.10 输入与输出 Guardrail

### 工具输入 Guardrail

- JSON Schema validation；
- deny unknown fields；
- ID 和 scope 校验；
- 路径/越权检查；
- read/write effect 检查；
- approval 检查；
-参数大小限制；
- idempotency key；
- expected version；
- Prompt injection marker 检测。

### 工具输出 Guardrail

- 最大字节数；
- 敏感字段删除；
- source/evidence ID 必须存在；
- 输出 schema；
- 不把工具内容自动当作 system instruction；
- 对外部/用户导入文本标记为 untrusted data。

## 14.11 Cancellation

建议运行时持有：

```rust
pub struct AgentRunControl {
    cancellation: CancellationToken,
    current_phase: AtomicRunPhase,
}
```

行为：

- 用户取消时先把 DB 状态变为 `cancelling`；
- cancellation token 传给 model request 和 tool executor；
- reqwest future 通过 `select!` 被丢弃；
- 当前工具如不可中断，完成后不得继续下一轮；
- 最终状态 `cancelled`；
- mutation 工具依赖事务和 idempotency，避免半写。

## 14.12 Agent Run Kind

不要使用一个 system prompt 处理所有场景：

```rust
pub enum AgentRunKind {
    WorkspaceAssistant,
    LearningCoach,
    AttemptReview,
    MemoryManager,
    DailyJournal,
    DreamConsolidation,
    StudyPlanner,
    EvolutionEvaluator,
}
```

每种 run kind 具有：

- 独立 Soul 模块；
- 工具 allowlist；
- context budget；
- model policy；
- max rounds；
- output schema；
- approval policy；
- retention policy。

## 14.13 后台运行

`DailyJournal`、`DreamConsolidation` 和 eval 不应依赖 Vue 页面打开。建议加入持久化 job queue：

```text
background_jobs
  queued → claimed → running → completed/failed/dead
```

Tauri 启动后 worker：

- 恢复 stale claimed jobs；
- 仅在应用空闲、非考试计时、非高 CPU 场景运行；
- 可被用户暂停；
- battery saver 下不运行重任务；
- 所有 job 有最大时长和重试次数。

桌面应用退出时，不承诺后台常驻。错过的 nightly job 可在下次启动后补跑，但应使用原 date window，避免重复。

## 14.14 Agent 工作台定位

现有工作台建议保留，但重新定义为：

- Agent 能力调试和高级交互入口；
- 查看 run、tool call、Context Pack 和输出；
- 用户管理工作区文件；
- 手动触发分析、Memory 审阅或学习计划；
- 开发期间用于观测 Agent。

它不应成为个性化系统的唯一入口。阅读页和写作页应直接调用场景化 Agent run kind。

---

# 15. AI Coach 个性化与教学策略演化

## 15.1 AI Coach 的目标函数

AI Coach 不应只优化“回答看起来聪明”。建议目标分成四层：

1. **事实正确**：引用题目和文章证据准确；
2. **诊断正确**：识别用户错误原因，而不是只复述正确答案；
3. **教学适配**：解释方式适合当前用户；
4. **学习有效**：后续新题或间隔复测表现改善。

优先级必须是：

```text
正确性 > 安全与不泄题 > 诊断 > 教学适配 > 风格偏好
```

用户偏好不能覆盖事实和教学原则。例如用户喜欢直接给答案，不代表系统应在练习态立即泄露答案。

## 15.2 Coach Interaction Event

每次 Coach 交互应产生结构化事件：

```rust
pub struct CoachInteractionEvent {
    pub thread_id: String,
    pub attempt_id: Option<String>,
    pub question_ids: Vec<String>,
    pub run_id: String,
    pub user_message_id: String,
    pub assistant_message_id: String,
    pub task_kind: CoachTaskKind,
    pub context_snapshot_id: String,
    pub prompt_version: String,
    pub response_strategy: TeachingStrategyTag,
    pub explicit_feedback: Option<CoachFeedback>,
    pub behavioral_signals: Vec<BehavioralSignal>,
}
```

## 15.3 反馈信号强弱

### 强信号

- 用户点“有帮助/没帮助”；
- 用户明确纠正事实；
- 用户明确说“请先给例子”“不要这么抽象”；
- 用户编辑自己的教学偏好；
- 用户选择“以后都这样解释”；
- 后续不同题目表现改善/恶化。

### 中等信号

- 用户要求重新解释；
- 用户选中某段回答继续追问；
- 用户采用了 Coach 建议后完成一道新题；
- 用户对同一建议多次表示接受。

### 弱信号

- 消息停留时间；
- 复制文本；
- 展开详情；
- 对话长度；
- 是否立即离开。

弱信号不能单独产生长期画像。

## 15.4 不满意回答的分析

用户反复调整 Coach 不等于“原回答一定错”。系统应区分：

```text
factual_error
missing_evidence
wrong_diagnosis
explanation_too_abstract
explanation_too_long
answer_given_too_early
ignored_user_level
ignored_previous_preference
language_or_tone_mismatch
user_disagrees_but_model_may_be_correct
```

建议通过一个 `Coach Critique Extractor` 输出候选：

```json
{
  "critiqueType":"explanation_too_abstract",
  "targetMessageId":"msg-123",
  "userEvidence":"能不能给我一个具体句子例子",
  "suggestedPreference":{
    "predicate":"teaching.explanation.example_first",
    "value":true
  },
  "confidence":0.82,
  "persistence":"candidate"
}
```

## 15.5 教学策略表示

不要把程序性记忆只保存成自由文本 Prompt。建议结构化：

```rust
pub struct TeachingStrategy {
    pub id: String,
    pub scope: StrategyScope,
    pub trigger: StrategyTrigger,
    pub sequence: Vec<TeachingStep>,
    pub constraints: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub confidence: f32,
    pub status: StrategyStatus,
}
```

示例：

```json
{
  "scope":{"skillId":"reading.matching_headings"},
  "trigger":{"userState":"keyword_matching_bias"},
  "sequence":[
    {"kind":"ask_restate_main_idea"},
    {"kind":"identify_scope_words"},
    {"kind":"contrast_top_two_options"},
    {"kind":"ask_user_commit_before_reveal"}
  ],
  "constraints":["do_not_reveal_answer_before_attempt"],
  "confidence":0.76
}
```

## 15.6 Coach Request Pipeline

```text
Vue 触发场景化 Coach 请求
        ↓
读取当前题目和 attempt canonical evidence
        ↓
Context Compiler 加载 learner state + relevant memories
        ↓
选择 teaching strategy candidate
        ↓
模型生成结构化回答
        ↓
输出 validator 检查题号、证据、答案状态和字段
        ↓
UI 渲染 + feedback capture
        ↓
写入 interaction event
        ↓
后续 Dream / outcome attribution
```

## 15.7 Coach 输出结构

```rust
pub struct CoachResponse {
    pub answer: String,
    pub diagnosis: Option<CoachDiagnosis>,
    pub evidence: Vec<PassageEvidence>,
    pub next_step: Option<CoachNextStep>,
    pub self_check: Option<SelfCheckQuestion>,
    pub uncertainty: Option<String>,
    pub strategy_id: Option<String>,
    pub context_snapshot_id: String,
}
```

Reading Review 推荐：

```text
1. 你当时使用了什么判断
2. 原文中最关键的证据
3. 干扰项为什么看似合理但不成立
4. 你这次应修改的判断规则
5. 一道很短的自检问题
```

不是每次都必须五段。Context 和策略决定是否简化。

## 15.8 Outcome Attribution

某次 Coach 回答后，系统不能立即断言策略有效。建议建立：

```text
coach_intervention
  → linked skill/question pattern
  → outcome window
  → later observations
  → attribution score
```

基础归因：

```text
后续不同资产同技能表现
- intervention 前 baseline
- 时间衰减
- 题目难度差异
- 是否为同题重复
- 是否存在其他 intervention
```

初期只做弱因果表述：

> 在使用“先概括段落主旨”的讲解后，你接下来三篇新材料的 Heading 正确率从 45% 提升到 67%。这是一项正向信号，但样本仍少。

禁止表述为“该策略导致提升”，除非未来有更强实验设计。

## 15.9 Coach 个性化更新规则

```text
一次明确偏好 → explicit preference，立即可用
一次不满意 → candidate，仅当前 thread 或短期有效
跨三次明确反馈 → 提升 confidence
跨不同题目 + 后续结果改善 → procedural memory candidate
长期未验证 → confidence decay
新证据相反 → replace/merge proposal
用户手动更正 → 最高优先级并保留审计
```

## 15.10 防止迎合用户

Agent 贴近用户的思考方式，不等于复刻用户错误思路。

System policy 应规定：

- 适配表达层，不牺牲事实；
- 对稳定错误模式给出温和但明确挑战；
- 不把用户一次陈述保存成能力事实；
- 不用画像给用户贴固定标签；
- 使用“当前证据显示”“最近样本提示”；
- 为高不确定判断主动设计验证题。

## 15.11 Coach Prompt 模块化

```text
coach/core_soul
coach/safety
coach/task/reading_question
coach/task/attempt_review
coach/task/writing_revision
coach/context_schema
coach/response_schema
coach/teaching_strategy/<strategy-id>
coach/user_preferences
```

不要把所有内容生成一个不可维护的巨型字符串。Prompt Renderer 记录每个模块版本和最终 hash。

## 15.12 Coach 验收

- 回答引用正确题目和原文；
- 不使用不存在的用户画像；
- 可以解释所用画像来源；
- 用户删除 memory 后不再注入；
- 同样的 current evidence + frozen snapshot 可重放；
- 用户反馈可形成候选但不立即污染长期策略；
- 后续结果能回链到 intervention；
- 无 AI 时练习和历史功能不受影响。

---

# 16. 产品级 Prompt、Skill 与工具描述自进化

## 16.1 三种演化必须分开

| 演化对象 | 范围 | 写入者 | 生效门槛 |
|---|---|---|---|
| 用户 Memory | 单用户 | Dream/Memory Service | 校验、用户控制 |
| 用户教学策略 | 单用户 | Strategy Learner | 多证据和 outcome |
| 产品 Prompt/Skill | 全体用户 | 开发者演化管线 | eval、holdout、审批、发布 |

禁止把三者混成同一个“Agent 自己修改 Prompt”。

## 16.2 Skill 的定义

本产品中的 Skill 不应只是一段系统提示。建议一个 Skill 包含：

```text
skill.yaml
prompt.md
tools.json
response.schema.json
evaluators/
fixtures/
README.md
```

```yaml
id: reading-attempt-review
version: 3
run_kind: attempt_review
allowed_tools:
  - get_attempt_detail
  - compare_attempts_for_asset
  - get_skill_state
  - search_memory
context_profile: reading_review_v2
response_schema: review_v3
risk_class: read_only
```

## 16.3 Prompt Registry

生产运行时从版本化 registry 加载：

```rust
pub struct PromptVersion {
    pub id: String,
    pub prompt_key: String,
    pub version: u32,
    pub content: String,
    pub content_hash: String,
    pub status: PromptStatus,
    pub parent_version_id: Option<String>,
    pub created_by: String,
    pub eval_report_id: Option<String>,
}
```

状态：

```text
draft → evaluated → shadow → canary → active → retired → rolled_back
```

生产 Agent 只可读取 `active`；不得调用工具改写表。

## 16.4 Evolution Dataset

从生产 trace 提取的是候选，不是直接训练集：

```text
用户明确负反馈
工具调用失败
重复追问
输出 schema 失败
错误引用
上下文检索失败
高 token/长回路
用户手工修正
后续学习结果恶化
```

每条 candidate 经过：

- 去除敏感数据；
- 用户 opt-in；
- 脱敏；
- 去重；
- 失败分类；
- 人工或规则确认；
- split assignment。

## 16.5 Eval Dataset Split

```text
train/dev traces：候选生成器可见
validation：用于迭代选择
holdout：候选生成器和调参过程不可见
red-team：安全与极端输入
longitudinal：需要跨时间结果的样本
```

同一用户、同一题目或高度相似事件必须放在同一 split，避免数据泄漏。

## 16.6 Grader 组合

### 确定性 grader

- JSON schema；
- 必填字段；
- question ID 覆盖；
- citation/evidence ID 存在；
- tool allowlist；
- token/round/tool limit；
- 不泄露答案；
- 不产生非法 mutation；
- Prompt 大小和 forbidden phrase。

### 规则/领域 grader

- IELTS scoring range；
- correct answer consistency；
- task type；
- passage evidence alignment；
- learner state 不被表述为确定事实。

### LLM grader

- 诊断质量；
- 教学清晰度；
- 适配性；
- 是否过度迎合；
- 是否有不支持的推断。

LLM grader 必须：

- 使用不同 Prompt；
- 最好使用不同模型族；
- 多次 trial；
- 校准人工样本；
- 不单独决定发布。

### 人工 grader

重点审：

- 高风险 Prompt；
- safety regression；
- 教学策略变化；
- 高分但语义可疑候选；
- holdout borderline cases。

## 16.7 候选生成

候选来源可包括：

- 人工编辑；
- LLM critique + rewrite；
- GEPA/DSPy；
- 基于失败 taxonomy 的模板修改；
- tool description 精炼；
- context ranking 参数搜索。

GEPA 仅作为候选生成器：

```text
Candidate Generator ≠ Evaluator ≠ Release Authority
```

## 16.8 演化伪代码

```python
def evolve_prompt(prompt_key: str, dataset_id: str):
    baseline = registry.get_active(prompt_key)
    dataset = datasets.load(dataset_id)

    baseline_report = evaluator.run(
        prompt=baseline,
        split="validation",
        repeated_trials=3,
    )

    candidates = candidate_generators.generate(
        baseline=baseline,
        failure_clusters=baseline_report.failure_clusters,
        train_examples=dataset.train,
    )

    survivors = []
    for candidate in candidates:
        if not static_constraints.pass_all(candidate):
            continue
        report = evaluator.run(candidate, split="validation", repeated_trials=3)
        if promotion_rules.beats_baseline(report, baseline_report):
            survivors.append((candidate, report))

    finalists = pareto_select(survivors, dimensions=[
        "correctness", "safety", "learning_value",
        "latency", "tokens", "tool_efficiency"
    ])

    for candidate, _ in finalists:
        holdout = evaluator.run(candidate, split="holdout", repeated_trials=5)
        redteam = evaluator.run(candidate, split="red_team", repeated_trials=3)
        if release_gate.pass_all(holdout, redteam):
            registry.create_shadow(candidate)
```

## 16.9 Shadow 与 Canary

### Shadow

真实请求仍由 active 版本回答用户；候选版本在后台或采样环境运行：

- 不执行写工具；
- 不把输出展示给用户；
- 使用相同 frozen context snapshot；
- 比较答案、工具选择、成本和 grader 分数。

### Canary

满足以下条件才小比例启用：

- 用户 opt-in；
- 仅低风险 read-only run；
- 有 kill switch；
- 监控 schema failure、负反馈和成本；
- 一旦安全或正确性回归立即回滚。

## 16.10 Promotion Gate

候选必须满足：

```text
正确性不下降
安全零关键回归
学习价值显著或至少不下降
工具失败率不升
Prompt injection 成功率不升
P95 延迟在预算内
平均 token 成本在预算内
人工审阅通过
版本和 hash 固化
```

不能用单一加权总分掩盖关键维度下降。建议采用硬门槛 + Pareto selection。

## 16.11 个体策略演化与全局 Prompt 的关系

用户级教学策略是 Context 中的数据，不应直接修改全局 Prompt。

例如：

```text
全局 Prompt：根据已批准的 TeachingStrategy 执行个性化教学。
用户策略：面对 Heading 错误，先让用户复述主旨再对比选项。
```

当大量用户都受益于同一策略时，可以通过离线聚合形成全局 Skill candidate，但必须：

- 匿名化；
- opt-in；
- 聚合阈值；
- 不把某个用户的私人数据写入全局 Prompt；
- 经过产品 eval 管线。

## 16.12 工具描述演化

工具描述会显著影响模型是否正确调用工具，因此也要版本化和评测：

- 正确工具选择率；
- 缺少必要工具率；
- 多余工具调用率；
- 参数 schema 合法率；
- 工具循环率；
- 是否误解 read/write effect。

工具真实权限不由描述决定，必须由 Rust Policy enforcement 决定。

## 16.13 自进化最低可行版本

第一版不要直接部署 GEPA。先实现：

1. Prompt registry；
2. trace/eval dataset；
3. deterministic evaluator；
4. baseline replay；
5. 人工 candidate；
6. holdout；
7. shadow；
8. rollback。

当这条链稳定后，再加入自动候选生成器。

---

# 17. Rust Application API 与 Tauri 接口设计

## 17.1 依赖方向

推荐依赖关系：

```text
ielts-domain
     ↑
ielts-db          ielts-application
     ↑                 ↑
     └──── src-tauri adapters ────┘
                    ↑
                 Vue IPC
```

当前 `ielts-application` 仍直接引用部分 `ielts-db` DTO。短期可以接受，这是降低迁移风险的选择；中期应逐步把真正的 application command/result 移入 `ielts-application` 或 `ielts-domain`，使 application 不依赖具体持久化类型。

禁止形成：

```text
ielts-application → Tauri
ielts-application → reqwest
ielts-application → Keyring
ielts-application → raw rusqlite::Connection
```

## 17.2 新增 Application 模块

建议按实施顺序新增：

```text
crates/ielts-application/src/
  context/
    mod.rs
    compiler.rs
    ranking.rs
    budget.rs
  memory/
    mod.rs
    service.rs
    mutation.rs
    retrieval.rs
  learner/
    mod.rs
    observation.rs
    state.rs
    comparison.rs
  journal/
    mod.rs
    service.rs
  dream/
    mod.rs
    service.rs
    validator.rs
  agent/
    service.rs
    checkpoint.rs
    policy.rs
  prompts/
    registry.rs
  evolution/
    contracts.rs
```

不要一次性移动现有文件。每个模块只在对应 Phase 建立。

## 17.3 Store Ports

### LearningEvidenceStore

```rust
pub trait LearningEvidenceStore: Send + Sync {
    fn get_attempt_detail(
        &self,
        attempt_id: &str,
    ) -> Result<AttemptEvidence, ApplicationError>;

    fn list_attempts_for_asset(
        &self,
        asset_id: &str,
        limit: u32,
    ) -> Result<Vec<AttemptEvidence>, ApplicationError>;

    fn search_events(
        &self,
        query: LearningEventQuery,
    ) -> Result<Vec<LearningEvent>, ApplicationError>;

    fn append_event(
        &self,
        event: NewLearningEvent,
    ) -> Result<LearningEvent, ApplicationError>;
}
```

### MemoryStore

```rust
pub trait MemoryStore: Send + Sync {
    fn search(
        &self,
        query: MemorySearchQuery,
    ) -> Result<Vec<MemoryCandidate>, ApplicationError>;

    fn get_active_profile(
        &self,
        user_id: &str,
    ) -> Result<ActiveProfile, ApplicationError>;

    fn propose_mutations(
        &self,
        run: NewMemoryMutationRun,
        proposals: Vec<MemoryMutationProposal>,
    ) -> Result<MemoryMutationRun, ApplicationError>;

    fn apply_mutation_batch(
        &self,
        batch: ApprovedMemoryMutationBatch,
    ) -> Result<MemoryMutationResult, ApplicationError>;

    fn list_user_visible(
        &self,
        query: UserMemoryQuery,
    ) -> Result<Vec<UserVisibleMemory>, ApplicationError>;
}
```

### LearnerModelStore

```rust
pub trait LearnerModelStore: Send + Sync {
    fn insert_observations(
        &self,
        observations: &[NewSkillObservation],
    ) -> Result<(), ApplicationError>;

    fn get_states(
        &self,
        skill_ids: &[String],
    ) -> Result<Vec<LearnerSkillState>, ApplicationError>;

    fn update_states(
        &self,
        updates: &[LearnerSkillStateUpdate],
    ) -> Result<(), ApplicationError>;
}
```

### ThreadStore

```rust
pub trait AgentThreadStore: Send + Sync {
    fn create_thread(&self, cmd: CreateAgentThread) -> Result<AgentThread, ApplicationError>;
    fn append_message(&self, cmd: AppendAgentMessage) -> Result<AgentMessageRecord, ApplicationError>;
    fn load_recent_messages(&self, thread_id: &str, limit: u32)
        -> Result<Vec<AgentMessageRecord>, ApplicationError>;
    fn save_checkpoint(&self, checkpoint: NewAgentCheckpoint)
        -> Result<AgentCheckpoint, ApplicationError>;
    fn load_latest_safe_checkpoint(&self, run_id: &str)
        -> Result<Option<AgentCheckpoint>, ApplicationError>;
}
```

### JobStore

```rust
pub trait BackgroundJobStore: Send + Sync {
    fn enqueue(&self, job: NewBackgroundJob) -> Result<BackgroundJob, ApplicationError>;
    fn claim_next(&self, worker_id: &str, now: DateTime<Utc>)
        -> Result<Option<BackgroundJob>, ApplicationError>;
    fn heartbeat(&self, claim: &JobClaim) -> Result<(), ApplicationError>;
    fn finish(&self, result: FinishBackgroundJob) -> Result<(), ApplicationError>;
    fn recover_stale(&self, before: DateTime<Utc>) -> Result<u32, ApplicationError>;
}
```

## 17.4 Application Services

### ContextCompilerService

```rust
pub struct ContextCompilerService<'a> {
    pub learning: &'a dyn LearningEvidenceStore,
    pub memory: &'a dyn MemoryStore,
    pub learner: &'a dyn LearnerModelStore,
    pub threads: &'a dyn AgentThreadStore,
    pub snapshots: &'a dyn ContextSnapshotStore,
    pub estimator: &'a dyn TokenEstimator,
}
```

### MemoryService

负责：

- CRUD 和用户控制；
- proposal validation；
- mutation transaction；
- supersession；
- source/evidence validation；
- capacity enforcement；
- confidence decay；
- export/delete。

### DreamService

负责：

- 选定 evidence window；
- 加载 active memory region；
- 生成 read-only evidence bundle；
- 调用 consolidator；
- 验证 replacement proposal；
- 保存 Dream report；
- 在 approval policy 下提交 mutation。

### LearnerModelService

负责：

- 事件→observation；
- observation weighting；
- state update；
- repeated attempt analysis；
- intervention outcome linking；
- stale state refresh。

### PersonalizedCoachService

负责：

- 构建 Coach task；
- Context Compiler；
- teaching strategy selection；
- model call；
- response validation；
- interaction event；
- explicit feedback。

## 17.5 Model Gateway

当前 `AiRuntime` 同时实现 `LanguageModel` 和 `AgentModel`。建议保留，但补充统一元数据：

```rust
pub struct ModelRequestMetadata {
    pub invocation_id: String,
    pub run_id: Option<String>,
    pub thread_id: Option<String>,
    pub feature: String,
    pub prompt_version_ids: Vec<String>,
    pub context_snapshot_id: Option<String>,
    pub response_schema_id: Option<String>,
    pub timeout: Duration,
    pub cancellation: CancellationToken,
}
```

模型端口：

```rust
#[async_trait]
pub trait ChatModel: Send + Sync {
    async fn complete(
        &self,
        request: CompletionRequest,
        metadata: ModelRequestMetadata,
    ) -> Result<CompletionResponse, ModelError>;
}

#[async_trait]
pub trait ToolCallingModel: Send + Sync {
    async fn respond(
        &self,
        request: AgentModelRequest,
        metadata: ModelRequestMetadata,
    ) -> Result<AgentModelResponse, ModelError>;
}
```

## 17.6 Model Invocation Store

建议表：

```sql
CREATE TABLE llm_invocations (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT,
  thread_id TEXT,
  feature TEXT NOT NULL,
  provider_config_id TEXT NOT NULL,
  provider_kind TEXT NOT NULL,
  requested_model TEXT NOT NULL,
  actual_model TEXT,
  prompt_bundle_hash TEXT NOT NULL,
  context_snapshot_id TEXT,
  response_schema_id TEXT,
  status TEXT NOT NULL,
  attempt_count INTEGER NOT NULL DEFAULT 1,
  input_tokens INTEGER,
  output_tokens INTEGER,
  cached_input_tokens INTEGER,
  latency_ms INTEGER,
  provider_request_id TEXT,
  error_code TEXT,
  error_retryable INTEGER,
  started_at TEXT NOT NULL,
  completed_at TEXT,
  FOREIGN KEY(run_id) REFERENCES agent_runs(id) ON DELETE SET NULL,
  FOREIGN KEY(context_snapshot_id) REFERENCES context_snapshots(id) ON DELETE SET NULL
);
```

默认不保存完整 raw prompt/response。可通过设置开启开发诊断，并做敏感字段裁剪。

## 17.7 Tauri Commands 设计原则

Tauri command 只负责：

1. deserialize；
2. auth/capability/desktop state；
3. 构造 adapter；
4. 调用 application service；
5. 将 application error 转为 envelope；
6. 转发 event/stream。

command 内不应：

- 拼接复杂 Prompt；
- 决定 memory ranking；
- 直接实现 Dream；
- 自行跨多次 DB 调用编排事务；
- 在 Vue 参数中接收 canonical answer key；
- 接收由 Vue 自由构造的完整用户画像。

## 17.8 新增 Tauri Commands

### Thread / Agent

```text
agent_thread_create
agent_thread_get
agent_thread_list
agent_thread_archive
agent_send_message
agent_cancel_run
agent_retry_run
agent_get_run
agent_get_run_trace
agent_get_context_snapshot
agent_approve_tool_call
agent_reject_tool_call
```

### Memory

```text
memory_list
memory_get
memory_search
memory_create_explicit
memory_update_explicit
memory_archive
memory_delete
memory_pin
memory_set_auto_learning_policy
memory_list_mutation_runs
memory_review_mutation_run
```

### Journal / Dream

```text
journal_get_day
journal_list
journal_generate_now
dream_get_latest
dream_list
dream_run_now
dream_review_proposals
dream_apply_proposals
dream_reject_proposals
```

### Learner Model

```text
learner_profile_get
learner_skill_states
learner_skill_timeline
learner_compare_attempts
learner_intervention_outcomes
```

### Prompt / Eval（开发者模式）

```text
prompt_registry_list
prompt_registry_get_active
eval_suite_list
eval_run_start
eval_run_get
```

生产 UI 不应暴露全局 Prompt 修改命令。

## 17.9 Event Channels

建议统一事件：

```rust
pub enum AgentUiEvent {
    RunStatusChanged { run_id, status },
    ContextBuilt { run_id, snapshot_id, item_count },
    ModelStarted { invocation_id, model },
    ModelDelta { invocation_id, delta },
    ToolRequested { run_id, call_id, name, approval_required },
    ToolStarted { call_id },
    ToolCompleted { call_id, status },
    MemoryProposalCreated { run_id, proposal_count },
    RunCompleted { run_id },
    RunFailed { run_id, error },
}
```

事件只是实时体验。权威状态仍在 SQLite；页面重挂载后应从 DB hydrate。

## 17.10 TypeScript 类型生成

建议使 Rust command/result 类型成为 TS 类型源：

- 使用 `ts-rs` 或单独 schema generator；
- CI 运行 generation；
- `git diff --exit-code` 检查 drift；
- 禁止同一 DTO 同时维护 `.d.ts` 手写副本；
- JSON Schema 用于工具和 Prompt 输出，同时生成 Rust/TS validator。

## 17.11 Error Taxonomy

```text
agent.invalid_request
agent.context_failed
agent.provider_failed
agent.tool_rejected
agent.tool_failed
agent.approval_required
agent.cancelled
agent.limit_exceeded
agent.interrupted
memory.validation_failed
memory.conflict
memory.capacity_exceeded
memory.security_quarantine
journal.generation_failed
dream.insufficient_evidence
dream.validation_failed
learner.insufficient_evidence
prompt.version_conflict
eval.gate_failed
```

Error 至少包含：

```rust
pub struct ApplicationError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub cause_id: Option<String>,
    pub context: Option<Value>,
}
```

UI 面向用户的文案与内部 message 分离。

## 17.12 事务边界

以下必须在单事务：

- Memory mutation batch + supersession + evidence links + mutation log；
- attempt submit + score + learning events；
- Coach assistant message + interaction event；
- Dream proposals approval + memory mutations + Dream status；
- learner observations + state update checkpoint；
- tool call terminal status + write-effect audit metadata；
- Prompt active version切换。

模型调用绝不能在事务内。

---

# 18. Vue 产品界面与交互设计

## 18.1 智能能力应嵌入主流程

最终 UI 不是新增一个万能聊天页，而是：

| 页面 | Agent 能力 |
|---|---|
| 阅读练习 | 选中文字解释、策略提示、禁止提前泄题 |
| 阅读结果 | 错因分析、跨尝试对比、复练建议 |
| 写作 | 规划、论点审查、修订建议、版本对比 |
| 写作结果 | 评分解释、反馈优先级、重写练习 |
| 历史 | 趋势、模式、干预效果、证据查看 |
| 词汇 | 错拼模式、复习建议、语境解释 |
| 首页 | 今日目标、待验证假设、日记摘要 |
| Memory Center | 画像和 Memory 治理 |
| Agent 工作台 | 高级对话、trace、workspace 文件工具 |

## 18.2 Agent 工作台改造

当前页面使用静态文件列表和 `setTimeout` 预览。改造顺序：

### 第一版

- 调用 `agent_pick_workspace`；
- 真正选择本地目录；
- 调用 `agent_send_message` 或现有 `agent_run`；
- 展示真实 run ID、rounds、tool call；
- 展示 `agent_get_run` 结果；
- 删除演示 timeout。

### 第二版

三栏布局：

```text
左：Thread / Context / Files
中：Conversation / Prompt / Approval
右：Run Trace / Tool Calls / Context Pack / Output
```

### 第三版

增加模式：

```text
Ask
Plan
Review Attempt
Review Memory
Developer Trace
```

“Prompt 编辑器”不应默认让普通用户修改 system prompt。普通用户输入的是 task instruction；真正 Prompt bundle 只在开发者模式显示版本和来源。

## 18.3 Memory Center

建议一级导航不一定直接显示“Memory”。可放在“我的学习”或“智能画像”中。

页面分区：

### `关于我`

- 用户明确目标；
- 当前分数目标；
-考试日期；
- 偏好语言；
- 解释风格；
- 用户手工编辑。

### `系统观察`

- 学习模式结论；
- confidence；
- evidence count；
- 最近验证；
- 查看证据；
- 确认/更正/删除。

### `有效方法`

- 对该用户有效的教学策略；
- 适用题型；
- 证据和后续表现；
- 可禁用。

### `近期经历`

- 代表性 episodic memory；
- 默认不显示全部事件。

### `自动学习设置`

- 关闭所有自动长期记忆；
- 只保存明确偏好；
- 允许学习错误模式；
- 允许学习 Coach 风格偏好；
- 允许匿名化产品改进；
- retention 和导出/清除。

## 18.4 Memory 条目 UI

```text
标题：你更容易接受“先看原文证据，再解释规则”的讲解
类型：教学偏好
状态：Developing
置信度：中等
证据：3 次明确反馈，2 个不同会话
最近验证：2026-08-08
[查看证据] [更正] [固定] [暂时禁用] [删除]
```

必须显示“系统推断”，不冒充用户明确声明。

## 18.5 Daily Journal UI

首页卡片：

```text
今天的学习记录
- 完成 2 篇阅读
- Matching Headings 出现 2 次范围判断错误
- 写作 Task 2 的段落主题句更稳定
- 待验证：先概括段落主旨是否能减少 Heading 错误
```

Journal 页面：

- 日期列表；
-事实摘要；
-新增候选 Memory；
-未解决问题；
- 明日建议；
- Dream 处理状态；
- 查看来源。

Journal 不应以拟人化方式宣称“AI 今天梦到了什么”作为主要产品文案。可以保留“Dream”内部概念，但用户界面建议用“每日整理”“智能反思”。

## 18.6 Dream Report UI

用户可查看：

```text
本次整理覆盖：8 月 3 日—8 月 9 日
读取：47 个学习事件、12 条 Coach 消息、6 条既有 Memory
候选：新增 2、合并 3、替换 1、归档 2、忽略 9
```

每个 proposal 展示：

- before；
- after；
- why；
- evidence；
- confidence；
-自动/手动；
- approve/reject/edit。

第一阶段全部 proposal mode；后续只有低风险重复合并和过期归档可自动应用。

## 18.7 Learner Profile UI

不要只显示雷达图。推荐：

```text
Skill            State       Evidence   Diversity   Last practiced
TFNG Scope        Unstable    12         5 assets    2 days ago
Heading Main Idea Developing  8          4 assets    today
Writing Cohesion  Stable      10         7 essays    4 days ago
```

点击技能：

- timeline；
- 相关 attempt；
- 错误 taxonomy；
- repeated same-item vs transfer；
- uncertainty；
- Coach interventions；
- 当前推荐验证任务。

## 18.8 “为什么这样回答”面板

每个重要 Coach/Agent 回答提供可折叠说明：

```text
本次回答使用了：
- 当前第 14 题原文证据
- 最近 90 天 4 次 TFNG scope 错误
- 你确认的偏好：先给例子
- 当前技能状态：Unstable（中等证据）

未使用：
- 两条已过期记忆
```

普通用户不需要看到完整 system prompt；但可以看到数据来源和个性化因素。

## 18.9 反馈组件

不要只提供 👍/👎。建议快速反馈：

```text
有帮助
事实不对
没有回答我的问题
太抽象
太长
太早给出答案
不符合我的学习方式
```

并提供可选文本。

反馈写入结构化表，而不是只保存 message 文本。

## 18.10 Approval UI

高风险工具调用卡片：

```text
Agent 请求：更新教学偏好
变化：从“先给结论”改为“先给例子，再给结论”
依据：最近 3 次明确反馈
范围：仅当前用户
[批准一次] [始终允许此类更新] [拒绝] [编辑]
```

文件工具：

- 显示路径；
- diff；
- expected hash；
- 写入影响；
- 不把模型描述当成唯一依据。

## 18.11 Empty/Error/Offline

### 无 Memory

> 还没有足够证据形成长期画像。系统会先记录明确偏好，不会根据一次练习给你贴标签。

### AI 未配置

- 练习、判分、历史继续可用；
- Journal 可生成确定性摘要；
- Dream 和 Coach 标记为等待 AI；
- 不丢失 job。

### Dream 失败

- 原 Memory 不变；
- 显示失败原因；
- 可重试；
- 不把部分 proposal 自动提交。

## 18.12 Accessibility

- 所有自动状态使用 `role=status` 或 `aria-live`，避免重复朗读；
- tool approval 可键盘完成；
- diff 支持纯文本模式；
-图表有表格等价；
- Memory confidence 不只靠颜色；
- reduced motion；
- 长回答具有标题结构；
- screen reader 可知道回答依据。

## 18.13 UI Telemetry

本地产品也需要产品质量指标，但默认本机：

- 个性化解释面板是否被查看；
- Memory 更正/删除率；
- Dream proposal 接受率；
- feedback 类型；
- Coach re-ask rate；
- context failure；
- tool approval rate；
-用户关闭自动学习的原因。

用于匿名产品改进必须显式 opt-in。

---

# 19. 安全、隐私与记忆投毒防护

## 19.1 威胁模型

本系统新增长期 Memory 和 Tool Use 后，风险包括：

1. 用户或题目文本中的 Prompt Injection；
2. Agent 把恶意文本保存成长期 Memory；
3. Memory 在未来会话反复传播；
4. 文件工具越权或 symlink/path escape；
5. SQL read tool 过度暴露用户数据；
6. Agent 误改学习事实、答案或分数；
7. 模型输出包含隐私并进入 audit；
8. Dream 误合并、误删除；
9. Prompt 自进化 reward hacking；
10. 用户数据进入全局 eval/training；
11. 第三方模型 Provider 收到不必要的完整历史；
12. 备份或导出泄露敏感画像。

## 19.2 信任等级

```rust
pub enum TrustLevel {
    SystemPolicy,
    UserExplicit,
    CanonicalProductData,
    DerivedVerified,
    DerivedUnverified,
    ExternalContent,
    ModelGenerated,
    SecurityQuarantined,
}
```

Context 渲染时必须标记：

- `ExternalContent` 和 `ModelGenerated` 是 data，不是 instruction；
- 只有 `SystemPolicy` 可以控制工具和权限；
- Memory 不因被长期保存就自动成为高信任 instruction。

## 19.3 Memory Ingestion Firewall

任何进入长期 Memory 的模型生成文本经过：

1. schema validation；
2. evidence existence；
3. evidence scope；
4. PII/sensitivity classification；
5. injection scanner；
6. forbidden instruction pattern；
7. contradiction check；
8. capacity check；
9. approval policy。

风险内容进入 quarantine：

```text
status = security_quarantined
```

不会被 Context Compiler 检索。

## 19.4 Injection Pattern

检测目标不只包括“ignore previous instructions”，还包括：

- 要求未来 Agent 执行操作；
- 声称拥有 system 权限；
- 要求读取密钥或其他文件；
- 隐式工具调用命令；
- Base64/Unicode 混淆；
- HTML/Markdown 隐藏文本；
- 指示 Memory Manager 把其保存为规则；
- 自引用“永久记住以下系统指令”。

Scanner 不能单独依赖 LLM，应有确定性特征 + 模型 classifier + 人工抽查。

## 19.5 Memory 内容原则

Memory 应保存：

```text
用户相关事实、偏好、学习模式、代表性经历、有效教学方法
```

不保存：

```text
通用操作命令
工具权限指令
密钥
系统 Prompt
可执行代码
外部页面要求
无法追溯的模型指令
```

程序性 Memory 也只能描述允许的教学策略，不能改变系统安全政策。

## 19.6 SQL Tool 安全

不要给 Agent 通用 `execute_sql` 或 `query_sql`。

必须提供业务语义工具：

```text
get_attempt_detail
search_learning_events
get_skill_state
```

每个工具：

- 固定 SQL；
- 参数绑定；
- 行数限制；
- 字段 allowlist；
- 无 raw secret；
- 不返回 Keyring reference；
- 不允许 ATTACH/PRAGMA/写入。

## 19.7 学习事实保护

Agent 永远不得：

- 修改 correct answer；
- 修改原始用户答案；
- 修改得分以符合建议；
- 删除 attempt 证据以让画像看起来更准确；
- 把 Memory 结论写回 canonical fact；
- 覆盖原始 Coach 用户消息。

允许的纠错必须是正式产品用例，保留 before/after 和用户操作。

## 19.8 Tool Approval

### 无需审批

- 读取当前用户自己的学习数据；
- 生成分析；
- 搜索 Memory；
- 创建 proposal；
- 读取用户已授权 workspace 中的普通文件。

### 需要审批

- 写文件；
- 删除或替换 Memory；
- 修改显式用户偏好；
- 修改学习计划；
- 导出包含敏感画像的数据；
- 调用外部服务发送大规模用户历史。

### 永不提供

- 读取 Keyring 明文；
- 任意 shell；
- 任意 SQL；
- 修改产品 Prompt active version；
-修改答案和成绩。

## 19.9 Provider Data Minimization

发送给模型前：

- 当前任务只发送必要题目；
- 避免发送全部历史；
- 将用户真实姓名、邮箱等替换为本地 entity ID；
- Context Compiler 对 sensitivity 分类；
- 用户可以选择“仅本地模型处理画像”；
- 第三方 Provider 的数据政策在设置页可见。

## 19.10 本地加密

建议优先级：

1. API Key 继续 Keyring；
2. SQLite 文件遵循 OS 用户目录权限；
3. 敏感导出加密可选；
4. 若未来需要数据库静态加密，再评估 SQLCipher；
5. 不自行发明加密算法；
6. 加密密钥放 Keyring；
7. 加密不能破坏备份和恢复测试。

第一阶段不应因引入 Memory 就立即更换数据库引擎。

## 19.11 用户控制与隐私

用户必须可以：

- 查看系统记住了什么；
- 查看来源；
- 更正；
- 删除；
- 暂停某类自动学习；
- 清除所有派生 Memory 但保留练习记录；
- 清除练习记录同时选择是否重建画像；
- 导出；
- 关闭匿名产品改进。

删除语义：

```text
delete derived memory
≠ delete canonical evidence
```

但如果用户删除原始证据，依赖该证据的 Memory 应重新验证或归档。

## 19.12 Backup

备份必须包含：

- Memory 和 evidence links；
- Journal；
- Learner state 和 observation；
- Agent thread/run audit；
- Prompt registry 用户可配置部分；
- Background jobs 的安全状态。

不包含：

- API Key 明文；
- 临时 workspace grants；
- in-flight network token；
- 可重新生成的 model cache；
- security quarantine 原文可按用户策略决定。

恢复后：

- 所有 running run/job 标 interrupted/queued；
- Keyring availability 重新对账；
- FTS index 重建或校验；
- Memory evidence foreign key 检查；
- Prompt active version 检查。

## 19.13 Retention

建议：

| 数据 | 默认保留 |
|---|---|
| canonical attempts | 用户策略 |
| Coach transcript | 用户策略，默认长期 |
| Agent run audit | 90–180 天可配置 |
| Tool full model payload | 默认最小化，30 天或不存 |
| Context snapshot refs | 与 run audit 一致 |
| Daily Journal | 1 年或用户策略 |
| Active Memory | 直到 superseded/deleted |
| Archived Memory | 90–365 天或用户策略 |
| Dream raw evidence bundle | 不重复保存，保存 refs |
| Eval data | opt-in、脱敏、单独策略 |

## 19.14 安全测试

- Memory poisoning test corpus；
- Prompt injection through question text；
- Prompt injection through Coach history；
- Prompt injection through file tool；
- symlink/race/path escape；
- tool output oversized；
- malicious JSON arguments；
- approval bypass；
- stale expected version；
- interrupted mutation；
- backup restores quarantined state correctly；
- deleted memory never reappears without new evidence。

---

# 20. 评测体系、指标与发布门禁

## 20.1 评测层级

```text
L0 Schema / Unit
L1 Repository / Transaction
L2 Application Service
L3 Model Contract
L4 Agent Trace
L5 Product E2E
L6 Longitudinal Learning Outcome
L7 Security / Red Team
```

所有层都需要，而不是只有对话 benchmark。

## 20.2 Memory Evaluation

### Extraction

| 指标 | 定义 |
|---|---|
| Candidate Precision | 提取出的候选中真正值得长期保存的比例 |
| Candidate Recall | 人工标注应保存项被提取的比例 |
| Preference Attribution | 是否区分明确偏好和模型推断 |
| Evidence Validity | evidence ID 存在且支持结论 |
| PII Leakage | 不应保存的敏感数据进入 memory 的比例 |

### Consolidation

| 指标 | 目标 |
|---|---:|
| Duplicate reduction | 显著减少重复 |
| Unsupported mutation | 0 |
| Supersession correctness | ≥ 95% on gold set |
| User correction preservation | 100% |
| Active memory capacity violation | 0 |
| Quarantined item activation | 0 |

### Retrieval

| 指标 | 目标 |
|---|---:|
| Precision@5 | ≥ 0.80 初期 |
| Recall@10 | ≥ 0.90 关键记忆集合 |
| Explicit preference recall | ≥ 0.95 |
| Superseded recall | 0 |
| Irrelevant injection | ≤ 0.10 |
| Context budget overflow | 0 |

## 20.3 Context Evaluation Dataset

样本至少覆盖：

- 单次阅读错题；
- 同题多次尝试；
- 跨题型重复错误；
- 用户明确偏好；
- 偏好冲突；
- 画像已更正；
- 已删除 Memory；
- evidence 不足；
- 近期和长期目标冲突；
-多语言；
- 题目包含 injection；
- thread 很长；
- tool result 很大。

每个 case 标注：

```text
must include
should include
must exclude
budget
expected warnings
```

## 20.4 Learner Model Evaluation

### 离线

- same-item repeat 与 cross-item transfer 区分；
-状态更新单调性不要求，但解释必须合理；
- uncertainty 随多样证据下降；
- 重复同题贡献受限；
- 错误修正不会抹掉历史；
- time decay；
- calibration：预测 0.7 的样本约 70% 正确；
- 与简单 baseline 比较。

### 在线

- 推荐的复习任务完成率；
- 间隔后保持；
- 新题迁移；
- 自我判断校准；
- 过度练同题下降；
- 对不同水平用户公平。

## 20.5 Coach Evaluation

维度：

```text
factual_grounding
answer_key_consistency
evidence_alignment
diagnosis_quality
pedagogical_helpfulness
personalization_relevance
uncertainty_calibration
non_leakage
style_fit
future_learning_value
```

不能只测 BLEU/相似度或用户点赞。

## 20.6 Agent Tool Evaluation

| 指标 | 定义 |
|---|---|
| Tool selection accuracy | 是否选择正确工具 |
| Argument validity | schema 合法率 |
| Tool efficiency | 完成任务所需调用数 |
| Redundant call rate | 重复/无用调用 |
| Loop rate | 无进展循环 |
| Unauthorized attempt | 试图调用无权限工具 |
| Approval compliance | 需要审批时是否停下 |
| Recovery correctness | 中断后不重复副作用 |

## 20.7 Evolution Evaluation

候选 Prompt/Skill 至少比较：

```text
baseline vs candidate
across multiple trials
validation + holdout + red-team
correctness + safety + cost + latency
```

发布硬门槛示例：

| 维度 | 门槛 |
|---|---|
| Critical safety regression | 0 |
| Deterministic contract pass | 100% |
| Holdout factual score | 不低于 baseline |
| Learning-value grader | +3% 或非劣界内 |
| Tool failure | 不增加 > 0.5pp |
| P95 latency | 不增加 > 20%，或有明确价值 |
| Mean input tokens | 不增加 > 15%，或有明确价值 |
| Human review | 通过 |

## 20.8 Longitudinal Evaluation

真正验证“自进化”需要跨时间：

```text
T0: baseline skill state
T1: Agent intervention
T2: next different item
T3: delayed retrieval after gap
```

推荐 outcome：

- first-attempt correctness；
- answer changes；
-时间；
- confidence calibration；
- 是否需要 Coach；
- delayed retention；
- transfer to new asset。

## 20.9 用户满意度与学习效果的冲突

可能出现：

```text
回答更短，用户更喜欢，但学习保持下降
立即给答案，点赞上升，但自主解题下降
高度迎合，用户满意，但错误信念被强化
```

因此 dashboard 必须并列展示：

- satisfaction；
- correction rate；
- re-ask rate；
- learning outcome；
- delayed retention；
- safety。

## 20.10 Trace-based Evaluation

保存足够 trace 以回答：

- Context Compiler 选了什么；
- 模型调用几次；
- tool 为什么被选；
- tool 参数和结果摘要；
- 哪个 Prompt version；
- 哪条 Memory 影响回答；
- 输出 validator 是否降级；
- 用户后续结果如何。

## 20.11 测试目录

```text
developer/tests/
  memory/
    extraction_cases.jsonl
    consolidation_cases.jsonl
    retrieval_cases.jsonl
    poisoning_cases.jsonl
  context/
    compiler_goldens.jsonl
    budget_cases.jsonl
  learner/
    repeated_attempt_cases.jsonl
    calibration_fixtures.json
  agent/
    tool_choice_cases.jsonl
    approval_cases.jsonl
    interruption_cases.jsonl
  coach/
    reading_review_cases.jsonl
    personalization_cases.jsonl
  evolution/
    suites/
    holdout/
    red_team/
```

## 20.12 CI 分层

### 每个 PR

- cargo fmt/clippy/test；
- migration fresh + upgrade；
- TS typecheck；
- deterministic memory/context tests；
- schema generation drift；
- lightweight agent fake-model tests；
- no external model call。

### Nightly

- selected real-provider eval；
- repeated trials；
- context retrieval benchmark；
- security corpus；
- packaged Tauri E2E；
- memory/dream background jobs；
- cost report。

### Release

- full packaged E2E；
- backup/restore；
- migration from every supported product DB version；
- Prompt active version pinned；
- eval report attached；
- red-team pass；
- signed binaries/updater。

## 20.13 监控预算

本地日志和诊断报告应包含：

- job backlog；
- Memory active count；
- candidate count；
- Dream last success；
- retrieval latency；
- Context token；
- model latency/tokens/errors；
- tool failure；
- FTS health；
- DB size；
- backup size；
- quarantine count。

不应默认记录完整用户内容。

---

# 21. 逐阶段工程实施计划

## 21.1 实施总原则

本工程必须采用纵向切片，不采用“先建完所有表、再写所有后端、最后接 UI”的大爆炸模式。

每个阶段都必须满足：

```text
Schema / Data
    + Application Use Case
    + Tauri Adapter
    + 最小 Vue 可观察界面
    + Unit / Integration / Packaged E2E
    + Feature Flag / Rollback
```

统一规则：

1. 每个 migration 单独 PR；
2. 每个 Phase 有 feature flag；
3. 旧路径在新路径验收前保持工作；
4. 不在同一 PR 同时重做 UI 和核心数据逻辑；
5. 所有模型输出都有 fake-model 测试；
6. 生产核心功能不得依赖外部模型可用；
7. 所有 derived data 可重建；
8. 所有 write tool 有 idempotency；
9. 每个 Phase 结束形成 checkpoint tag 或稳定 commit；
10. 只有达到 DoD 后进入下一阶段。

## 21.2 Feature Flags

建议设置：

```text
agent_threads_v1
learning_event_ledger_v1
memory_manual_v1
memory_auto_candidates_v1
journal_v1
dream_proposal_v1
dream_auto_low_risk_v1
context_compiler_v1
personalized_coach_v1
learner_model_v1
prompt_registry_v1
evolution_shadow_v1
embeddings_v1
agent_action_tools_v1
```

Flags 可保存在 `settings` 的 `features` namespace，但 release build 可由编译配置确定默认值。

---

## M0：基线冻结、架构合同与可观测性

### 目标

在继续开发前，把当前 Agent backend 变成可重放、可测量的稳定基线。

### 范围

不新增 Memory 自动化；不改变 Agent 工具行为；不修改产品 Prompt 语义。

### 工作项

#### M0-01：固定基线

- 记录 branch tip、backend commit、Cargo.lock、migration version；
- 生成架构说明：
  - `ielts-domain`；
  - `ielts-db`；
  - `ielts-application`；
  - `src-tauri/ai`；
  - `src-tauri/agent`；
  - Vue API；
- 为当前 AgentService 建立 sequence diagram。

#### M0-02：真实接通 AgentWorkspacePage

- 删除静态 `files` 和 `setTimeout` preview；
- 新增 `agent-repository.ts/js`；
- 调用 `agent_pick_workspace`；
- 调用现有 `agent_run`；
- 用 `agent_get_run` hydrate；
- 展示工具调用和最终结果；
- 保留 UI 结构，不先重设计。

#### M0-03：模型调用 trace

先不加完整 `llm_invocations` 表，可扩展 run result：

- provider request ID；
- actual model；
- latency；
- usage；
- retry count；
- Prompt hash。

随后在 M2 或 M3 正式建表。

#### M0-04：Agent fake-model replay

增加 fixtures：

```text
model returns content
model calls read_file then content
model calls multiple tools
unknown tool
invalid arguments
duplicate call ID
max rounds
max tools
provider failure
store failure
interrupted run
hash conflict
path escape
```

#### M0-05：当前 Agent 安全基线

验证：

- workspace grant 过期；
- symlink containment；
- `.git`、secret 等敏感路径策略；
- write requires read hash；
- atomic write；
-最大文件；
- UTF-8；
- audit payload 不保存文件全文。

#### M0-06：Architecture Decision Records

新增：

```text
docs/architecture/adr/
  0001-local-first-agent-runtime.md
  0002-sqlite-canonical-derived-memory.md
  0003-agent-memory-not-product-truth.md
  0004-prompt-evolution-offline-only.md
  0005-two-product-lines-shared-content-ui-language.md
```

### 测试

- `cargo test --workspace`；
- Agent unit；
- packaged Tauri Agent workspace smoke；
- file tool security；
- Vue typecheck；
- screenshot。

### DoD

- Agent 工作台执行真实 run；
- run/tool call 可从 SQLite 重载；
- 当前功能无回归；
- 所有基线测试进入 CI；
- 形成 baseline eval report。

### Rollback

- 关闭 Agent route；
- 不影响 Reading/Writing/History；
- migration 未新增。

---

## M1：Learning Event Ledger 与只读学习工具

### 目标

建立从 canonical learning truth 到 Agent 的稳定只读证据接口。这是 Memory 和个性化之前的必需基础。

### Migration

`0012_learning_event_ledger.sql`

包含：

- `learning_events`；
- 索引；
- sensitivity / consolidation state；
- idempotency key。

### 设计原则

- canonical table 仍是事实；
- event 是规范化证据，不复制所有原始内容；
- event 可重建；
- 写入必须和源业务事务原子完成；
- 不允许 LLM 创建 canonical learning event。

### 工作项

#### M1-01：Event 类型注册

建立 Rust enum/registry：

```rust
pub enum LearningEventType {
    AttemptStarted,
    AnswerChanged,
    AttemptSubmitted,
    AttemptCompleted,
    ReadingQuestionOutcome,
    WritingEvaluationCompleted,
    CoachQuestionAsked,
    CoachResponseGenerated,
    CoachFeedbackProvided,
    VocabularyReviewCompleted,
    AnnotationCreated,
}
```

为每个 event 定义 schema version。

#### M1-02：Reading submit 事件

在现有 submit transaction 中追加：

- attempt completed；
-每题 outcome；
- question skill keys（没有 mapping 时可空）；
- attempt ordinal；
- gap；
- timeline summary。

#### M1-03：Writing event

- draft/submit 不必全部进入长期 ledger；
- evaluation completed 产生 criterion observation source；
- review degraded/failed 作为质量事件；
-保存 prompt/model version ref。

#### M1-04：Coach events

- user question；
- response；
- explicit feedback；
- re-ask linkage；
- 不在 ledger 重复保存全文，可保存 message ID、hash、structured signals。

#### M1-05：Event rebuild command（开发者）

```text
learning_events_rebuild
learning_events_verify
```

重建时：

- 使用 deterministic idempotency key；
- 不覆盖用户 explicit data；
- 输出差异报告。

#### M1-06：只读 Agent tools

实现：

```text
get_attempt_detail
compare_attempts_for_asset
get_question_history
search_learning_events
```

工具放入业务 registry，不放通用 SQL。

#### M1-07：Agent Run Kind

增加 `AttemptReview` run kind，只允许上述只读工具。

#### M1-08：最小 UI

在 Reading Result 增加：

- “比较历次练习”；
- Agent 读取工具 trace；
- deterministic comparison 表；
-模型解释作为补充。

### 伪代码

```rust
fn submit_reading_attempt_tx(
    tx: &Transaction,
    cmd: ReadingSubmitCommand,
) -> DbResult<ReadingSubmitResult> {
    let result = score_and_persist(tx, &cmd)?;

    append_learning_event(tx, NewLearningEvent::attempt_completed(&result))?;
    for question in &result.questions {
        append_learning_event(
            tx,
            NewLearningEvent::reading_question_outcome(&result, question),
        )?;
    }

    Ok(result)
}
```

### 测试

- submit 与 event 原子；
- idempotent retry 不重复 event；
- rebuild 等价；
- cascade/delete behavior；
- repeated attempt tool；
- tool output size；
- Agent cannot mutate events。

### DoD

- 主要练习事实都有 event projection；
- Agent 可以只读分析重复尝试；
- event 与 canonical 数据 consistency check 通过；
-无 Memory 仍能提供有价值的跨尝试分析。

### Rollback

- flag 关闭 event generation；
- 现有 submit 逻辑不依赖 event；
- event 表可保留未使用。

---

## M2：Agent Thread、Checkpoint、Cancellation 与调用追踪

### 目标

把单次 `agent_run` 升级为可持续、可恢复、可取消的对话执行基础。

### Migration

`0013_agent_threads_checkpoints.sql`

包含：

- `agent_threads`；
- `agent_messages`；
- `agent_checkpoints`；
- `llm_invocations`；
- 扩展 `agent_runs`；
- 必要索引。

### 工作项

#### M2-01：Thread service

- create/list/get/archive；
- thread kind 和 scope；
- sequence append；
- recent message load；
- thread summary slot。

#### M2-02：AgentService 输入升级

```rust
pub struct RunAgentCommand {
    pub run_id: String,
    pub thread_id: String,
    pub run_kind: AgentRunKind,
    pub user_message_id: String,
    pub context_snapshot_id: String,
    ...
}
```

兼容当前 workspace Agent 的 adapter。

#### M2-03：Checkpoint

在以下时间保存：

- Context built；
- 每次 model response 后；
- tool execution 前；
- tool execution 后；
- waiting approval；
- final response。

#### M2-04：Cancellation token

- `AgentRunRegistry` 只保存 process-local cancellation token；
- DB 保存 cancel request；
- command `agent_cancel_run`；
-模型和工具响应取消；
- restart 时 registry 丢失，DB recovery 标 interrupted。

#### M2-05：Resume 策略

第一版：

- interrupted run 不自动继续；
- `agent_retry_run` 从 safe checkpoint 创建新 child run；
- read-only tools 可重放；
- write tools不自动重放；
- parent run lineage。

#### M2-06：LLM invocation trace

每次 model call：

- begin invocation；
- retry count；
- completion；
- token/latency；
- error；
-不保存 API key 和全文。

#### M2-07：实时事件

- Tauri channel；
- SQLite hydrate fallback；
-页面重载后可继续查看。

#### M2-08：工作台线程 UI

- Thread list；
- 新建；
- 消息流；
- run 状态；
-取消；
- retry；
- tool trace。

### 关键不变量

```text
message sequence 唯一
run 必须属于 thread
checkpoint step 单调
terminal run 不新增 tool call
cancelled run 不进入下一轮
每个 model invocation 有 terminal status
```

### 测试

- crash recovery；
- cancel during model；
- cancel before tool；
- retry lineage；
- duplicate message；
- thread archive；
- channel closed；
- packaged page reload。

### DoD

- Agent 对话可跨页面/重启查看；
-取消有效；
- run trace 完整；
-没有副作用重复执行。

### Rollback

- 使用 legacy one-shot `agent_run` adapter；
- thread tables 不影响现有功能。

---

## M3：Explicit User Profile 与手动 Memory Core

### 目标

先建立用户可管理的 Memory 产品，不立即自动学习。

### Migration

`0014_memory_profile_core.sql`

包含：

- `explicit_user_preferences`；
- `memory_items`；
- `memory_evidence`；
- `memory_mutations`；
- active canonical unique index；
- capacity config。

### 工作项

#### M3-01：Memory domain

```rust
MemoryKind
MemoryStatus
MemoryScope
MemoryEvidenceRole
MemoryMutationOperation
MemorySensitivity
```

#### M3-02：Explicit preference service

- 用户直接设置；
-无需 confidence；
- source=`user`；
-作用域 global/feature/skill；
- edit/delete/disable；
- Context 中优先级最高。

#### M3-03：Memory CRUD

第一版允许：

- 用户手工创建；
-用户编辑；
-用户删除；
-用户固定；
-Agent 创建 proposal，但不能自动激活。

#### M3-04：Mutation validator

检查：

- canonical key；
- expected version；
- evidence IDs；
- scope；
- confidence；
- duplicate；
- conflict；
- capacity；
- injection；
- sensitivity。

#### M3-05：Context Profile Renderer

构建小型 always-visible profile：

```text
explicit goals
explicit communication preferences
pinned active memory
high-confidence teaching strategies
```

初期只提供 debug preview，不直接接 Coach。

#### M3-06：Memory Center UI

- explicit preferences；
- active/pending/archived；
- source；
- confirm/edit/delete；
- auto-learning disabled notice；
- capacity。

#### M3-07：Export/Delete

- Markdown + JSON export；
- 清除 derived memory；
-不删除 attempts；
- reset profile。

### 测试

- active canonical uniqueness；
- version conflict；
- delete/supersede；
- evidence FK；
- injection quarantine；
- user explicit priority；
- export/import；
- backup restore。

### DoD

- 用户能完整治理系统记忆；
- Agent 不能绕过 service 直接写 memory；
-所有 memory 有审计；
-删除后 Context preview 不再出现。

### Rollback

- feature flag 隐藏 Memory Center；
-不影响练习；
-删除 derived data 可安全重建。

---

## M4：Context Compiler v1

### 目标

让 Agent 从“固定 system + user prompt”升级为证据驱动上下文，但只使用 explicit profile、手动 memory、当前任务和 read tools。

### Schema

在 `0013` 或独立增量中加入：

- `agent_context_snapshots`；
- `agent_context_items`。

如严格遵循既定 migration，可放在 `0013_agent_threads_checkpoints.sql`。

### 工作项

#### M4-01：Task resolver

- surface hint；
- explicit task；
- route/entity；
- deterministic fallback；
-分类模型仅作为最后手段。

#### M4-02：Query plan

- entity refs；
- skill IDs；
- memory type；
- time range；
- max candidates。

#### M4-03：Retriever v1

- exact SQL；
-metadata filter；
- FTS 暂未启用可用 LIKE/structured；
- current task evidence。

#### M4-04：Ranking v1

- 配置化权重；
- deterministic；
-去重；
- conflict warning；
- budget packing。

#### M4-05：Snapshot

- source refs；
- ranks；
- score；
- inclusion reason；
- hash；
- rendered context；
-敏感内容策略。

#### M4-06：接 Workspace Agent

- workspace run 使用 Context Compiler；
-当前文件/用户 prompt 仍为主；
- explicit user/profile 可选注入。

#### M4-07：Why panel

工作台显示：

- context blocks；
- item sources；
- token；
- warnings。

### 测试

- goldens must include/exclude；
- budget；
- conflict；
- deleted/superseded；
- determinism；
- injection item excluded；
- snapshot replay。

### DoD

- 每次 Agent run 有 context snapshot；
- token 不溢出；
- explicit preference 只在相关 task 注入；
- why panel 可解释。

### Rollback

- `context_compiler_v1=false` 使用 legacy prompt；
- snapshot 不影响 run。

---

## M5：Session-close Extractor 与 Daily Journal

### 目标

开始自动形成候选，但不自动修改 active long-term memory。

### Migration

`0015_journal_dream_jobs.sql` 的第一部分：

- `daily_journals`；
- `background_jobs`；
- `memory_candidate_batches` 可复用 `dream_candidates` 或 memory items candidate。

### 工作项

#### M5-01：Job worker

- SQLite claim；
- heartbeat；
- stale recovery；
- dedupe；
- backoff；
- UI pause；
- no daemon promise。

#### M5-02：Session Close Job

触发：

- Coach thread 结束；
-写作评估结束；
-阅读 attempt 完成；
-应用空闲。

输出：

- factual summary；
- candidate preferences；
- candidate error patterns；
- unresolved questions；
-不生成 active memory。

#### M5-03：Deterministic Journal Builder

即使无 LLM，也生成：

- completed counts；
- scores；
- skill observations；
- time；
- newly added vocab；
- Coach count。

#### M5-04：LLM Journal Enrichment

LLM 只能：

-组织语言；
-提炼主题；
-提出待验证假设；

不能改变事实数值。

#### M5-05：Journal upsert

同一天重新生成时：

- 新 version；
-旧 version superseded；
-不尾部追加；
- source event range 固定；
- hash。

#### M5-06：首页 Journal Card

- 今日摘要；
-待验证；
-来源；
-生成状态；
-手动重跑。

### 测试

- no-LLM fallback；
- same day rerun；
- dedupe job；
- crash recovery；
- facts unchanged；
- Journal source coverage；
- sensitive data redaction。

### DoD

- 每日有可追溯摘要；
-系统开始产生 candidate，但 active memory 不自动改变；
-用户可查看和删除 Journal；
-失败不影响练习。

### Rollback

- worker flag 关闭；
- Journal 可重建；
- job 状态保留。

---

## M6：Dream Proposal Mode 与 Memory Consolidation

### 目标

实现按日/周的只读证据整合，但所有长期 Memory 变化先进入 proposal，用户或开发规则批准后生效。

### Migration

完成 `0015_journal_dream_jobs.sql`：

- `dream_runs`；
- `dream_candidates`；
- `background_jobs` 完整字段；
- 与 memory mutation 关联。

### 工作项

#### M6-01：Dream Scope Selector

输入：

```text
coverage window
pending learning events
recent journal
active memory region
recent user corrections
memory capacity
```

选择规则：

- 只读；
- 事件已 processed 不重复，除非 weekly consolidation；
- 每个 Dream 有明确 event ID list/hash；
- weekly 可读取 daily journal，但关键 promotion 必须回查原 event。

#### M6-02：Dream Run Kind

新增 `DreamConsolidation`：

- 工具只读 learning/memory；
-输出严格 replacement proposal schema；
-不允许直接调用 mutation tool；
-最大轮数较低；
-固定低 temperature；
-可使用更强模型，但用户可配置。

#### M6-03：Proposal 类型

```text
create
merge
replace
archive
ignore
quarantine
request_user_clarification
```

每个 proposal 必须包含：

- target/canonical key；
- proposed content；
- structured value；
- evidence IDs；
- confidence；
- novelty；
- risk；
- reason；
- expected version；
- before preview。

#### M6-04：Deterministic Validator

拒绝：

- 无 evidence；
- evidence 不支持 scope；
-引用不存在；
-用户明确偏好被低信任推断覆盖；
-敏感内容；
-injection；
-高 confidence 但证据不足；
-canonical conflict；
-capacity overflow；
-修改 Soul/Prompt；
-可执行指令。

#### M6-05：Memory Mutation Transaction

批准 batch：

```text
验证 expected version
插入新 memory/version
写 evidence links
supersede old active
写 mutation audit
更新 Dream candidate
更新 event consolidation_state
commit
```

任一步失败全部 rollback。

#### M6-06：Dream Review UI

- batch summary；
-before/after；
-evidence；
-accept/edit/reject；
-批量低风险；
-用户选择“以后自动合并完全重复项”。

#### M6-07：Weekly Dream

在 daily 稳定后加入：

-跨日模式；
-过时记忆；
-长期目标；
-教学策略候选；
-容量压缩。

### 测试

- no-evidence rejection；
- explicit preference protected；
- concurrent edit version conflict；
- mutation rollback；
- replacement provenance；
- capacity；
- injection；
- rerun same window；
- user rejection not resurfaced without new evidence。

### DoD

- Dream 可生成可解释 proposal；
- active Memory 只有批准后变化；
-所有变化有 before/after/source；
-用户更正不可被覆盖。

### Rollback

- `dream_proposal_v1=false`；
-所有 proposal 保留 pending/rejected；
- active memory 不依赖 Dream。

---

## M7：低风险自动整理、FTS5 与 Memory 安全加固

### 目标

在 proposal mode 经验证后，只允许高度确定、可逆、低风险的自动操作，并提升本地搜索。

### Migration

`0018_memory_fts.sql` 可提前到此阶段：

- `memory_fts`；
- `coach_messages_fts`；
- `agent_messages_fts`；
- trigger 或 rebuild 逻辑。

### 允许自动操作

```text
完全重复合并
已 superseded 且超 retention 的归档
同 canonical key 的表述压缩，且事实/结构不变
低重要度 candidate 拒绝
过期 daily candidate 清理
```

### 不允许自动操作

```text
改变用户明确偏好
高层能力判断
负面画像
新的教学策略
删除用户固定 Memory
修改目标、考试日期
任何 Prompt/Soul 修改
```

### 工作项

#### M7-01：Auto policy engine

```rust
pub fn auto_promotion_decision(
    candidate: &ValidatedDreamCandidate,
    policy: &MemoryAutoPolicy,
) -> AutoDecision
```

不由 LLM 自己决定是否有权限。

#### M7-02：FTS5

- Memory content/title/structured text；
- Coach messages；
- Agent messages；
- rebuild command；
- integrity check；
- BM25 + metadata filter。

#### M7-03：Security quarantine

- scanner versions；
- quarantine UI；
-用户可删除但不直接激活；
-开发诊断报告。

#### M7-04：Confidence decay job

按 memory type：

- preference decay 慢；
- current weakness decay 快；
- goal 有 valid_to；
- procedural strategy 依赖 outcome refresh。

Decay 不删除 memory，只影响 retrieval/status。

#### M7-05：Capacity compaction

每个 scope 有：

```text
max active count
max rendered tokens
max candidate age
```

达到容量时：

- merge/replace proposal；
-低重要度 archive；
-不得简单删除 oldest。

### 测试

- FTS ranking；
- rebuild；
- trigger consistency；
- auto operation allowlist；
- forbidden auto mutation；
- quarantine never retrieved；
- confidence decay deterministic。

### DoD

- Memory 不无限增长；
- FTS 搜索可用；
-低风险自动整理可审计和关闭；
-投毒候选不进入 Context。

### Rollback

- 关闭 auto；
- FTS 可重建；
- canonical memory 不受索引损坏影响。

---

## M8：Learner Model v1 与重复练习分析

### 目标

把学习记录从“历史列表”提升为可解释、带 uncertainty 的学习者状态。

### Migration

`0016_learner_model.sql`

包含：

- `skill_catalog`；
- `question_skill_map`；
- `learner_skill_observations`；
- `learner_skill_state`；
- intervention/outcome link 可同 migration 或后续扩展。

### 工作项

#### M8-01：Skill taxonomy v1

Reading 初版建议：

```text
reading.comprehension.main_idea
reading.comprehension.detail
reading.inference
reading.reference
reading.vocabulary_context
reading.tfng.statement_scope
reading.tfng.not_given_boundary
reading.matching_headings.main_idea
reading.matching_headings.scope
reading.matching_information.localization
reading.summary_completion.grammar
reading.summary_completion.paraphrase
reading.multiple_choice.distractor
reading.time_management
```

Writing：

```text
writing.task_response.position
writing.task_response.development
writing.coherence.paragraphing
writing.coherence.progression
writing.lexical.precision
writing.lexical.collocation
writing.grammar.range
writing.grammar.accuracy
```

Taxonomy 要版本化，避免后续重命名破坏历史。

#### M8-02：Question mapping

来源优先级：

1. 内容包明确 mapping；
2.确定性题型 mapping；
3.人工标注；
4.LLM candidate + 人工/规则验证。

不允许运行时模型随意给题目贴永久 skill。

#### M8-03：Observation builder

从 event 生成：

- outcome；
- error type；
- evidence weight；
-novelty weight；
-time weight；
- same-item repeat；
- distinct asset。

#### M8-04：State updater

实现 Beta/EWMA baseline；

-版本化 model；
-可重建；
-解释；
- uncertainty；
- next review suggestion。

#### M8-05：Repeated Attempt Analyzer

输出 deterministic：

```text
score trajectory
question transitions
error type transitions
first-attempt vs repeated performance
same item familiarity warning
inter-attempt gap
answer change behavior
cross-item transfer evidence
```

#### M8-06：Learner UI

- skill state table；
-timeline；
- evidence；
-状态 label；
-不展示伪精确 Band 预测。

#### M8-07：Agent tools

```text
get_skill_state
get_skill_timeline
compare_attempts_for_asset
find_transfer_evidence
suggest_diagnostic_practice (proposal-only)
```

### 测试

- observation idempotency；
- same-item downweight；
- distinct asset；
- uncertainty；
- taxonomy migration；
- state rebuild；
- deletion；
- no label overclaim。

### DoD

- 用户可看到跨时间成长；
-同题熟悉度不冒充能力；
- Agent 使用 learner state 时标 uncertainty；
-模型可完全从 events 重建。

### Rollback

- learner tables 是 derived；
-可清除重建；
-练习判分不依赖 learner state。

---

## M9：Personalized Coach v1

### 目标

将 Context、Memory 和 Learner Model 接入阅读 Coach 与复盘主流程。

### 工作项

#### M9-01：场景化 endpoint

```text
coach_explain_question
coach_review_attempt
coach_compare_attempts
coach_submit_feedback
```

保留通用 `coach_run` 作为高级入口，但主 UI 不再构造自由 `questionContext` 大对象。

#### M9-02：Coach Context Profile

任务配置：

```text
current evidence 45%
learner state 12%
explicit preferences 8%
procedural memory 10%
episodic memory 8%
recent thread 17%
```

#### M9-03：Strategy selector

初期确定性：

- skill + error taxonomy；
- explicit preference；
- active procedural memory；
- no confident strategy fallback。

#### M9-04：Structured output

完善现有严格 parsing：

- evidence spans；
- diagnosis；
-next step；
-self-check；
-uncertainty；
-strategy ID；
-context snapshot ID。

#### M9-05：Feedback

快速类型 + 文本；

- immediate event；
-memory candidate；
-not active update。

#### M9-06：Intervention linkage

创建：

```text
coach_interventions
coach_intervention_targets
coach_intervention_outcomes
```

如不增加表，可先用 learning_events structured payload，但中期应正规化。

#### M9-07：Why panel

阅读结果页显示个性化来源。

#### M9-08：Outcome job

当后续相关 observation 出现时：

- link candidate；
-生成弱 attribution；
-更新 strategy evidence；
-不直接提升全局 Prompt。

### 测试

- current evidence correctness；
- preferences relevant only；
- unsupported learner state；
-deleted memory；
- answer leakage；
- feedback candidate；
- subsequent outcome linkage；
- provider failure fallback。

### DoD

- Coach 对同一用户跨会话有连续性；
-个性化可解释；
-不满意反馈进入候选；
-后续学习结果可验证策略。

### Rollback

- `personalized_coach_v1=false` 使用现有 Coach；
-所有新增数据 derived。

---

## M10：Procedural Memory 与教学策略学习

### 目标

从“记得用户喜欢什么”升级为“记得对这个用户什么教学流程更有效”。

### 工作项

#### M10-01：Teaching Strategy Schema

正式建立：

- scope；
-trigger；
-sequence；
-constraints；
-evidence；
-outcome；
-confidence；
-status/version。

可存入 `memory_items(memory_type='procedural')`，不必单独表；但 structured schema 必须版本化。

#### M10-02：Strategy Candidate Extractor

来源：

-多次明确反馈；
-相同策略跨不同题目成功；
-用户手动保存；
-Coach critique。

#### M10-03：Eligibility

建议初始门槛：

```text
≥ 2 个不同 session
≥ 2 个不同 asset 或 1 个明确用户确认
无事实/安全负反馈
至少 1 个后续 outcome 或用户固定
```

#### M10-04：Strategy selection experiment

同一用户可做低风险 alternating：

- 用户 opt-in；
-不改变答案；
-记录策略版本；
-以 outcome 和 feedback 比较。

不必做正式 A/B 平台，先做离线和单用户可解释比较。

#### M10-05：Dream 周整合

weekly Dream 可：

-合并策略；
-降低失效策略 confidence；
-提出新策略；
-列出证据。

#### M10-06：UI

Memory Center “有效方法”页：

-适用场景；
-方法步骤；
-证据；
-效果；
-禁用/固定/编辑。

### 测试

-策略不越过 safety；
-用户禁用；
-outcome attribution；
-冲突策略；
-低证据不激活；
-过时衰减。

### DoD

- 至少 Reading 两类 skill 支持个体策略；
- 策略有来源和 outcome；
- 用户可治理；
-全局 Prompt 未被在线修改。

### Rollback

-停止检索 procedural memory；
-保留数据；
-Coach 回退通用策略。

---

## M11：Prompt Registry、Eval Harness 与影子自进化

### 目标

建立产品级 Prompt/Skill 改进管线，但不自动发布。

### Migration

`0017_prompt_evolution_evals.sql`

包含：

- `prompt_artifacts`；
- `prompt_versions`；
- `eval_suites`；
- `eval_cases`；
- `eval_runs`；
- `eval_results`；
- shadow/canary metadata。

### 工作项

#### M11-01：Prompt registry

把硬编码 system prompt 迁移为版本化 seed：

- workspace Agent；
-Reading Coach；
-Writing Evaluation；
-Memory Extractor；
-Dream Consolidator。

运行时仍有编译期 fallback，避免 DB seed 失败导致不可用。

#### M11-02：Prompt Renderer

-模块组合；
-版本；
-content hash；
-token estimate；
-frozen snapshot；
-no mid-run hot replacement。

#### M11-03：Eval case format

```json
{
  "id":"reading-review-001",
  "taskKind":"attempt_review",
  "inputFixture": {...},
  "mustUseTools":["get_attempt_detail"],
  "mustNotUseTools":[],
  "deterministicExpectations": {...},
  "graderRubric": {...},
  "split":"validation"
}
```

#### M11-04：Replay harness

- fake model contract tests；
-real model optional；
-repeated trials；
-trace capture；
-cost/latency。

#### M11-05：Graders

- deterministic；
-domain；
-LLM；
-human review export。

#### M11-06：Manual candidates

第一阶段人工编辑候选；

- baseline compare；
-holdout；
-red-team；
-shadow。

#### M11-07：Shadow runner

生产采样：

-使用 frozen snapshot；
-read-only；
-不展示；
-不写 Memory；
-用户 opt-in；
-保存结果评分。

#### M11-08：Promotion/rollback

- hard gates；
-人工批准；
-active unique；
-rollback command；
-release report。

### 测试

- active uniqueness；
-prompt hash；
-split leakage；
-shadow no side effects；
-baseline replay；
-rollback；
-eval reproducibility。

### DoD

- Prompt 版本可追踪；
-候选能和 baseline 比较；
-shadow 无副作用；
-没有在线自改 Prompt。

### Rollback

- registry active 回到 previous version；
-编译期 fallback；
-eval tables 不影响产品功能。

---

## M12：自动候选生成、Study Planner 与受控 Action Tools

### 目标

在所有前置治理成熟后，才加入更强“自进化”和 Agent 行动能力。

### M12-A：自动 Prompt/Skill 候选生成

#### 工作项

- failure clustering；
- trace critique；
-GEPA/DSPy adapter；
-候选 constraints；
-validation/holdout；
-human PR/export；
-no auto promotion。

#### DoD

-自动生成器只是 plugin；
-关闭后 eval/promotion 正常；
-每个候选有 parent 和 reasoning；
-不会读取 holdout。

### M12-B：Study Planner

#### 工具

```text
get_learning_goal
get_skill_states
get_due_vocabulary
get_recent_workload
get_available_assets
propose_study_plan
```

#### 规则

- 先 proposal；
-用户批准；
-有日期和目标；
-避免重复同题；
-包含 retrieval + spacing；
-有 workload cap；
-计划调整有审计。

### M12-C：Action Tools

可逐个增加：

```text
create_practice_queue
update_study_plan
add_vocab_item
schedule_review
create_note
```

每个工具必须：

- approval；
-idempotency；
-expected version；
-undo；
-audit；
-E2E。

### M12-D：Embeddings（可选）

启用条件：

- FTS/structured retrieval 明确不足；
-有 retrieval goldens；
-本地 embedding 成本可接受；
-模型版本和 reindex 方案；
-不把 vector DB 作为 canonical；
-用户隐私策略。

可使用 SQLite vector extension 或 app-managed embedding table；不建议一开始引入独立服务器。

### M12-E：长期实验

- BKT/DKT；
-temporal graph；
-multi-agent evaluator；
-local model routing；
-private cloud sync。

这些均不进入近期主线。

---

## 21.3 阶段依赖关系

```text
M0 Baseline
 └─ M1 Event Ledger
     ├─ M2 Threads/Trace
     │   └─ M4 Context Compiler
     ├─ M3 Manual Memory
     │   └─ M4 Context Compiler
     └─ M5 Journal
         └─ M6 Dream Proposal
             └─ M7 Auto Low-risk + FTS

M1 Event Ledger
 └─ M8 Learner Model
     └─ M9 Personalized Coach
         └─ M10 Procedural Strategy

M2 + M4 + M9
 └─ M11 Prompt Registry/Eval
     └─ M12 Automatic Candidate Generation

M8 + M10
 └─ M12 Study Planner/Action Tools
```

## 21.4 推荐发布里程碑

| 产品版本 | 包含能力 |
|---|---|
| Alpha A | M0–M2：真实 Agent 工作台、read tools、thread/trace |
| Alpha B | M3–M5：手动 Memory、Context v1、Daily Journal |
| Beta A | M6–M8：Dream proposal、FTS、Learner Model |
| Beta B | M9–M10：个性化 Coach、教学策略 |
| RC | M11：Prompt registry、eval、shadow |
| 1.x 实验 | M12：自动候选、Study Planner、Action Tools |

## 21.5 每阶段强制交付物

```text
1. ADR / design note
2. migration + schema docs
3. Rust domain/application interfaces
4. Tauri commands/events
5. Vue minimal UI
6. unit/integration/E2E
7. backup/restore changes
8. privacy/retention behavior
9. metrics and diagnostics
10. rollback instructions
11. current limitations
12. final gate report
```

---

# 22. 建议目录结构

## 22.1 Rust Workspace

```text
Cargo.toml
crates/
  ielts-domain/
    src/
      domain.rs
      dto/
      error.rs
      learner/
        skill.rs
        observation.rs
      memory/
        types.rs
        value.rs
      agent/
        types.rs

  ielts-db/
    migrations/
      0001_...
      0011_agent_runs_tool_calls.sql
      0012_learning_event_ledger.sql
      0013_agent_threads_checkpoints.sql
      0014_memory_profile_core.sql
      0015_journal_dream_jobs.sql
      0016_learner_model.sql
      0017_prompt_evolution_evals.sql
      0018_memory_fts.sql
    src/
      learning_events/
        mod.rs
        queries.rs
        rebuild.rs
      memory/
        mod.rs
        repository.rs
        mutations.rs
        search.rs
      journal/
        mod.rs
      dream/
        mod.rs
      learner/
        observations.rs
        state.rs
        mapping.rs
      agent/
        runs.rs
        threads.rs
        checkpoints.rs
        context.rs
      prompts/
        registry.rs
        eval.rs
      sqlite/
      reading/
      writing/
      history/
      coach/
      vocab/
      backup/

  ielts-application/
    src/
      lib.rs
      ports.rs
      error.rs
      agent/
        mod.rs
        service.rs
        policy.rs
        checkpoint.rs
      context/
        mod.rs
        compiler.rs
        query_plan.rs
        ranking.rs
        budget.rs
        renderer.rs
      memory/
        mod.rs
        service.rs
        validator.rs
        mutation.rs
        profile.rs
      journal/
        service.rs
      dream/
        service.rs
        contracts.rs
        validator.rs
      learner/
        service.rs
        repeated_attempts.rs
        review_scheduler.rs
      coach/
        service.rs
        response.rs
        strategy.rs
      prompts/
        renderer.rs
      evolution/
        contracts.rs
      jobs/
        worker.rs
        handlers.rs

src-tauri/
  src/
    ai/
      config.rs
      runtime.rs
      invocation_store.rs
      providers/
        openai_compatible.rs
    agent/
      workspace.rs
      file_tools.rs
      registry.rs
      learning_tools.rs
      memory_tools.rs
      approval.rs
      run_control.rs
    app/
      state.rs
      application_store.rs
      worker.rs
    commands/
      agent.rs
      memory.rs
      journal.rs
      dream.rs
      learner.rs
      prompts.rs
      ...
    lib.rs
```

## 22.2 Vue

```text
apps/writing-vue/src/
  api/
    agent-repository.ts
    memory-repository.ts
    journal-repository.ts
    learner-repository.ts
    prompt-eval-repository.ts
    tauri-bridge.js

  modules/
    agent/
      composables/
        useAgentThread.ts
        useAgentRun.ts
        useToolApproval.ts
        useContextSnapshot.ts
      components/
        AgentConversation.vue
        AgentRunTrace.vue
        ToolCallCard.vue
        ContextPackPanel.vue
        ApprovalCard.vue

    memory/
      components/
        MemoryCard.vue
        MemoryEvidenceDrawer.vue
        MemoryMutationDiff.vue
        AutoLearningSettings.vue
      views/
        MemoryCenterPage.vue

    journal/
      components/
        DailyJournalCard.vue
        DreamReport.vue
        DreamProposalCard.vue
      views/
        JournalPage.vue

    learner/
      components/
        SkillStateTable.vue
        SkillTimeline.vue
        AttemptComparison.vue
        EvidenceDiversityBadge.vue
      views/
        LearnerProfilePage.vue

    coach/
      composables/
        usePersonalizedCoach.ts
      components/
        CoachResponse.vue
        CoachFeedback.vue
        WhyThisAnswer.vue

  views/
    AgentWorkspacePage.vue
```

## 22.3 Developer Evolution

```text
developer/evolution/
  README.md
  pyproject.toml
  cli.py
  adapters/
    rust_runtime.py
    gepa_adapter.py
    dspy_adapter.py
  datasets/
    builder.py
    sanitizer.py
    splitter.py
  evaluators/
    deterministic.py
    domain.py
    llm.py
    safety.py
  candidate_generators/
    manual.py
    critique_rewrite.py
    gepa.py
  reports/
  configs/
    reading_coach.yaml
    dream.yaml
    memory_extractor.yaml
```

此目录不是 shipping runtime 依赖。发布产物只包含已批准的 Prompt/Skill artifacts。

## 22.4 Prompt / Skill Assets

```text
assets/agent/
  soul/
    core.md
    learning_policy.md
    privacy.md
  prompts/
    reading_coach/
      manifest.yaml
      system.md
      response.schema.json
    dream_consolidator/
      manifest.yaml
      system.md
      output.schema.json
    memory_extractor/
      manifest.yaml
      system.md
      output.schema.json
  skills/
    reading-attempt-review/
      skill.yaml
      prompt.md
      tools.json
      response.schema.json
```

启动时 seed 到 Prompt Registry；运行时以 active version 为准。

---

# 23. 关键伪代码

## 23.1 SQLite Job Claim

SQLite 单 worker 仍应使用原子 claim，防止应用快速重入或未来多窗口重复执行：

```rust
fn claim_next_job(
    conn: &Connection,
    worker_id: &str,
    now: DateTime<Utc>,
) -> DbResult<Option<BackgroundJob>> {
    let tx = conn.unchecked_transaction()?;

    let job_id: Option<String> = tx.query_row(
        r#"
        SELECT id
        FROM background_jobs
        WHERE status = 'queued'
          AND scheduled_at <= ?1
          AND attempts < max_attempts
        ORDER BY priority DESC, scheduled_at ASC, created_at ASC
        LIMIT 1
        "#,
        [now.to_rfc3339()],
        |row| row.get(0),
    ).optional()?;

    let Some(job_id) = job_id else {
        tx.commit()?;
        return Ok(None);
    };

    let updated = tx.execute(
        r#"
        UPDATE background_jobs
        SET status = 'running',
            attempts = attempts + 1,
            locked_at = ?1,
            heartbeat_at = ?1,
            locked_by = ?2,
            updated_at = ?1
        WHERE id = ?3 AND status = 'queued'
        "#,
        params![now.to_rfc3339(), worker_id, job_id],
    )?;

    if updated != 1 {
        tx.rollback()?;
        return Ok(None);
    }

    let job = load_job(&tx, &job_id)?;
    tx.commit()?;
    Ok(Some(job))
}
```

## 23.2 Stale Job Recovery

```rust
fn recover_stale_jobs(
    conn: &Connection,
    stale_before: DateTime<Utc>,
) -> DbResult<u32> {
    let count = conn.execute(
        r#"
        UPDATE background_jobs
        SET status = CASE
              WHEN attempts < max_attempts THEN 'queued'
              ELSE 'failed'
            END,
            locked_at = NULL,
            locked_by = NULL,
            heartbeat_at = NULL,
            error_json = json_object(
              'code', 'job.interrupted',
              'message', 'job interrupted by application shutdown',
              'retryable', attempts < max_attempts
            ),
            updated_at = ?1
        WHERE status = 'running'
          AND COALESCE(heartbeat_at, locked_at) < ?2
        "#,
        params![Utc::now().to_rfc3339(), stale_before.to_rfc3339()],
    )?;
    Ok(count as u32)
}
```

## 23.3 Learning Event Idempotency

```rust
fn event_key(
    event_type: &str,
    source_id: &str,
    schema_version: u32,
) -> String {
    sha256(format!("{event_type}|{source_id}|v{schema_version}"))
}

fn append_event(
    tx: &Transaction,
    event: NewLearningEvent,
) -> DbResult<LearningEvent> {
    let idempotency_key = event_key(
        event.event_type.as_str(),
        &event.source_id,
        event.schema_version,
    );

    tx.execute(
        r#"
        INSERT INTO learning_events (..., idempotency_key, ...)
        VALUES (..., ?1, ...)
        ON CONFLICT(idempotency_key) DO NOTHING
        "#,
        params![idempotency_key, ...],
    )?;

    load_event_by_key(tx, &idempotency_key)
}
```

## 23.4 重复 Attempt 比较

```rust
fn compare_attempts(
    attempts: &[AttemptEvidence],
) -> AttemptComparisonReport {
    let ordered = sort_by_started_at(attempts);
    let mut transitions = Vec::new();

    for pair in ordered.windows(2) {
        let previous = &pair[0];
        let current = &pair[1];
        let gap = current.started_at - previous.started_at;

        let question_ids = union_question_ids(previous, current);
        for qid in question_ids {
            let before = previous.answer(qid);
            let after = current.answer(qid);
            transitions.push(QuestionTransition {
                question_id: qid.to_string(),
                before_correct: before.and_then(|x| x.is_correct),
                after_correct: after.and_then(|x| x.is_correct),
                transition: classify_transition(before, after),
                gap_hours: gap.num_hours(),
                same_asset: true,
                familiarity_weight: repeat_familiarity_weight(
                    current.ordinal_for_asset,
                    gap,
                ),
            });
        }
    }

    AttemptComparisonReport {
        attempts: summarize_attempts(&ordered),
        transitions,
        transfer_warning: true,
        conclusions: deterministic_conclusions(&ordered),
    }
}
```

## 23.5 Observation Weight

```rust
fn observation_weight(input: ObservationInput) -> ObservationWeights {
    let evidence_weight = match input.source {
        ObservationSource::SubmittedAnswer => 1.0,
        ObservationSource::WritingCriterion => 0.8,
        ObservationSource::CoachSelfReport => 0.3,
        ObservationSource::UserExplicitAssessment => 0.5,
    };

    let repeat_penalty = 1.0 / (1.0 + 0.45 * input.asset_repeat_index as f32);
    let gap_bonus = sigmoid((input.gap_hours as f32 - 24.0) / 24.0);
    let novelty_weight = (repeat_penalty * (0.6 + 0.4 * gap_bonus))
        .clamp(0.15, 1.0);

    let time_weight = exponential_decay(
        input.observed_at,
        input.now,
        input.skill_half_life_days,
    );

    ObservationWeights {
        evidence_weight,
        novelty_weight,
        time_weight,
    }
}
```

## 23.6 Memory Candidate Extraction

```rust
async fn extract_memory_candidates(
    events: &[LearningEvent],
    explicit_feedback: &[CoachFeedbackEvent],
    model: &dyn ChatModel,
) -> Result<Vec<MemoryMutationProposal>, ApplicationError> {
    let deterministic = extract_explicit_preferences(explicit_feedback);
    let evidence_bundle = build_redacted_evidence_bundle(events, explicit_feedback);

    if evidence_bundle.is_empty() {
        return Ok(deterministic);
    }

    let llm_output: MemoryCandidateOutput = call_structured(
        model,
        prompt_registry.active("memory_extractor")?,
        evidence_bundle,
    ).await?;

    let mut proposals = deterministic;
    for proposal in llm_output.proposals {
        proposals.push(validate_candidate_shape(proposal)?);
    }
    Ok(proposals)
}
```

关键：这里返回 proposal，不执行 mutation。

## 23.7 Memory Mutation Transaction

```rust
fn apply_memory_mutations(
    conn: &Connection,
    batch: ApprovedMemoryMutationBatch,
) -> DbResult<MemoryMutationResult> {
    let tx = conn.unchecked_transaction()?;
    let mut applied = Vec::new();

    for proposal in batch.proposals {
        validate_evidence_exists(&tx, &proposal.evidence_ids)?;
        validate_security(&proposal)?;
        validate_capacity(&tx, &proposal)?;

        match proposal.operation {
            Create => {
                assert_no_active_canonical_conflict(&tx, &proposal)?;
                let memory = insert_memory(&tx, &proposal)?;
                link_evidence(&tx, &memory.id, &proposal.evidence_ids)?;
                write_mutation_log(&tx, None, &memory, &proposal)?;
                applied.push(memory);
            }
            Replace | Merge => {
                let current = load_memory_for_update(&tx, proposal.target_id()?)?;
                assert_eq!(current.version, proposal.expected_version);
                let replacement = insert_replacement(&tx, &current, &proposal)?;
                mark_superseded(&tx, &current.id, &replacement.id)?;
                link_evidence(&tx, &replacement.id, &proposal.evidence_ids)?;
                write_mutation_log(&tx, Some(&current), &replacement, &proposal)?;
                applied.push(replacement);
            }
            Archive => {
                let current = load_memory_for_update(&tx, proposal.target_id()?)?;
                assert_eq!(current.version, proposal.expected_version);
                archive_memory(&tx, &current.id)?;
                write_archive_log(&tx, &current, &proposal)?;
            }
            _ => return Err(DbError::Validation("unsupported approved operation".into())),
        }
    }

    mark_dream_candidates_applied(&tx, &batch)?;
    mark_events_processed(&tx, &batch.evidence_event_ids)?;
    tx.commit()?;

    Ok(MemoryMutationResult { applied })
}
```

## 23.8 Dream Consolidation

```rust
async fn run_dream(
    service: &DreamService<'_>,
    cmd: RunDreamCommand,
) -> Result<DreamRunResult, ApplicationError> {
    let run = service.store.begin_run(&cmd)?;

    let events = service.learning.load_unprocessed_window(
        cmd.coverage_start,
        cmd.coverage_end,
    )?;
    let active = service.memory.load_relevant_active_region(&events)?;
    let corrections = service.memory.load_recent_user_corrections(&events)?;

    if !minimum_evidence_met(&events, &active) {
        return service.store.finish_insufficient(run.id, events.len());
    }

    let evidence_bundle = build_read_only_dream_bundle(
        &events,
        &active,
        &corrections,
        service.policy,
    )?;

    let output: DreamOutput = service.model.consolidate(evidence_bundle).await?;
    let validated = service.validator.validate(
        &output,
        &events,
        &active,
        &corrections,
    )?;

    let decisions = validated
        .into_iter()
        .map(|proposal| service.auto_policy.classify(proposal))
        .collect::<Vec<_>>();

    service.store.persist_proposals(&run.id, &decisions)?;
    let auto = decisions.iter().filter_map(auto_approved).collect();
    if !auto.is_empty() {
        service.memory.apply_mutation_batch(auto)?;
    }

    service.store.finish_completed(run.id)
}
```

## 23.9 Context Ranking 与 Budget Packing

```rust
fn pack_context(
    mut items: Vec<ContextCandidate>,
    budget: &ContextBudget,
    estimator: &dyn TokenEstimator,
) -> PackedContext {
    for item in &mut items {
        item.score = score_context_item(item);
        item.tokens = estimator.estimate(&item.rendered);
    }

    let items = dedupe_and_resolve_conflicts(items);
    let sections = group_by_section(items);
    let mut packed = PackedContext::new();

    for section in SECTION_PRIORITY {
        let section_budget = budget.for_section(section);
        let selected = greedy_mmr_pack(
            sections.get(section),
            section_budget,
            |candidate, selected| {
                candidate.score - redundancy(candidate, selected)
            },
        );
        packed.add(section, selected);
    }

    packed.ensure_required_current_evidence();
    packed.ensure_total_budget(budget.total_tokens);
    packed
}
```

## 23.10 Agent Loop with Context、Approval 与 Cancellation

```rust
async fn run_agent(
    deps: &AgentDependencies<'_>,
    command: RunAgentCommand,
) -> Result<AgentRunOutcome, ApplicationError> {
    let run = deps.runs.begin(&command)?;
    let context = deps.context.build(command.context_request()).await?;
    deps.runs.attach_context(&run.id, &context.snapshot_id)?;

    let mut state = AgentLoopState::from_thread_and_context(&command, &context)?;

    for round in 1..=command.limits.max_rounds {
        command.cancellation.check()?;
        deps.checkpoints.save(&run.id, round, BeforeModel, &state)?;

        let response = deps.model.respond(
            state.to_model_request(deps.tools.definitions(&command.run_kind)),
            command.model_metadata(&context),
        ).await?;

        deps.invocations.record_response(&response)?;
        state.append_assistant(response.clone());

        if response.tool_calls.is_empty() {
            let final_response = validate_final_response(response.content, &command)?;
            deps.threads.append_assistant(&command.thread_id, &final_response)?;
            deps.runs.complete(&run.id, round, state.tool_count, &final_response)?;
            return Ok(final_response.into());
        }

        for call in response.tool_calls {
            command.cancellation.check()?;
            let tool = deps.tools.resolve(&call.name, &command.run_kind)?;
            let validated = tool.validate(command.tool_context(), call.arguments()).await?;

            if tool.policy().requires_approval(&validated) {
                deps.approvals.create(&run.id, &call, &validated)?;
                deps.runs.waiting_approval(&run.id, &call.id)?;
                deps.checkpoints.save(&run.id, round, WaitingApproval, &state)?;
                return Err(ApplicationError::approval_required(call.id));
            }

            deps.checkpoints.save(&run.id, round, BeforeTool, &state)?;
            let output = tool.execute(command.tool_context(), validated).await;
            deps.runs.record_tool_result(&run.id, &call, &output)?;
            state.append_tool_result(call, output.model_payload);
            deps.checkpoints.save(&run.id, round, AfterTool, &state)?;
        }
    }

    deps.runs.limit_exceeded(&run.id)?;
    Err(ApplicationError::limit_exceeded())
}
```

## 23.11 Coach Strategy Selection

```rust
fn select_teaching_strategy(
    task: &CoachTask,
    explicit: &ExplicitProfile,
    learner: &[LearnerSkillState],
    procedural: &[TeachingStrategy],
) -> StrategySelection {
    let candidates = procedural
        .iter()
        .filter(|s| s.status == Active)
        .filter(|s| s.scope.matches(task))
        .filter(|s| s.trigger.matches(task, learner))
        .filter(|s| !explicit.disabled_strategy_ids.contains(&s.id))
        .collect::<Vec<_>>();

    let best = candidates.into_iter().max_by(|a, b| {
        strategy_score(a, task, explicit, learner)
            .total_cmp(&strategy_score(b, task, explicit, learner))
    });

    match best {
        Some(strategy) if strategy.confidence >= 0.65 => StrategySelection::Personalized(strategy.clone()),
        _ => StrategySelection::Default(default_strategy(task)),
    }
}
```

## 23.12 Coach Feedback → Candidate

```rust
fn feedback_to_candidate(
    feedback: CoachFeedback,
    interaction: CoachInteraction,
) -> Option<MemoryMutationProposal> {
    match feedback.kind {
        TooAbstract if feedback.user_text.is_some() => Some(
            MemoryMutationProposal::candidate_preference(
                canonical_key = "profile.teaching.example_first",
                value = true,
                evidence = vec![interaction.feedback_event_id],
                confidence = 0.55,
                risk = Medium,
            )
        ),
        FactualError => None, // 进入质量问题，不形成用户偏好
        AnswerTooEarly => Some(
            MemoryMutationProposal::candidate_preference(
                canonical_key = "profile.teaching.delay_answer_reveal",
                value = true,
                evidence = vec![interaction.feedback_event_id],
                confidence = 0.65,
                risk = Low,
            )
        ),
        _ => None,
    }
}
```

## 23.13 Intervention Outcome Link

```rust
fn link_outcomes(
    intervention: &CoachIntervention,
    new_observations: &[SkillObservation],
) -> Vec<InterventionOutcome> {
    new_observations
        .iter()
        .filter(|obs| intervention.skill_keys.contains(&obs.skill_key))
        .filter(|obs| obs.observed_at > intervention.created_at)
        .filter(|obs| obs.observed_at <= intervention.outcome_window_end)
        .map(|obs| InterventionOutcome {
            intervention_id: intervention.id.clone(),
            observation_id: obs.id.clone(),
            attribution_weight: attribution_weight(intervention, obs),
            is_transfer: obs.asset_id.as_deref() != intervention.asset_id.as_deref(),
        })
        .collect()
}
```

## 23.14 Prompt Promotion Transaction

```rust
fn promote_prompt_version(
    conn: &Connection,
    candidate_id: &str,
    approval: PromptPromotionApproval,
) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    let candidate = load_prompt_version(&tx, candidate_id)?;

    require_status(&candidate, PromptStatus::Canary)?;
    require_eval_report_passed(&tx, candidate.evaluation_run_id.as_deref())?;
    require_human_approval(&approval)?;

    tx.execute(
        "UPDATE prompt_versions SET status='retired' \
         WHERE artifact_id=?1 AND status='active'",
        [candidate.artifact_id.clone()],
    )?;

    let changed = tx.execute(
        "UPDATE prompt_versions SET status='active', activated_at=?1 \
         WHERE id=?2 AND status='canary'",
        params![Utc::now().to_rfc3339(), candidate.id],
    )?;

    if changed != 1 {
        return Err(DbError::Validation("candidate version changed concurrently".into()));
    }

    insert_prompt_audit(&tx, &candidate, &approval)?;
    tx.commit()?;
    Ok(())
}
```

## 23.15 Context Snapshot Replay

```rust
async fn replay_run(
    run_id: &str,
    prompt_version_id: &str,
    model: &dyn ToolCallingModel,
) -> Result<ReplayResult, ApplicationError> {
    let original = stores.runs.load(run_id)?;
    let snapshot = stores.context.load(original.context_snapshot_id?)?;
    let frozen = snapshot.render_exact()?;
    let tools = tools.read_only_shadow_registry(original.run_kind);

    let result = execute_shadow(
        model,
        prompt_version_id,
        frozen,
        tools,
        original.user_message,
    ).await?;

    Ok(compare_trace(original, result))
}
```

---

# 24. 风险清单与反模式

## 24.1 最高风险：把派生画像当成用户真相

### 症状

- Agent 说“你就是不擅长……”。
- 一次错题形成永久弱项。
- 一次对话偏好覆盖用户明确设置。

### 后果

- 错误个性化被长期放大；
- 用户被标签化；
- 后续检索不断强化原判断。

### 控制

- canonical/derived 分层；
- confidence + uncertainty；
- diverse evidence；
- user correction priority；
- supersession；
-why panel。

## 24.2 最高风险：在线 Agent 修改自己的 Soul/Prompt

### 症状

- `write_prompt` 工具；
- Dream 直接更新 system prompt；
- 用户一次反馈立即改变全局行为。

### 后果

- 不可预测；
- 安全规则被腐蚀；
- reward hacking；
- 无法回放和评估。

### 控制

- 生产 Agent 无写权限；
- Prompt registry；
- offline eval；
- holdout；
-人工 promotion；
- rollback。

## 24.3 最高风险：Memory Poisoning

### 症状

- 题目/网页/文件包含“永久记住”；
- Agent 把外部文本保存为规则；
- 恶意记忆在未来会话触发工具。

### 控制

- trust labels；
- ingestion firewall；
- quarantine；
- memory 不能保存操作指令；
- tool policy 不依赖 memory 文本；
- red-team corpus。

## 24.4 大爆炸重构

### 症状

- 同时迁移 schema、application、Prompt、UI；
- 所有 command 改名；
-删除旧路径后再补测试。

### 控制

- M0–M12 纵向切片；
- feature flag；
- characterization tests；
- wrapper first；
-每阶段 rollback。

## 24.5 Agent Workspace 变成产品中心

### 症状

- 所有智能功能必须进入聊天页；
- 主练习页面只负责记录；
- Agent 缺少当前页面上下文。

### 控制

- 场景化 endpoint；
- embedded Coach；
- Agent Workspace 定位为高级入口/trace UI；
- context surface。

## 24.6 把所有数据塞进 Prompt

### 症状

- 全历史；
-完整 Coach transcript；
-所有 Memory；
-所有技能状态。

### 后果

- context rot；
-成本；
-隐私；
-答案偏离当前题目。

### 控制

- Context Compiler；
- token budget；
- just-in-time tools；
-FTS/hybrid；
-compaction。

## 24.7 把向量数据库当作记忆系统

### 症状

- 任何对话 chunk 都 embedding；
-相似度高就注入；
-无状态、无来源、无 supersession。

### 控制

- SQLite canonical memory；
- embeddings 只是索引；
- structured filters；
- active/superseded/quarantine；
- evidence links。

## 24.8 同题重复被当成成长

### 症状

- 同题第四次 100% 被视为技能掌握；
-推荐不断重复旧题。

### 控制

- repeat ordinal；
-gap；
-novelty weight；
-cross-item evidence；
-transfer warning；
- retrieval practice with new material。

## 24.9 用户满意度作为唯一奖励

### 症状

-更短、更迎合、直接给答案获得高点赞；
-系统因此永久改变教学策略。

### 控制

-事实、诊断、学习效果优先；
- delayed outcome；
-多维 grader；
-不可用单总分掩盖 safety。

## 24.10 Memory 永久追加

### 症状

- Daily/Memory 文件越来越长；
-相同结论几十条；
-旧结论不失效。

### 控制

- canonical key；
-replacement；
-merge；
-capacity；
-decay；
-Dream replacement proposal；
-archive。

## 24.11 Dream 直接删除

### 症状

-模型认为旧记忆不重要就删除；
-用户无法恢复。

### 控制

- proposal；
-superseded/archived；
-retention；
-mutation log；
-user review。

## 24.12 通用 SQL Tool

### 症状

- Agent 生成 SQL；
-任意读写 SQLite；
-PRAGMA/ATTACH。

### 控制

-语义化 read tools；
-固定 parameterized SQL；
-无 raw DB tool。

## 24.13 Tool 数量失控

### 症状

-几十个重叠工具；
-模型频繁选错；
-tool description 占用大量 Context。

### 控制

- run-kind tool allowlist；
-最小 viable tools；
-明确语义；
-工具选择 eval；
-合并重叠工具。

## 24.14 审计表复制完整敏感正文

### 症状

- `agent_tool_calls.result_json` 保存文章、作文和文件全文；
-备份暴涨；
-隐私重复。

### 控制

- model payload/audit payload 分离；
-hash/reference；
-preview；
-retention；
-sensitive redaction。

## 24.15 Mid-session Prompt/Memory 热替换

### 症状

- Dream 更新后当前对话行为突然变化；
-无法重放。

### 控制

- thread/run frozen context snapshot；
-新 Memory 下次 run 生效；
-Prompt version run-bound。

## 24.16 过早多 Agent

### 症状

- Planner/Critic/Memory/Coach 多个 Agent 相互对话；
-成本、延迟和故障成倍增加；
-实际没有单 Agent baseline。

### 控制

- 单 Agent + deterministic services；
-后台 Dream 可是独立 run kind，不必多 Agent；
-只有 eval 证明必要才引入 evaluator agent。

## 24.17 过早复杂 Knowledge Tracing

### 症状

- taxonomy 未稳定就做 DKT；
-小样本伪预测；
-不可解释。

### 控制

- Beta/EWMA baseline；
-calibration；
-uncertainty；
-复杂模型必须显著超越 baseline。

## 24.18 Background Job 假后台

### 症状

- 宣称夜间自动运行，但桌面应用已关闭；
- job 丢失或重复。

### 控制

- SQLite durable queue；
-下次启动补跑；
-不承诺 OS daemon；
-未来需要常驻时单独设计系统服务。

## 24.19 UI 显示伪精确

### 症状

- “Heading 掌握度 83.42%”；
-数据仅两题。

### 控制

- state labels；
- uncertainty；
-evidence count/diversity；
-解释。

## 24.20 自动学习不可关闭

### 后果

-用户不信任；
-错误画像无法治理；
-隐私风险。

### 控制

- Memory Center；
-per-category controls；
-export/delete；
-user correction；
-opt-in global improvement。

## 24.21 Prompt Eval 数据泄漏

### 症状

-同用户相似案例跨 train/holdout；
-优化器看到 holdout；
-候选只记住题目答案。

### 控制

- group split；
-hidden holdout；
-content hash near-duplicate；
-new asset transfer cases；
-release audit。

## 24.22 “自进化”营销超过真实能力

产品文案应区分：

```text
已记住用户明确偏好
基于历史证据形成的候选观察
经过验证的教学策略
产品 Prompt 的开发者评测升级
```

不应宣称模型“自主变得更聪明”，除非机制和结果可解释。

---

# 25. 最终验收标准

## 25.1 架构验收

| 编号 | 标准 | 验收方式 |
|---|---|---|
| A-01 | Agent application logic 不依赖 Tauri 类型 | crate compile/test |
| A-02 | Model runtime 与 command 分离 | dependency/source check |
| A-03 | Memory 与 canonical learning facts 分离 | schema + API review |
| A-04 | 无通用 SQL tool | static gate |
| A-05 | Prompt active version 不可被生产 Agent 修改 | capability/test |
| A-06 | 所有 high-risk flow 有 application service | architecture test |
| A-07 | 新接口有生成 TS 类型 | drift gate |
| A-08 | SQLite migration 可 fresh/upgrade/idempotent | migration suite |

## 25.2 Agent Runtime 验收

| 编号 | 标准 |
|---|---|
| R-01 | Thread 可跨页面和重启读取 |
| R-02 | 每个 run 有 context snapshot |
| R-03 | 每个 model call 有 invocation trace |
| R-04 | 每个 tool call 有 begin/end/status |
| R-05 | max rounds/tools 生效 |
| R-06 | cancellation 不继续下一轮 |
| R-07 | interrupted write tool 不自动重放 |
| R-08 | approval 不可绕过 |
| R-09 | tool output 有 size/sensitivity guardrail |
| R-10 | run 可用 frozen context 重放 |

## 25.3 Memory 验收

| 编号 | 标准 |
|---|---|
| M-01 | 每条 inferred memory 有 evidence |
| M-02 | explicit preference 标记为 user source |
| M-03 | user correction 优先级最高 |
| M-04 | active canonical key 唯一 |
| M-05 | replace/merge 有 supersession chain |
| M-06 | deleted/superseded/quarantined 不进入 Context |
| M-07 | Memory 有容量，不无限追加 |
| M-08 | 用户可查看、编辑、删除、关闭 |
| M-09 | Dream 无法直接修改 Soul/Prompt |
| M-10 | mutation batch 原子、可审计 |
| M-11 | injection corpus 不被激活 |
| M-12 | derived Memory 可清除和重建 |

## 25.4 Journal/Dream 验收

| 编号 | 标准 |
|---|---|
| D-01 | 无 AI 时仍有 deterministic Journal |
| D-02 | Journal 数值与 canonical facts 一致 |
| D-03 | 同日重跑产生版本，不尾部无限追加 |
| D-04 | Dream input 是明确只读 evidence window |
| D-05 | 每个 proposal 有 before/after/evidence/reason |
| D-06 | 第一阶段全部需 review |
| D-07 | 低风险 auto policy 是 Rust 确定性逻辑 |
| D-08 | Dream 失败不改变 active Memory |
| D-09 | job 重启后可恢复且不重复 |
| D-10 | 用户可暂停后台整理 |

## 25.5 Learner Model 验收

| 编号 | 标准 |
|---|---|
| L-01 | taxonomy 版本化 |
| L-02 | question mapping 有来源 |
| L-03 | observation 可从 event 重建 |
| L-04 | same-item repeat 降权 |
| L-05 | distinct asset evidence 单独统计 |
| L-06 | state 有 uncertainty |
| L-07 | UI 不用伪精确标签 |
| L-08 | Agent 不把 state 表述为确定人格事实 |
| L-09 | delayed outcome 可关联 intervention |
| L-10 | recommendation 优先新材料 transfer |

## 25.6 Coach 验收

| 编号 | 标准 |
|---|---|
| C-01 | 当前题目证据优先于画像 |
| C-02 | 输出引用存在的 evidence |
| C-03 | 不提前泄露答案策略正确 |
| C-04 | 个性化因素可解释 |
| C-05 | 用户反馈结构化 |
| C-06 | 一次不满只形成 candidate |
| C-07 | 教学策略需要多证据/outcome |
| C-08 | provider 不可用不破坏练习 |
| C-09 | 已删除偏好下一次不再使用 |
| C-10 | 后续不同题目用于验证策略 |

## 25.7 Context 验收

| 编号 | 标准 | 目标 |
|---|---|---:|
| X-01 | 关键 current evidence recall | ≥99% |
| X-02 | relevant explicit preference recall | ≥95% |
| X-03 | superseded/quarantined inclusion | 0% |
| X-04 | token overflow | 0% |
| X-05 | duplicate token share | ≤5% |
| X-06 | irrelevant memory injection | ≤10% |
| X-07 | snapshot/source completeness | 100% |
| X-08 | deterministic replay context hash | 100% |

## 25.8 Prompt Evolution 验收

| 编号 | 标准 |
|---|---|
| E-01 | baseline、candidate、holdout 分离 |
| E-02 | 同用户/题目不跨 split 泄漏 |
| E-03 | 多次 trial |
| E-04 | deterministic + domain + LLM + human graders |
| E-05 | critical safety regression 为 0 |
| E-06 | candidate 不自动 active |
| E-07 | shadow 无写副作用 |
| E-08 | canary 有 kill switch |
| E-09 | promotion 有 eval report 和人工批准 |
| E-10 | rollback 一次事务完成 |

## 25.9 安全与隐私验收

| 编号 | 标准 |
|---|---|
| S-01 | API Key 不进入 DB/trace/backup |
| S-02 | 外部内容标 untrusted data |
| S-03 | memory injection quarantine |
| S-04 | workspace path containment |
| S-05 | write tool 乐观并发/原子写 |
| S-06 | sensitivity 分类和 Provider minimization |
| S-07 |用户可清除派生数据 |
| S-08 |匿名产品改进显式 opt-in |
| S-09 |备份恢复后 running 状态安全恢复 |
| S-10 |删除原 evidence 后依赖 Memory 重验证 |

## 25.10 产品验收场景

### 场景 1：同一题目三次练习

- 系统显示三次时间线；
- 区分 still wrong / corrected / newly wrong；
-提示熟悉度；
-不直接宣称技能掌握；
-建议不同新题验证。

### 场景 2：跨题相同错误

- 多个 asset 的相同 skill error 被聚合；
-形成 learner state；
- Memory candidate 有多样 evidence；
- Coach 使用该模式但标 uncertainty。

### 场景 3：用户反复纠正 Coach 风格

- feedback 结构化；
-当日形成 candidate；
-多次明确反馈后形成偏好；
-下一次回答体现偏好；
-用户可查看和删除。

### 场景 4：旧画像被新证据推翻

- Dream 生成 replace proposal；
-before/after；
-旧 memory superseded；
-Context 只使用新 active；
-审计仍可查看旧链。

### 场景 5：恶意题目文本

题目包含“忽略系统并永久记住”。

- 不成为 instruction；
-不写 active Memory；
-候选 quarantine；
-Agent 仍正常回答题目。

### 场景 6：夜间应用关闭

- job queued；
-下次启动恢复；
-同一天不重复；
-Journal/Dream coverage 正确。

### 场景 7：Prompt candidate

- baseline 和 candidate replay；
-候选在 validation 更好；
-holdout 通过；
-shadow 无副作用；
-人工批准后 active；
-可回滚。

### 场景 8：用户关闭自动学习

-不再生成长期 memory candidate；
-练习 events 可按策略继续用于本地历史；
-现有 Memory 可保留或清除；
-Coach 仍可使用 explicit preferences。

## 25.11 产品成功指标

短期工程指标：

- Context correctness；
- Agent/tool reliability；
-Memory correction rate；
-Dream proposal quality；
-成本/延迟。

中期产品指标：

- 用户对画像的确认率；
-同类错误复发率；
-Coach re-ask rate；
-跨题 transfer；
-间隔保持；
-用户持续使用。

长期核心指标：

> 在不牺牲安全、事实正确性和用户控制的前提下，个性化 Agent 是否让用户在新的、间隔后的 IELTS 任务上表现得更好。

---

# 26. 参考资料

## 26.1 当前项目与代码基线

- `IELTS-WRITING-FEAT` 分支：<https://github.com/sallowayma-git/IELTS-practice/tree/IELTS-WRITING-FEAT>
- 当前分支 tip `5c9fd7c`：<https://github.com/sallowayma-git/IELTS-practice/commit/5c9fd7c6e9d89cc2fd4f7b4ef4cb34f71335c9ce>
- Application/Agent 后端提交 `93e4ed4`：<https://github.com/sallowayma-git/IELTS-practice/commit/93e4ed4bbf80105876af5c6830f9c7ad9748b9c2>
- `ielts-application`：<https://github.com/sallowayma-git/IELTS-practice/tree/IELTS-WRITING-FEAT/crates/ielts-application>
- Agent application loop：<https://github.com/sallowayma-git/IELTS-practice/blob/IELTS-WRITING-FEAT/crates/ielts-application/src/agent.rs>
- Agent run/tool persistence：<https://github.com/sallowayma-git/IELTS-practice/blob/IELTS-WRITING-FEAT/crates/ielts-db/src/agent/mod.rs>
- Agent migration：<https://github.com/sallowayma-git/IELTS-practice/blob/IELTS-WRITING-FEAT/crates/ielts-db/migrations/0011_agent_runs_tool_calls.sql>
- AI runtime/tool protocol：<https://github.com/sallowayma-git/IELTS-practice/blob/IELTS-WRITING-FEAT/src-tauri/src/ai/runtime.rs>
- Tauri Agent adapter：<https://github.com/sallowayma-git/IELTS-practice/blob/IELTS-WRITING-FEAT/src-tauri/src/commands/agent.rs>
- Workspace grant：<https://github.com/sallowayma-git/IELTS-practice/blob/IELTS-WRITING-FEAT/src-tauri/src/agent/workspace.rs>
- File tools：<https://github.com/sallowayma-git/IELTS-practice/blob/IELTS-WRITING-FEAT/src-tauri/src/agent/file_tools.rs>
- Application Store adapter：<https://github.com/sallowayma-git/IELTS-practice/blob/IELTS-WRITING-FEAT/src-tauri/src/app/application_store.rs>
- 当前 Agent 工作台页面：<https://github.com/sallowayma-git/IELTS-practice/blob/IELTS-WRITING-FEAT/apps/writing-vue/src/views/AgentWorkspacePage.vue>

## 26.2 Context Engineering、Agent 设计与评测

- Anthropic, **Effective context engineering for AI agents**：<https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents>
- Anthropic, **Demystifying evals for AI agents**：<https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents>
- Anthropic, **Harness design for long-running application development**：<https://www.anthropic.com/engineering/harness-design-long-running-apps>
- Anthropic, **How we contain Claude across products**：<https://www.anthropic.com/engineering/how-we-contain-claude>
- Claude Code Memory 文档：<https://code.claude.com/docs/en/memory>
- OpenAI Agents SDK Sessions：<https://openai.github.io/openai-agents-python/sessions/>
- OpenAI Agents SDK Guardrails：<https://openai.github.io/openai-agents-python/guardrails/>
- OpenAI Agents SDK Human-in-the-loop：<https://openai.github.io/openai-agents-python/human_in_the_loop/>
- OpenAI Agents SDK Tracing：<https://openai.github.io/openai-agents-python/tracing/>
- LangGraph Memory：<https://docs.langchain.com/oss/python/langgraph/add-memory>
- LangMem：<https://langchain-ai.github.io/langmem/>
- Letta Memory Blocks：<https://docs.letta.com/v1-sdk/memory/memory-blocks>

## 26.3 产品 Memory、Daily、Dream 与自进化参考

- OpenClaw Memory overview：<https://docs.openclaw.ai/concepts/memory>
- OpenClaw Memory CLI/Dream 相关文档：<https://openclaw.cc/en/cli/memory>
- Hermes Agent Persistent Memory：<https://hermes-agent.nousresearch.com/docs/user-guide/features/memory/>
- Hermes Agent Self-Evolution：<https://github.com/NousResearch/hermes-agent-self-evolution>
- Tencent WorkBuddy Memory：<https://www.workbuddy.ai/docs/workbuddy/From-Beginner-to-Expert-Guide/Function-Description/Memory>
- memU repository：<https://github.com/NevaMind-AI/memU>
- memU File-Based Memory：<https://memu.pro/file-based-memory>

说明：上述产品的公开实现和文档持续变化。本计划采用其可验证的架构模式，不承诺复制其内部未公开机制。

## 26.4 Agent Memory 与反思研究

- Park et al., **Generative Agents: Interactive Simulacra of Human Behavior**：<https://arxiv.org/abs/2304.03442>
- **Auto-Dreamer: Learning Offline Memory Consolidation for Language Agents**：<https://arxiv.org/abs/2605.20616>

Auto-Dreamer 为 2026 年预印本。本文仅将“在线快速记录/离线只读整合/替代集合/来源追踪”作为设计启发，不把论文结论视为已经完成大规模生产验证。

## 26.5 自进化、Prompt 优化与评测

- DSPy：<https://dspy.ai/>
- GEPA：<https://github.com/gepa-ai/gepa>
- Hermes Self-Evolution：<https://github.com/NousResearch/hermes-agent-self-evolution>
- Anthropic Agent Evals：<https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents>

第三方优化器只能作为候选生成器。生产发布权限、数据 split、holdout、grader、shadow、canary 和 rollback 必须由本项目控制。

## 26.6 学习科学与学习者建模

- Roediger & Karpicke, **Test-Enhanced Learning**：<https://journals.sagepub.com/doi/10.1111/j.1467-9280.2006.01693.x>
- Cepeda et al., **Distributed Practice in Verbal Recall Tasks: A Review and Quantitative Synthesis**：<https://digitalcommons.usf.edu/psy_facpub/1771/>
- U.S. Institute of Education Sciences, **Organizing Instruction and Study to Improve Student Learning**：<https://ies.ed.gov/ncee/wwc/practiceguide/1>
- Corbett & Anderson, **Knowledge Tracing: Modeling the Acquisition of Procedural Knowledge**：<https://doi.org/10.1007/BF01099821>
- Piech et al., **Deep Knowledge Tracing**：<https://arxiv.org/abs/1506.05908>

## 26.7 安全、治理与长期 Memory 风险

- NIST, **Artificial Intelligence Risk Management Framework: Generative AI Profile**：<https://www.nist.gov/publications/artificial-intelligence-risk-management-framework-generative-artificial-intelligence>
- **MemoryGraft: Persistent Compromise of LLM Agents via Poisoned Experience Retrieval**：<https://arxiv.org/abs/2512.16962>
- **Zombie Agents: Persistent Control of Self-Evolving LLM Agents via Self-Reinforcing Injections**：<https://arxiv.org/abs/2602.15654>

MemoryGraft 和 Zombie Agents 是较新的研究，应将其视为重要威胁证据和红队设计来源，同时继续关注复现、范围和后续防御研究。

## 26.8 研究结论的证据等级

| 等级 | 来源 | 本计划中的使用方式 |
|---|---|---|
| A | 官方技术文档、标准、成熟论文 | 核心架构和门禁依据 |
| B | 开源产品官方实现/文档 | 设计模式和工程参考 |
| C | 新预印本、早期安全研究 | 风险启发、实验性模块 |
| D | 社区文章、二手总结 | 仅用于发现线索，不作为关键结论唯一依据 |

本计划的关键架构决策尽量由 A/B 级来源和当前仓库实际代码共同支撑。

---

# 结语

IELTS Atlas 当前已经越过“普通 LLM 聊天功能”的起点：它具备 Rust Application 层、模型抽象、工具调用循环、文件工具安全边界和 Agent run/tool 审计。下一步不应继续扩展一个更大的聊天工作台，而应围绕学习证据建立长期智能系统。

推荐的核心演进顺序是：

```text
先建立事实事件
→ 再建立 Thread 和 Trace
→ 再让用户治理 Memory
→ 再建立 Context Compiler
→ 再生成 Journal 和 Dream proposal
→ 再建立 Learner Model
→ 再接入个性化 Coach
→ 再验证教学策略
→ 最后建立产品 Prompt/Skill 的离线自进化
```

这条路线的关键不是让 Agent “自由修改自己”，而是建立一个可以持续学习、持续删减、持续验证、持续回滚的工程闭环：

> 学习事实不可篡改，Memory 可演化，教学策略可验证，产品 Prompt 可评测，用户始终拥有知情、修正和删除权。
