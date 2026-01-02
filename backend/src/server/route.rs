use actix_session::Session;
use actix_web::{HttpResponse, Responder, get, patch, post, web};
use backend_database::DbPool;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde::Deserialize;

use crate::model::*;

#[get("/api/ping")]
pub async fn ping() -> impl Responder {
    HttpResponse::Ok().json("pong".to_string())
}

#[post("/api/login")]
pub async fn login(
    _pool: web::Data<DbPool>,
    _session: Session,
    _req: web::Json<LoginRequest>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[post("/api/logout")]
pub async fn logout(_session: Session) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/api/user")]
pub async fn get_current_user(_pool: web::Data<DbPool>, _session: Session) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[patch("/api/user")]
pub async fn update_current_user(
    _pool: web::Data<DbPool>,
    _session: Session,
    _req: web::Json<UserPatchRequest>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[post("/api/user")]
pub async fn create_user(
    _pool: web::Data<DbPool>,
    _session: Session,
    _req: web::Json<UserPostRequest>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/api/topics")]
pub async fn get_topics(
    _pool: web::Data<DbPool>,
    _session: Session,
    _query: web::Query<PaginationQuery>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[post("/api/topics")]
pub async fn create_topic(
    _pool: web::Data<DbPool>,
    _session: Session,
    _req: web::Json<TopicsPostRequest>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/api/topics/search")]
pub async fn search_topics(
    _pool: web::Data<DbPool>,
    _session: Session,
    _query: web::Query<SearchQuery>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/api/topics/{topic_id}")]
pub async fn get_topic_detail(
    _pool: web::Data<DbPool>,
    _session: Session,
    _topic_id: web::Path<i32>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[patch("/api/topics/{topic_id}")]
pub async fn update_topic(
    _pool: web::Data<DbPool>,
    _session: Session,
    _topic_id: web::Path<i32>,
    _req: web::Json<TopicUpdateRequest>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/api/assignments")]
pub async fn get_assignments(
    _pool: web::Data<DbPool>,
    _session: Session,
    _query: web::Query<PaginationQuery>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[post("/api/assignments")]
pub async fn create_assignment(
    _pool: web::Data<DbPool>,
    _session: Session,
    _req: web::Json<AssignmentsPostRequest>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[patch("/api/assignments/{student_id}/{topic_id}")]
pub async fn update_assignment_status(
    _pool: web::Data<DbPool>,
    _session: Session,
    _path: web::Path<(i32, i32)>,
    _req: web::Json<AssignmentRecordPatchRequest>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/api/progress_reports")]
pub async fn get_progress_reports(_pool: web::Data<DbPool>, _session: Session) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[post("/api/progress_reports")]
pub async fn create_progress_report(
    _pool: web::Data<DbPool>,
    _session: Session,
    _req: web::Json<ProgressReportsPostRequest>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[patch("/api/progress_reports/{report_id}")]
pub async fn update_progress_report(
    _pool: web::Data<DbPool>,
    _session: Session,
    _report_id: web::Path<i32>,
    _req: web::Json<ProgressReportRecordPatchRequest>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/api/final_defenses")]
pub async fn get_final_defenses(_pool: web::Data<DbPool>, _session: Session) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[post("/api/final_defenses")]
pub async fn create_final_defense(
    _pool: web::Data<DbPool>,
    _session: Session,
    _req: web::Json<FinalDefensesPostRequest>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[patch("/api/final_defenses/{report_id}")]
pub async fn update_final_defense(
    _pool: web::Data<DbPool>,
    _session: Session,
    _report_id: web::Path<i32>,
    _req: web::Json<FinalDefensesRecordPatchRequest>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    pub keyword: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TopicUpdateRequest {
    Teacher(TopicsPostTeacherRequest),
    Office(TopicsPostOfficeRequest),
    Admin(TopicsPostAdminRequest),
}
