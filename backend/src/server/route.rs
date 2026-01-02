use actix_session::Session;
use actix_web::{HttpResponse, Responder, ResponseError, get, patch, post, web};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use backend_database::DbPool;
use backend_database::model::*;
use diesel::prelude::*;
use serde::Deserialize;

use crate::model::*;

#[get("/api/ping")]
pub async fn ping() -> impl Responder {
    HttpResponse::Ok().json("pong".to_string())
}

macro_rules! for_each_role {
    ($conn:expr, $needle:expr, $default:expr $(,)?) => {
        $default
    };

    ($conn:expr, $needle:expr, $default:expr,
     $table_dsl:path => $table_struct:ty => $rhs:expr;
     $($rest:tt)*
    ) => {
        if $table_dsl.find($needle).first::<$table_struct>($conn).is_ok() {
            $rhs
        } else {
            for_each_role!($conn, $needle, $default, $($rest)*)
        }
    };
}

#[post("/api/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<LoginRequest>,
) -> Result<impl Responder, impl ResponseError> {
    use backend_database::schema;

    let mut conn = pool.get().map_err(|_| {
        ApiError::InternalServerError("Failed to get database connection".to_string())
    })?;
    let user = schema::sysuser::dsl::sysuser
        .filter(schema::sysuser::columns::user_login.eq(&req.username))
        .first::<SysUser>(&mut conn)
        .map_err(|_| ApiError::Unauthorized)?;

    let salt = SaltString::encode_b64(&user.user_password_salt)
        .map_err(|_| ApiError::InternalServerError("Failed to encode password salt".to_string()))?;
    let hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| ApiError::InternalServerError("Failed to hash password".to_string()))?
        .hash
        .ok_or(ApiError::InternalServerError(
            "Failed to get password hash".to_string(),
        ))?;

    if hash.as_bytes() == user.user_password_hash.as_slice() {
        let auth_info = for_each_role!(
            &mut conn, user.user_id, Err(ApiError::InternalServerError("User not in any role".to_string())),
            schema::sysadmin::dsl::sysadmin  => SysAdmin => Ok(AuthInfo::SysAdmin {
                user_id: user.user_id,
                username: user.user_login.clone(),
                impersonating: None,
            });
            schema::student::dsl::student => Student => Ok(AuthInfo::User {
                user_id: user.user_id,
                username: user.user_login.clone(),
                role: AuthInfoUserRole::Student,
            });
            schema::teacher::dsl::teacher => Teacher => Ok(AuthInfo::User {
                user_id: user.user_id,
                username: user.user_login.clone(),
                role: AuthInfoUserRole::Teacher,
            });
            schema::defenseboard::dsl::defenseboard => DefenseBoard => Ok(AuthInfo::User {
                user_id: user.user_id,
                username: user.user_login.clone(),
                role: AuthInfoUserRole::DefenseBoard,
            });
            schema::office::dsl::office => Office => Ok(AuthInfo::User {
                user_id: user.user_id,
                username: user.user_login.clone(),
                role: AuthInfoUserRole::Office,
            });
        )?;

        session
            .insert(AUTH_INFO_SESSION_KEY, &auth_info)
            .map_err(|_| {
                ApiError::InternalServerError("Failed to store session information".to_string())
            })?;

        Ok(HttpResponse::Ok().finish())
    } else {
        Err(ApiError::Unauthorized)
    }
}

#[post("/api/logout")]
pub async fn logout(session: Session) -> impl Responder {
    if session.contains_key(AUTH_INFO_SESSION_KEY) {
        session.purge();
        HttpResponse::Ok().finish()
    } else {
        ApiError::Unauthorized.error_response()
    }
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
