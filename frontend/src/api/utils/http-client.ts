import type { AxiosError, AxiosInstance, AxiosRequestConfig } from 'axios'
import axios from 'axios'
import {
  ApiError,
  AuthenticationError,
  AuthorizationError,
  NotFoundError,
  ValidationError,
} from '../exceptions'

export interface HttpClientConfig {
  baseURL: string
  timeout?: number
  headers?: Record<string, string>
}

export class HttpClient {
  private client: AxiosInstance

  constructor(config: HttpClientConfig) {
    this.client = axios.create({
      baseURL: config.baseURL,
      timeout: config.timeout || 30_000,
      headers: {
        'Content-Type': 'application/json',
        ...config.headers,
      },
      withCredentials: true, // Enable sending cookies with requests
    })

    this.setupInterceptors()
  }

  async get<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.get<T>(url, config)
    return response.data
  }

  async post<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.post<T>(url, data, config)
    return response.data
  }

  async put<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.put<T>(url, data, config)
    return response.data
  }

  async patch<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.patch<T>(url, data, config)
    return response.data
  }

  async delete<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.delete<T>(url, config)
    return response.data
  }

  async upload<T>(url: string, formData: FormData, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.post<T>(url, formData, {
      ...config,
      headers: {
        'Content-Type': 'multipart/form-data',
        ...config?.headers,
      },
      timeout: config?.timeout || 90_000,
    })
    return response.data
  }

  private setupInterceptors() {
    this.client.interceptors.response.use(
      (response) => response,
      (error: AxiosError) => {
        return Promise.reject(this.handleError(error))
      },
    )
  }

  private handleError(error: AxiosError): Error {
    if (!error.response) {
      return new ApiError(error.message || 'Network error')
    }

    const { status, data } = error.response
    const message = (data as any)?.detail || (data as any)?.message || error.message

    switch (status) {
      case 401: {
        return new AuthenticationError(message)
      }
      case 403: {
        return new AuthorizationError(message)
      }
      case 404: {
        return new NotFoundError(message)
      }
      case 400:
      case 422: {
        return new ValidationError(message, data)
      }
      default: {
        return new ApiError(message, status, data)
      }
    }
  }
}
