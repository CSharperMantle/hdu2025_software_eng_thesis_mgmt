import { HttpClient } from '../utils/http-client'
import type {
  PaginationParams,
  AssignmentsGetResponse,
  AssignmentsPostRequest,
  AssignmentRecordPatchRequest,
} from '../models'

export class AssignmentService {
  constructor(private http: HttpClient) {}

  async getAssignments(params?: PaginationParams): Promise<AssignmentsGetResponse> {
    return this.http.get<AssignmentsGetResponse>('/assignments', { params })
  }

  async createAssignment(data: AssignmentsPostRequest): Promise<void> {
    await this.http.post('/assignments', data)
  }

  async updateAssignmentStatus(
    studentId: number,
    topicId: number,
    data: AssignmentRecordPatchRequest
  ): Promise<void> {
    await this.http.patch(`/assignments/${studentId}/${topicId}`, data)
  }
}
