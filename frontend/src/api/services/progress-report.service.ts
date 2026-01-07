import type {
  ProgressReportDetailResponse,
  ProgressReportRecordPatchRequest,
  ProgressReportsGetResponse,
  ProgressReportsPostRequest,
} from '../models'
import type { HttpClient } from '../utils/http-client'

export class ProgressReportService {
  constructor (private http: HttpClient) {}

  async getProgressReports (): Promise<ProgressReportsGetResponse> {
    return this.http.get<ProgressReportsGetResponse>('/progress_reports')
  }

  async createProgressReport (data: ProgressReportsPostRequest): Promise<void> {
    await this.http.post('/progress_reports', data)
  }

  async updateProgressReport (
    reportId: number,
    data: ProgressReportRecordPatchRequest,
  ): Promise<ProgressReportDetailResponse> {
    return this.http.patch<ProgressReportDetailResponse>(`/progress_reports/${reportId}`, data)
  }
}
