import { HttpClient } from '../utils/http-client'
import type {
  FinalDefensesGetResponse,
  FinalDefensesPostRequest,
  FinalDefensesRecordTeacherPatchRequest,
  FinalDefensesRecordDefenseBoardPatchRequest,
  FinalDefenseDetails,
} from '../models'

export class FinalDefenseService {
  constructor(private http: HttpClient) {}

  async getFinalDefenses(): Promise<FinalDefensesGetResponse> {
    return this.http.get<FinalDefensesGetResponse>('/final_defenses')
  }

  async createFinalDefense(data: FinalDefensesPostRequest): Promise<void> {
    await this.http.post('/final_defenses', data)
  }

  async updateFinalDefenseAsTeacher(
    reportId: number,
    data: FinalDefensesRecordTeacherPatchRequest
  ): Promise<FinalDefenseDetails> {
    return this.http.patch<FinalDefenseDetails>(`/final_defenses/${reportId}`, data)
  }

  async updateFinalDefenseAsDefenseBoard(
    reportId: number,
    data: FinalDefensesRecordDefenseBoardPatchRequest
  ): Promise<FinalDefenseDetails> {
    return this.http.patch<FinalDefenseDetails>(`/final_defenses/${reportId}`, data)
  }
}
