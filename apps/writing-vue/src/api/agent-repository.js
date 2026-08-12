import { invokeCommand, unwrapCommandResponse } from '@/api/tauri-bridge.js'

export async function pickAgentWorkspace() {
  const response = await invokeCommand('agent_pick_workspace')
  return unwrapCommandResponse(response, 'agent_pick_workspace')
}

export async function runWorkspaceAgent(payload) {
  const grantId = String(payload?.grantId || '').trim()
  const prompt = String(payload?.prompt || '').trim()
  if (!grantId) throw new TypeError('agent workspace grant is required')
  if (!prompt) throw new TypeError('agent prompt is required')

  const response = await invokeCommand('agent_run', {
    request: {
      grantId,
      prompt,
      configId: payload?.configId || null
    }
  })
  return unwrapCommandResponse(response, 'agent_run')
}

export async function getAgentRun(runId) {
  const id = String(runId || '').trim()
  if (!id) throw new TypeError('agent run id is required')
  const response = await invokeCommand('agent_get_run', { runId: id })
  return unwrapCommandResponse(response, 'agent_get_run')
}

export function normalizeAgentRun(outcome, record) {
  const result = record?.result && typeof record.result === 'object' ? record.result : {}
  const usage = outcome?.usage || result.usage || null
  return {
    id: record?.id || outcome?.runId || '',
    status: record?.status || (outcome ? 'completed' : 'running'),
    content: outcome?.content || '',
    rounds: numberOr(record?.rounds, outcome?.rounds),
    toolCallCount: numberOr(record?.toolCallCount, outcome?.toolCalls),
    toolCalls: Array.isArray(record?.toolCalls) ? record.toolCalls : [],
    actualModel: outcome?.actualModel || outcome?.model || result.actualModel || null,
    latencyMs: numberOr(outcome?.latencyMs, result.latencyMs),
    retryCount: numberOr(outcome?.retryCount, result.retryCount),
    usage,
    providerRequestId: outcome?.providerRequestId || result.providerRequestId || null,
    promptHash: outcome?.promptHash || result.promptHash || '',
    error: record?.error || null,
    completedAt: record?.completedAt || null
  }
}

function numberOr(primary, fallback) {
  const value = primary ?? fallback ?? 0
  return Number.isFinite(Number(value)) ? Number(value) : 0
}

export const agentRepository = {
  pickWorkspace: pickAgentWorkspace,
  run: runWorkspaceAgent,
  getRun: getAgentRun,
  normalizeRun: normalizeAgentRun
}

export default agentRepository
