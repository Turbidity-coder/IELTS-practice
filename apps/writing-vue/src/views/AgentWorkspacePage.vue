<template>
  <section class="agent-page agent-workspace-page" data-agent-workspace>
    <header class="agent-page-header">
      <div class="agent-page-header__copy">
        <p class="agent-page-header__eyebrow">AI workspace</p>
        <h1>Agent 工作区</h1>
        <p class="agent-page-header__lede">把题目上下文、提示词和运行结果放在同一张工作台上。</p>
      </div>
      <div class="agent-page-header__status" :class="`is-${runState}`" role="status">
        <span class="agent-status-dot" aria-hidden="true"></span>
        <span>{{ runStateLabel }}</span>
      </div>
    </header>

    <div class="agent-workbench">
      <aside class="agent-panel agent-sidebar" aria-label="工作区文件">
        <div class="agent-panel__head agent-sidebar__head">
          <div>
            <p class="agent-panel__eyebrow">Workspace</p>
            <h2>本地工作区</h2>
          </div>
          <button
            class="agent-icon-button"
            type="button"
            aria-label="清除工作区选择"
            title="清除工作区选择"
            :disabled="workspaceLocked"
            @click="resetWorkspace"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M3 12a9 9 0 0 1 15.3-6.4L21 8"></path>
              <path d="M21 3v5h-5"></path>
              <path d="M21 12a9 9 0 0 1-15.3 6.4L3 16"></path>
              <path d="M3 21v-5h5"></path>
            </svg>
          </button>
        </div>

        <button class="agent-workspace-select" type="button" :disabled="workspaceLocked" @click="pickWorkspace">
          <span class="agent-workspace-select__icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H10l2 2h6.5A2.5 2.5 0 0 1 21 9.5v7A2.5 2.5 0 0 1 18.5 19h-13A2.5 2.5 0 0 1 3 16.5v-9Z"></path>
            </svg>
          </span>
          <span class="agent-workspace-select__copy">
            <strong>{{ workspaceName }}</strong>
            <small>{{ workspaceStatus }}</small>
          </span>
          <svg class="agent-workspace-select__chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="m9 18 6-6-6-6"></path>
          </svg>
        </button>

        <div class="agent-file-tree">
          <div class="agent-file-tree__label">
            <span>已访问文件</span>
            <span>{{ files.length }}</span>
          </div>
          <button
            v-for="file in files"
            :key="file.path"
            class="agent-file-row"
            :class="{ 'is-selected': selectedFile === file.path }"
            type="button"
            @click="selectFile(file.path)"
          >
            <span class="agent-file-row__icon" :class="`is-${file.kind}`" aria-hidden="true">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M6 3h8l4 4v14H6z"></path>
                <path d="M14 3v5h5"></path>
                <path d="M9 13h6M9 17h6"></path>
              </svg>
            </span>
            <span class="agent-file-row__copy">
              <strong>{{ file.name }}</strong>
              <small>{{ file.meta }}</small>
            </span>
            <span v-if="selectedFile === file.path" class="agent-file-row__marker" aria-hidden="true"></span>
          </button>
          <p v-if="files.length === 0" class="agent-file-tree__empty">
            {{ workspaceGrant ? 'Agent 访问文件后会显示在这里。' : '选择工作区后开始运行。' }}
          </p>
        </div>

        <div class="agent-sidebar__footer">
          <span class="agent-sidebar__footer-dot" aria-hidden="true"></span>
          <span>{{ workspaceGrant ? '短期本地授权' : '尚未授权工作区' }}</span>
        </div>
      </aside>

      <section class="agent-panel agent-prompt-panel" aria-label="提示词工作区">
        <div class="agent-panel__head">
          <div>
            <p class="agent-panel__eyebrow">Prompt</p>
            <h2>协作提示词</h2>
          </div>
          <span class="agent-model-badge">{{ modelLabel }}</span>
        </div>

        <div class="agent-prompt-toolbar" role="toolbar" aria-label="提示词工具">
          <div class="agent-segmented-control" role="tablist" aria-label="提示词模式">
            <button
              v-for="mode in promptModes"
              :key="mode.value"
              type="button"
              :class="{ 'is-active': promptMode === mode.value }"
              role="tab"
              :aria-selected="promptMode === mode.value"
              @click="promptMode = mode.value"
            >
              {{ mode.label }}
            </button>
          </div>
          <button class="agent-text-button" type="button" @click="resetPrompt">恢复示例</button>
        </div>

        <label class="agent-prompt-editor">
          <span class="sr-only">协作提示词</span>
          <textarea v-model="promptText" rows="12" spellcheck="false"></textarea>
          <span class="agent-prompt-editor__meta">{{ promptText.length }} characters</span>
        </label>

        <div class="agent-context-strip">
          <div class="agent-context-strip__label">
            <span class="agent-context-strip__icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H20v17H6.5A2.5 2.5 0 0 0 4 22V5.5Z"></path>
                <path d="M4 5.5V19"></path>
              </svg>
            </span>
            <span>上下文</span>
          </div>
          <button v-if="workspaceGrant" class="agent-context-chip" type="button" :disabled="workspaceLocked" @click="pickWorkspace">
            <span>{{ selectedFileName }}</span>
            <span aria-hidden="true">↗</span>
          </button>
          <button class="agent-add-context" type="button" aria-label="选择工作区" title="选择工作区" :disabled="workspaceLocked" @click="pickWorkspace">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M12 5v14M5 12h14"></path>
            </svg>
          </button>
        </div>

        <div class="agent-prompt-footer">
          <span class="agent-prompt-footer__hint">{{ promptHint }}</span>
          <button class="agent-run-button" type="button" :disabled="!canRun" @click="runAgent">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="m8 5 11 7-11 7V5Z"></path>
            </svg>
            {{ runState === 'running' ? '运行中' : '运行 Agent' }}
          </button>
        </div>
      </section>

      <aside class="agent-panel agent-run-panel" aria-label="运行状态">
        <div class="agent-panel__head">
          <div>
            <p class="agent-panel__eyebrow">Run log</p>
            <h2>运行状态</h2>
          </div>
          <span class="agent-run-count">#{{ runIdShort }}</span>
        </div>

        <div class="agent-run-summary" :class="`is-${runState}`">
          <span class="agent-run-summary__icon" aria-hidden="true">
            <svg v-if="runState === 'complete'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="m5 12 4 4L19 6"></path></svg>
            <svg v-else-if="runState === 'running'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v4M12 17v4M3 12h4M17 12h4M5.6 5.6l2.8 2.8M15.6 15.6l2.8 2.8M18.4 5.6l-2.8 2.8M8.4 15.6l-2.8 2.8"></path></svg>
            <svg v-else-if="runState === 'error'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="m6 6 12 12M18 6 6 18"></path></svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v18M3 12h18"></path></svg>
          </span>
          <div>
            <strong>{{ runStateLabel }}</strong>
            <span>{{ runStateDetail }}</span>
          </div>
        </div>

        <ol class="agent-run-steps">
          <li v-for="step in runSteps" :key="step.key" :class="`is-${step.state}`">
            <span class="agent-run-step__index">{{ step.index }}</span>
            <span class="agent-run-step__copy">
              <strong>{{ step.label }}</strong>
              <small>{{ step.detail }}</small>
            </span>
            <span class="agent-run-step__state" aria-hidden="true"></span>
          </li>
        </ol>

        <div class="agent-output-panel">
          <div class="agent-output-panel__head">
            <span>输出</span>
            <span v-if="lastRunAt">{{ lastRunAt }}</span>
          </div>
          <p>{{ outputText }}</p>
          <dl v-if="runMetadata.length" class="agent-output-metadata">
            <div v-for="item in runMetadata" :key="item.label">
              <dt>{{ item.label }}</dt>
              <dd :title="item.value">{{ item.value }}</dd>
            </div>
          </dl>
        </div>
      </aside>
    </div>
  </section>
</template>

<script setup>
import { computed, ref } from 'vue'
import agentRepository from '@/api/agent-repository.js'

const defaultPrompt = '请先阅读已选上下文，提炼关键事实，再给出一份简洁、可执行的学习建议。'
const promptText = ref(defaultPrompt)
const promptMode = ref('assist')
const selectedFile = ref('')
const workspaceGrant = ref(null)
const workspaceBusy = ref(false)
const runState = ref('idle')
const lastRun = ref(null)
const lastRunAt = ref('')
const outputText = ref('选择本地工作区后，运行结果会出现在这里。')
const workspaceLocked = computed(() => workspaceBusy.value || runState.value === 'running')

const promptModes = [
  { value: 'assist', label: '辅助' },
  { value: 'plan', label: '规划' }
]

const files = computed(() => {
  const byPath = new Map()
  for (const call of lastRun.value?.toolCalls || []) {
    const path = String(call.arguments?.path || call.result?.path || '').trim()
    if (!path || byPath.has(path)) continue
    const name = path.split(/[\\/]/).filter(Boolean).pop() || path
    byPath.set(path, {
      path,
      name,
      kind: fileKind(name),
      meta: `${call.toolName} · ${toolStatusLabel(call.status)}`
    })
  }
  return [...byPath.values()]
})
const workspaceName = computed(() => {
  const path = workspaceGrant.value?.displayPath || ''
  return path.split(/[\\/]/).filter(Boolean).pop() || '选择本地工作区'
})
const workspaceStatus = computed(() => workspaceGrant.value?.displayPath || '仅授权所选目录')
const selectedFileName = computed(() => {
  return files.value.find((file) => file.path === selectedFile.value)?.name || workspaceName.value
})
const modelLabel = computed(() => lastRun.value?.actualModel || '本地配置模型')
const canRun = computed(() => {
  return Boolean(workspaceGrant.value && promptText.value.trim() && !workspaceLocked.value)
})
const promptHint = computed(() => {
  if (!workspaceGrant.value) return '先选择一个本地工作区'
  return promptMode.value === 'plan' ? '先整理步骤，再开始运行' : '准备好后运行 Agent'
})
const runIdShort = computed(() => lastRun.value?.id?.slice(0, 8) || '--')
const runStateLabel = computed(() => ({
  idle: '待命',
  running: '运行中',
  complete: '已完成',
  error: '运行失败'
})[runState.value])
const runStateDetail = computed(() => {
  if (runState.value === 'running') return '模型与工具正在执行'
  if (runState.value === 'complete') return `${lastRun.value?.rounds || 0} 轮 · ${lastRun.value?.toolCallCount || 0} 次工具调用`
  if (runState.value === 'error') return '查看输出中的错误信息'
  return workspaceGrant.value ? '等待提示词' : '等待工作区授权'
})
const runSteps = computed(() => {
  const steps = [{
    key: 'workspace',
    label: '工作区授权',
    detail: workspaceGrant.value ? workspaceName.value : '尚未选择',
    state: workspaceGrant.value ? 'complete' : 'pending'
  }]
  if (runState.value === 'running') {
    steps.push({ key: 'run', label: '执行 Agent', detail: '等待模型返回', state: 'active' })
  }
  for (const call of lastRun.value?.toolCalls || []) {
    const path = call.arguments?.path || call.result?.path || `round ${call.round}`
    steps.push({
      key: `${call.sequence}-${call.callId}`,
      label: call.toolName,
      detail: `${toolStatusLabel(call.status)} · ${path}`,
      state: call.status === 'succeeded' ? 'complete' : call.status === 'running' ? 'active' : 'error'
    })
  }
  steps.push({
    key: 'result',
    label: '最终结果',
    detail: lastRun.value ? `${lastRun.value.rounds} 轮 · run ${runIdShort.value}` : '尚未运行',
    state: runState.value === 'complete' ? 'complete' : runState.value === 'error' ? 'error' : 'pending'
  })
  return steps.map((step, index) => ({ ...step, index: String(index + 1).padStart(2, '0') }))
})
const runMetadata = computed(() => {
  const run = lastRun.value
  if (!run) return []
  const tokens = run.usage ? `${run.usage.inputTokens} in / ${run.usage.outputTokens} out` : '未返回'
  return [
    { label: 'Run ID', value: run.id },
    { label: 'Actual model', value: run.actualModel || '未返回' },
    { label: 'Latency', value: `${run.latencyMs} ms` },
    { label: 'Usage', value: tokens },
    { label: 'Retries', value: String(run.retryCount) },
    { label: 'Request ID', value: run.providerRequestId || '未返回' },
    { label: 'Prompt hash', value: run.promptHash || '未返回' }
  ]
})

function selectFile(path) {
  selectedFile.value = path
}

async function pickWorkspace() {
  if (workspaceLocked.value) return
  workspaceBusy.value = true
  try {
    const grant = await agentRepository.pickWorkspace()
    if (!grant) return
    workspaceGrant.value = grant
    selectedFile.value = ''
    lastRun.value = null
    runState.value = 'idle'
    outputText.value = '工作区已授权，可以开始运行。'
    lastRunAt.value = ''
  } catch (error) {
    showError(error)
  } finally {
    workspaceBusy.value = false
  }
}

function resetPrompt() {
  promptText.value = defaultPrompt
}

function resetWorkspace() {
  if (workspaceLocked.value) return
  workspaceGrant.value = null
  selectedFile.value = ''
  lastRun.value = null
  runState.value = 'idle'
  outputText.value = '选择本地工作区后，运行结果会出现在这里。'
  lastRunAt.value = ''
}

async function runAgent() {
  if (!canRun.value) return
  runState.value = 'running'
  lastRun.value = null
  outputText.value = '正在执行模型与工作区工具…'
  lastRunAt.value = ''
  try {
    const outcome = await agentRepository.run({
      grantId: workspaceGrant.value.grantId,
      prompt: promptText.value
    })
    const record = await agentRepository.getRun(outcome.runId)
    if (!record) throw new Error(`Agent run ${outcome.runId} could not be reloaded from SQLite`)
    const run = agentRepository.normalizeRun(outcome, record)
    lastRun.value = run
    selectedFile.value = files.value[0]?.path || ''
    outputText.value = run.content || 'Agent 已完成，但未返回正文。'
    lastRunAt.value = formatTime(run.completedAt)
    runState.value = run.status === 'completed' ? 'complete' : 'error'
  } catch (error) {
    const failedRun = await hydrateFailedRun(error)
    if (failedRun) {
      lastRun.value = failedRun
      selectedFile.value = files.value[0]?.path || ''
    }
    showError(error, failedRun?.completedAt)
  }
}

async function hydrateFailedRun(error) {
  const runId = String(error?.context?.runId || '').trim()
  if (!runId) return null
  try {
    const record = await agentRepository.getRun(runId)
    return record ? agentRepository.normalizeRun(null, record) : null
  } catch {
    return null
  }
}

function showError(error, completedAt) {
  runState.value = 'error'
  outputText.value = error?.message || 'Agent 运行失败。'
  lastRunAt.value = formatTime(completedAt)
}

function formatTime(value) {
  const date = value ? new Date(value) : new Date()
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function fileKind(name) {
  if (/\.md$/i.test(name)) return 'markdown'
  if (/\.json$/i.test(name)) return 'json'
  return 'text'
}

function toolStatusLabel(status) {
  return ({
    running: '执行中',
    succeeded: '已完成',
    rejected: '已拒绝',
    failed: '失败',
    interrupted: '已中断'
  })[status] || status || '未知'
}
</script>
