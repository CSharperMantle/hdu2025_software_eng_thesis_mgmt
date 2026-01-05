import type {
  AssignmentStatus,
  ProgressOutcome,
  TopicReviewStatus,
  TopicType,
} from './models'

// Topic Types
export const TOPIC_TYPE_MAP = new Map<TopicType, string>([
  [0, '理论研究型'],
  [1, '应用开发型'],
  [2, '实验研究型'],
  [3, '工程设计型'],
  [4, '其他'],
])

export const TOPIC_TYPES = Array.from(TOPIC_TYPE_MAP.entries()).map(([value, name]) => ({
  value,
  name,
}))

// Topic Review Status
export const TOPIC_REVIEW_STATUS_MAP = new Map<TopicReviewStatus, { name: string, color: string }>([
  [0, { name: '待审核', color: 'warning' }],
  [1, { name: '已通过', color: 'success' }],
  [2, { name: '已拒绝', color: 'error' }],
])

export const TOPIC_REVIEW_STATUSES = Array.from(TOPIC_REVIEW_STATUS_MAP.entries()).map(([value, { name, color }]) => ({
  value,
  name,
  color,
}))

// Assignment Status
export const ASSIGNMENT_STATUS_MAP = new Map<AssignmentStatus, string>([
  [0, '待审核'],
  [1, '已通过'],
  [2, '已拒绝'],
])

export const ASSIGNMENT_STATUSES = Array.from(ASSIGNMENT_STATUS_MAP.entries()).map(([value, name]) => ({
  value,
  name,
}))

// Progress Outcome
export const PROGRESS_OUTCOME_MAP = new Map<ProgressOutcome, string>([
  [0, '无结论'],
  [1, '已通过'],
  [2, '已打回'],
])

export const PROGRESS_OUTCOMES = Array.from(PROGRESS_OUTCOME_MAP.entries()).map(([value, name]) => ({
  value,
  name,
}))

// Helper functions
export function getTopicTypeName(type: TopicType): string {
  return TOPIC_TYPE_MAP.get(type) || '未知'
}

export function getTopicReviewStatusName(status: TopicReviewStatus): string {
  return TOPIC_REVIEW_STATUS_MAP.get(status)?.name || '未知'
}

export function getTopicReviewStatusColor(status: TopicReviewStatus): string {
  return TOPIC_REVIEW_STATUS_MAP.get(status)?.color || 'default'
}

export function getAssignmentStatusName(status: AssignmentStatus): string {
  return ASSIGNMENT_STATUS_MAP.get(status) || '未知'
}

export function getProgressOutcomeName(outcome: ProgressOutcome): string {
  return PROGRESS_OUTCOME_MAP.get(outcome) || '未知'
}
