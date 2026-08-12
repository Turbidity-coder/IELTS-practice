export function resolveFeatureFlag(value, defaultValue) {
  if (value === undefined || value === null || String(value).trim() === '') {
    return defaultValue
  }
  return String(value).trim().toLowerCase() === 'true'
}

const buildEnvironment = typeof import.meta.env === 'object' ? import.meta.env : {}

export const featureFlags = Object.freeze({
  agentWorkspaceV1: resolveFeatureFlag(
    buildEnvironment.VITE_FEATURE_AGENT_WORKSPACE_V1,
    true
  ),
  readingAttemptReviewV1: resolveFeatureFlag(
    buildEnvironment.VITE_FEATURE_READING_ATTEMPT_REVIEW_V1,
    false
  )
})
