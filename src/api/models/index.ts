// Common types
export type UserRole = 'admin' | 'student' | 'teacher' | 'defense_board' | 'office'

export type TopicType = 0 | 1 | 2 | 3 | 4
export type TopicReviewStatus = 0 | 1 | 2
export type ProgressReportType = 0 | 1
export type ProgressOutcome = 0 | 1 | 2
export type AssignmentStatus = 0 | 1 | 2

// Ping
export interface PingResponse {
  message: 'pong'
}

// User & Auth models
export interface LoginRequest {
  username: string
  password: string
}

export interface UserGetResponse {
  id: number
  username: string
  role: UserRole
  name?: string
  avatar?: string
}

export interface UserPatchRequest {
  name?: string
  password?: string
  avatar?: string
}

export interface UserPostRequest {
  username: string
  password: string
  role: UserRole
  name?: string
  major_id?: number
  avatar?: string
}

// Topic models
export interface TopicBrief {
  topic_id: number
  teacher_name: string
  topic_name: string
  topic_max_students: number
  topic_type: TopicType
  current_student_count: number
}

export interface TopicDetails {
  topic_id: number
  major_id: number
  major_name: string
  teacher_id: number
  teacher_name: string
  topic_name: string
  topic_description: string
  topic_max_students: number
  topic_type: TopicType
  topic_review_status: TopicReviewStatus
  current_student_count: number
}

export interface TopicsGetResponse {
  total: number
  page: number
  page_size: number
  topics: TopicBrief[]
}

export interface TopicsPostRequest {
  major_id: number
  topic_name: string
  topic_description: string
  topic_max_students: number
  topic_type: TopicType
}

export interface TopicsPostTeacherRequest {
  topic_name?: string
  topic_description?: string
  topic_max_students?: number
  topic_type?: TopicType
}

export interface TopicsPostOfficeRequest {
  topic_review_status: TopicReviewStatus
}

export interface TopicsPostAdminRequest {
  topic_name?: string
  topic_description?: string
  topic_max_students?: number
  topic_type?: TopicType
  topic_review_status?: TopicReviewStatus
}

export interface TopicCreateResponse {
  topic_id: number
}

// Assignment models
export interface Assignment {
  student_id: number
  student_name: string
  student_major: string
  topic_id: number
  topic_name: string
  request_time: string
  status: AssignmentStatus
}

export interface AssignmentsGetResponse {
  total: number
  page: number
  page_size: number
  assignments: Assignment[]
}

export interface AssignmentsPostRequest {
  topic_id: number
}

export interface AssignmentRecordPatchRequest {
  approved: boolean
}

// Progress Report models
export interface ProgressReportDetailResponse {
  prog_report_id: number
  topic_id: number
  student_id: number
  student_name: string
  prog_report_type: ProgressReportType
  prog_report_time: string
  prog_report_attachment: string
  prog_report_outcome: ProgressOutcome
  prog_report_comment?: string
  prog_report_grade?: number
}

export interface ProgressReportsGetResponse {
  reports: ProgressReportDetailResponse[]
}

export interface ProgressReportsPostRequest {
  attachment: string
}

export interface ProgressReportRecordPatchRequest {
  outcome: ProgressOutcome
  comment?: string
  grade?: number
}

// Final Defense models
export interface FinalDefenseDetails {
  final_def_id: number
  topic_id: number
  topic_name: string
  student_id: number
  student_name: string
  defense_board_id?: number
  final_def_time: string
  final_def_attachment: string
  final_def_outcome?: boolean
  final_def_comment?: string
  final_def_grade?: number
}

export interface FinalDefensesGetResponse {
  defenses: FinalDefenseDetails[]
}

export interface FinalDefensesPostRequest {
  attachment: string
}

export interface FinalDefensesRecordTeacherPatchRequest {
  approved: boolean
}

export interface FinalDefensesRecordDefenseBoardPatchRequest {
  outcome: boolean
  comment: string
  grade: number
}

// Pagination params (for convenience)
export interface PaginationParams {
  page?: number
  page_size?: number
}
