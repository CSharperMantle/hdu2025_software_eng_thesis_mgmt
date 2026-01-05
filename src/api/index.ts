// Main client
export { ThesisManagementApiClient, createApiClient } from './client'

// Services
export { AuthService } from './services/auth.service'
export { TopicService } from './services/topic.service'
export { AssignmentService } from './services/assignment.service'
export { ProgressReportService } from './services/progress-report.service'
export { FinalDefenseService } from './services/final-defense.service'

// Models
export * from './models'

// Exceptions
export * from './exceptions'

// Utils
export { HttpClient } from './utils/http-client'
export type { HttpClientConfig } from './utils/http-client'
