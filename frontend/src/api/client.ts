import type { PingResponse } from './models'
import type { HttpClientConfig } from './utils/http-client'
import { AssignmentService } from './services/assignment.service'
import { AuthService } from './services/auth.service'
import { FinalDefenseService } from './services/final-defense.service'
import { ProgressReportService } from './services/progress-report.service'
import { TopicService } from './services/topic.service'
import { HttpClient } from './utils/http-client'

export class ThesisManagementApiClient {
  public auth: AuthService
  public topics: TopicService
  public assignments: AssignmentService
  public progressReports: ProgressReportService
  public finalDefenses: FinalDefenseService
  private httpClient: HttpClient

  constructor(config: HttpClientConfig) {
    this.httpClient = new HttpClient(config)

    this.auth = new AuthService(this.httpClient)
    this.topics = new TopicService(this.httpClient)
    this.assignments = new AssignmentService(this.httpClient)
    this.progressReports = new ProgressReportService(this.httpClient)
    this.finalDefenses = new FinalDefenseService(this.httpClient)
  }

  async ping(): Promise<PingResponse> {
    return this.httpClient.get<PingResponse>('/ping')
  }
}

export function createApiClient(
  baseURL: string,
  config?: Partial<HttpClientConfig>,
): ThesisManagementApiClient {
  return new ThesisManagementApiClient({
    baseURL,
    ...config,
  })
}
