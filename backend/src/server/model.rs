use actix_web::{HttpResponse, ResponseError};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    Student,
    Teacher,
    DefenseBoard,
    Office,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicType {
    /// 0: 理论研究型
    Theoretical,
    /// 1: 应用开发型
    Applied,
    /// 2: 实验研究型
    Experimental,
    /// 3: 工程设计型
    Engineering,
    /// 4: 其他
    Other,
}

impl From<TopicType> for i16 {
    fn from(value: TopicType) -> Self {
        match value {
            TopicType::Theoretical => 0,
            TopicType::Applied => 1,
            TopicType::Experimental => 2,
            TopicType::Engineering => 3,
            TopicType::Other => 4,
        }
    }
}

impl TryFrom<i16> for TopicType {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TopicType::Theoretical),
            1 => Ok(TopicType::Applied),
            2 => Ok(TopicType::Experimental),
            3 => Ok(TopicType::Engineering),
            4 => Ok(TopicType::Other),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicReviewStatus {
    /// 0: 待审核
    Pending,
    /// 1: 已通过
    Approved,
    /// 2: 已拒绝
    Rejected,
}

impl From<TopicReviewStatus> for i16 {
    fn from(value: TopicReviewStatus) -> Self {
        match value {
            TopicReviewStatus::Pending => 0,
            TopicReviewStatus::Approved => 1,
            TopicReviewStatus::Rejected => 2,
        }
    }
}

impl TryFrom<i16> for TopicReviewStatus {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TopicReviewStatus::Pending),
            1 => Ok(TopicReviewStatus::Approved),
            2 => Ok(TopicReviewStatus::Rejected),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressReportType {
    /// 0: 开题报告
    Proposal,
    /// 1: 中期检查
    Midterm,
}

impl From<ProgressReportType> for i16 {
    fn from(value: ProgressReportType) -> Self {
        match value {
            ProgressReportType::Proposal => 0,
            ProgressReportType::Midterm => 1,
        }
    }
}

impl TryFrom<i16> for ProgressReportType {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ProgressReportType::Proposal),
            1 => Ok(ProgressReportType::Midterm),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressOutcome {
    /// 0: 无结论
    NoConclusion,
    /// 1: 已通过
    Passed,
    /// 2: 已打回
    Rejected,
}

impl From<ProgressOutcome> for i16 {
    fn from(value: ProgressOutcome) -> Self {
        match value {
            ProgressOutcome::NoConclusion => 0,
            ProgressOutcome::Passed => 1,
            ProgressOutcome::Rejected => 2,
        }
    }
}

impl TryFrom<i16> for ProgressOutcome {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ProgressOutcome::NoConclusion),
            1 => Ok(ProgressOutcome::Passed),
            2 => Ok(ProgressOutcome::Rejected),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentStatus {
    /// 0: 待审核
    Pending,
    /// 1: 已通过
    Approved,
    /// 2: 已拒绝
    Rejected,
}

impl From<AssignmentStatus> for i16 {
    fn from(value: AssignmentStatus) -> Self {
        match value {
            AssignmentStatus::Pending => 0,
            AssignmentStatus::Approved => 1,
            AssignmentStatus::Rejected => 2,
        }
    }
}

impl TryFrom<i16> for AssignmentStatus {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AssignmentStatus::Pending),
            1 => Ok(AssignmentStatus::Approved),
            2 => Ok(AssignmentStatus::Rejected),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGetResponse {
    pub id: i32,
    pub username: String,
    pub role: UserRole,
    pub name: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPatchRequest {
    pub name: Option<String>,
    pub password: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPostRequest {
    pub username: String,
    pub password: String,
    pub role: UserRole,
    pub name: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicBrief {
    pub topic_id: i32,
    pub teacher_name: String,
    pub topic_name: String,
    pub topic_max_students: i32,
    pub topic_type: TopicType,
    pub current_student_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicDetails {
    pub topic_id: i32,
    pub major_id: i32,
    pub major_name: String,
    pub teacher_id: i32,
    pub teacher_name: String,
    pub topic_name: String,
    pub topic_description: String,
    pub topic_max_students: i32,
    pub topic_type: TopicType,
    pub topic_review_status: TopicReviewStatus,
    pub current_student_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicsGetResponse {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub topics: Vec<TopicBrief>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicsPostRequest {
    pub major_id: i32,
    pub topic_name: String,
    pub topic_description: String,
    pub topic_max_students: i32,
    pub topic_type: TopicType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicsPostTeacherRequest {
    pub topic_name: Option<String>,
    pub topic_description: Option<String>,
    pub topic_max_students: Option<i32>,
    pub topic_type: Option<TopicType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicsPostOfficeRequest {
    pub topic_review_status: TopicReviewStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicsPostAdminRequest {
    pub topic_name: Option<String>,
    pub topic_description: Option<String>,
    pub topic_max_students: Option<i32>,
    pub topic_type: Option<TopicType>,
    pub topic_review_status: Option<TopicReviewStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCreateResponse {
    pub topic_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub student_id: i32,
    pub student_name: String,
    pub student_major: String,
    pub topic_id: i32,
    pub topic_name: String,
    pub request_time: DateTime<Utc>,
    pub status: AssignmentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentsGetResponse {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub assignments: Vec<Assignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentsPostRequest {
    pub topic_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentRecordPatchRequest {
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReportDetailResponse {
    pub prog_report_id: i32,
    pub topic_id: i32,
    pub student_id: i32,
    pub student_name: String,
    pub prog_report_type: ProgressReportType,
    pub prog_report_time: DateTime<Utc>,
    pub prog_report_attachment: String,
    pub prog_report_outcome: ProgressOutcome,
    pub prog_report_comment: Option<String>,
    pub prog_report_grade: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReportsGetResponse {
    pub reports: Vec<ProgressReportDetailResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReportsPostRequest {
    pub attachment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReportRecordPatchRequest {
    pub outcome: ProgressOutcome,
    pub comment: Option<String>,
    pub grade: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalDefenseDetails {
    pub final_def_id: i32,
    pub topic_id: i32,
    pub topic_name: String,
    pub student_id: i32,
    pub student_name: String,
    pub defense_board_id: i32,
    pub final_def_time: DateTime<Utc>,
    pub final_def_attachment: String,
    pub final_def_outcome: Option<bool>,
    pub final_def_comment: Option<String>,
    pub final_def_grade: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalDefensesGetResponse {
    pub defenses: Vec<FinalDefenseDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalDefensesPostRequest {
    pub attachment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalDefensesRecordPatchRequest {
    pub outcome: bool,
    pub comment: String,
    pub grade: BigDecimal,
}

#[derive(Debug)]
pub enum ApiError {
    Unauthorized,
    Forbidden,
    NotFound,
    BadRequest(String),
    Conflict(String),
    InternalServerError,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Unauthorized => write!(f, "未登录"),
            ApiError::Forbidden => write!(f, "权限不足"),
            ApiError::NotFound => write!(f, "资源未找到"),
            ApiError::BadRequest(msg) => write!(f, "请求格式错误: {}", msg),
            ApiError::Conflict(msg) => write!(f, "资源冲突: {}", msg),
            ApiError::InternalServerError => write!(f, "服务器内部错误"),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::Unauthorized => HttpResponse::Unauthorized().json(ErrorResponse {
                message: self.to_string(),
            }),
            ApiError::Forbidden => HttpResponse::Forbidden().json(ErrorResponse {
                message: self.to_string(),
            }),
            ApiError::NotFound => HttpResponse::NotFound().json(ErrorResponse {
                message: self.to_string(),
            }),
            ApiError::BadRequest(_) => HttpResponse::BadRequest().json(ErrorResponse {
                message: self.to_string(),
            }),
            ApiError::Conflict(_) => HttpResponse::Conflict().json(ErrorResponse {
                message: self.to_string(),
            }),
            ApiError::InternalServerError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: self.to_string(),
                })
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: i32,
    pub role: UserRole,
    pub username: String,
}
