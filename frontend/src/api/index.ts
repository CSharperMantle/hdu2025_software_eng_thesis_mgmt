// Main client
export { createApiClient, ThesisManagementApiClient } from './client'

// Constants
export * from './constants'
// Exceptions
export * from './exceptions'
// Models
export * from './models'
export { AssignmentService } from './services/assignment.service'
// Services
export { AuthService } from './services/auth.service'

export { FinalDefenseService } from './services/final-defense.service'

export { ProgressReportService } from './services/progress-report.service'

export { TopicService } from './services/topic.service'
export { getErrorMessage } from './utils/error-messages'
// Utils
export { HttpClient } from './utils/http-client'

export type { HttpClientConfig } from './utils/http-client'
