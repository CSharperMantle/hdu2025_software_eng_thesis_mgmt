// Main client
export { createApiClient, ThesisManagementApiClient } from './client'

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

// Utils
export { HttpClient } from './utils/http-client'
export type { HttpClientConfig } from './utils/http-client'
export { getErrorMessage } from './utils/error-messages'

// Constants
export * from './constants'
