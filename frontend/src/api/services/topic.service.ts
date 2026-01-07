import type {
  PaginationParams,
  TopicCreateResponse,
  TopicDetails,
  TopicsGetResponse,
  TopicsPostAdminRequest,
  TopicsPostOfficeRequest,
  TopicsPostRequest,
  TopicsPostTeacherRequest,
} from '../models'
import type { HttpClient } from '../utils/http-client'

export class TopicService {
  constructor (private http: HttpClient) {}

  async getTopics (params?: PaginationParams): Promise<TopicsGetResponse> {
    return this.http.get<TopicsGetResponse>('/topics', { params })
  }

  async searchTopics (keyword?: string, params?: PaginationParams): Promise<TopicsGetResponse> {
    return this.http.get<TopicsGetResponse>('/topics/search', {
      params: { keyword, ...params },
    })
  }

  async getTopicById (topicId: number): Promise<TopicDetails> {
    return this.http.get<TopicDetails>(`/topics/${topicId}`)
  }

  async createTopic (data: TopicsPostRequest): Promise<TopicCreateResponse> {
    return this.http.post<TopicCreateResponse>('/topics', data)
  }

  async updateTopicAsTeacher (
    topicId: number,
    data: TopicsPostTeacherRequest,
  ): Promise<TopicDetails> {
    return this.http.patch<TopicDetails>(`/topics/${topicId}`, data)
  }

  async updateTopicAsOffice (topicId: number, data: TopicsPostOfficeRequest): Promise<TopicDetails> {
    return this.http.patch<TopicDetails>(`/topics/${topicId}`, data)
  }

  async updateTopicAsAdmin (topicId: number, data: TopicsPostAdminRequest): Promise<TopicDetails> {
    return this.http.patch<TopicDetails>(`/topics/${topicId}`, data)
  }
}
