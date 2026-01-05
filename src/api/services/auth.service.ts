import { HttpClient } from '../utils/http-client'
import type {
  LoginRequest,
  UserGetResponse,
  UserPatchRequest,
  UserPostRequest,
} from '../models'

export class AuthService {
  constructor(private http: HttpClient) {}

  async login(data: LoginRequest): Promise<void> {
    await this.http.post('/login', data)
  }

  async logout(): Promise<void> {
    await this.http.post('/logout')
  }

  async getCurrentUser(): Promise<UserGetResponse> {
    return this.http.get<UserGetResponse>('/user')
  }

  async updateCurrentUser(data: UserPatchRequest): Promise<void> {
    await this.http.patch('/user', data)
  }

  async createUser(data: UserPostRequest): Promise<UserGetResponse> {
    return this.http.post<UserGetResponse>('/user', data)
  }
}
