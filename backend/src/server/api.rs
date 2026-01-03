use actix_session::Session;
use actix_web::{HttpResponse, ResponseError, get, patch, post, web};
use backend_database::DbPool;
use backend_database::model::*;
use diesel::prelude::*;
use serde::Deserialize;
use str_macro::str;

use crate::helper::*;
use crate::map_schema_role;
use crate::model::*;

#[get("/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().json(str!("pong"))
}

#[post("/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, impl ResponseError> {
    use backend_database::schema;

    if is_session_authed(&session) {
        return Err(ApiError::BadRequest(str!("Already logged in")));
    }

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;
    let user = schema::sysuser::dsl::sysuser
        .filter(schema::sysuser::columns::user_login.eq(&req.username))
        .first::<SysUser>(&mut conn)
        .map_err(|_| ApiError::Unauthorized)?;

    let password_good = verify_password(
        &req.password,
        &user.user_password_hash,
        &user.user_password_salt,
    )
    .map_err(|_| ApiError::InternalServerError(str!("Failed to verify password")))?;

    if password_good {
        let auth_info = map_schema_role!(
            &mut conn, user.user_id, Err(ApiError::InternalServerError(str!("User not in any role"))),
            schema::sysadmin::dsl::sysadmin => SysAdmin => Ok(AuthInfo::SysAdmin {
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
                ApiError::InternalServerError(str!("Failed to store session information"))
            })?;

        Ok(HttpResponse::Ok().finish())
    } else {
        Err(ApiError::Unauthorized)
    }
}

#[post("/logout")]
pub async fn logout(session: Session) -> Result<HttpResponse, impl ResponseError> {
    if session.contains_key(AUTH_INFO_SESSION_KEY) {
        session.purge();
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(ApiError::Unauthorized)
    }
}

#[get("/user")]
pub async fn get_current_user(
    pool: web::Data<DbPool>,
    session: Session,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let user_id = get_session_user_id(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user ID from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let sys_user = schema::sysuser::dsl::sysuser
        .filter(schema::sysuser::columns::user_id.eq(user_id))
        .first::<SysUser>(&mut conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user information")))?;

    // Get user role and name based on role
    let (role, name) = map_schema_role!(
        &mut conn, user_id, Err(ApiError::InternalServerError(str!("User not in any role"))),
        schema::sysadmin::dsl::sysadmin => SysAdmin => {
            Ok((UserRole::Admin, None))
        };
        schema::student::dsl::student => Student => {
            let student = schema::student::dsl::student
                .filter(schema::student::columns::user_id.eq(user_id))
                .first::<Student>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to get student information")))?;
            Ok((UserRole::Student, Some(student.student_name)))
        };
        schema::teacher::dsl::teacher => Teacher => {
            let teacher = schema::teacher::dsl::teacher
                .filter(schema::teacher::columns::user_id.eq(user_id))
                .first::<Teacher>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to get teacher information")))?;
            Ok((UserRole::Teacher, Some(teacher.teacher_name)))
        };
        schema::defenseboard::dsl::defenseboard => DefenseBoard => {
            Ok((UserRole::DefenseBoard, None))
        };
        schema::office::dsl::office => Office => {
            Ok((UserRole::Office, None))
        };
    )?;

    Ok(HttpResponse::Ok().json(UserGetResponse {
        id: sys_user.user_id,
        username: sys_user.user_login,
        role,
        name,
        avatar: sys_user.user_avatar,
    }))
}

#[patch("/user")]
pub async fn update_current_user(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<UserPatchRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let user_id = get_session_user_id(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user ID from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    conn.build_transaction().read_write().run(|conn| {
        if let Some(ref password) = req.password {
            let (hash, salt) = hash_password(password)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to hash password")))?;

            diesel::update(
                schema::sysuser::dsl::sysuser.filter(schema::sysuser::columns::user_id.eq(user_id)),
            )
            .set((
                schema::sysuser::columns::user_password_hash.eq(hash),
                schema::sysuser::columns::user_password_salt.eq(salt),
            ))
            .execute(conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to update password")))?;
        }

        if let Some(ref avatar) = req.avatar {
            diesel::update(
                schema::sysuser::dsl::sysuser.filter(schema::sysuser::columns::user_id.eq(user_id)),
            )
            .set(schema::sysuser::columns::user_avatar.eq(avatar))
            .execute(conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to update avatar")))?;
        }

        if let Some(ref name) = req.name {
            // Is student?
            if diesel::select(diesel::dsl::exists(
                schema::student::dsl::student.filter(schema::student::columns::user_id.eq(user_id)),
            ))
            .get_result::<bool>(conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to check user role")))?
            {
                diesel::update(
                    schema::student::dsl::student
                        .filter(schema::student::columns::user_id.eq(user_id)),
                )
                .set(schema::student::columns::student_name.eq(name))
                .execute(conn)
                .map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to update student name"))
                })?;
            }
            // Is teacher?
            else if diesel::select(diesel::dsl::exists(
                schema::teacher::dsl::teacher.filter(schema::teacher::columns::user_id.eq(user_id)),
            ))
            .get_result::<bool>(conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to check user role")))?
            {
                diesel::update(
                    schema::teacher::dsl::teacher
                        .filter(schema::teacher::columns::user_id.eq(user_id)),
                )
                .set(schema::teacher::columns::teacher_name.eq(name))
                .execute(conn)
                .map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to update teacher name"))
                })?;
            }
        }

        Ok::<(), ApiError>(())
    })?;

    Ok(HttpResponse::Ok().finish())
}

#[post("/user")]
pub async fn create_user(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<UserPostRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }
    if !is_session_admin(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Can't deserialize auth info")))?
    {
        return Err(ApiError::Forbidden);
    }

    let (hash, salt) = hash_password(&req.password)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to hash password")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;
    let (new_sys_user, name) = conn.build_transaction().read_write().run(|conn| {
        let name = match req.role {
            UserRole::Student | UserRole::Teacher => req.name.clone(),
            _ => None,
        };

        let new_sys_user = diesel::insert_into(schema::sysuser::dsl::sysuser)
            .values(NewSysUser {
                user_login: &req.username,
                user_password_hash: &hash,
                user_password_salt: &salt,
                user_avatar: req.avatar.as_deref(),
            })
            .get_result::<SysUser>(conn)
            .map_err(|_| ApiError::Conflict(str!("Failed to create new user")))?;
        match req.role {
            UserRole::Admin => diesel::insert_into(schema::sysadmin::dsl::sysadmin)
                .values(NewSysAdmin {
                    user_id: new_sys_user.user_id,
                })
                .execute(conn),
            UserRole::Student => diesel::insert_into(schema::student::dsl::student)
                .values(NewStudent {
                    user_id: new_sys_user.user_id,
                    topic_id: None,
                    major_id: req.major_id.ok_or(ApiError::BadRequest(str!(
                        "Major ID is required for student role"
                    )))?,
                    student_name: name.as_deref().ok_or(ApiError::BadRequest(str!(
                        "Name is required for student role"
                    )))?,
                    assn_time: None,
                })
                .execute(conn),
            UserRole::Teacher => diesel::insert_into(schema::teacher::dsl::teacher)
                .values(NewTeacher {
                    user_id: new_sys_user.user_id,
                    teacher_name: name.as_deref().ok_or(ApiError::BadRequest(str!(
                        "Name is required for teacher role"
                    )))?,
                })
                .execute(conn),
            UserRole::DefenseBoard => diesel::insert_into(schema::defenseboard::dsl::defenseboard)
                .values(NewDefenseBoard {
                    user_id: new_sys_user.user_id,
                })
                .execute(conn),
            UserRole::Office => diesel::insert_into(schema::office::dsl::office)
                .values(NewOffice {
                    user_id: new_sys_user.user_id,
                })
                .execute(conn),
        }
        .map_err(|_| ApiError::InternalServerError(str!("Failed to assign role")))?;
        Ok::<_, ApiError>((new_sys_user, name))
    })?;

    Ok(HttpResponse::Ok().json(UserGetResponse {
        id: new_sys_user.user_id,
        username: new_sys_user.user_login,
        role: req.role,
        name,
        avatar: req.avatar.clone(),
    }))
}

#[get("/topics")]
pub async fn get_topics(
    _pool: web::Data<DbPool>,
    _session: Session,
    _query: web::Query<PaginationQuery>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/topics")]
pub async fn create_topic(
    _pool: web::Data<DbPool>,
    _session: Session,
    _req: web::Json<TopicsPostRequest>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[get("/topics/search")]
pub async fn search_topics(
    _pool: web::Data<DbPool>,
    _session: Session,
    _query: web::Query<SearchQuery>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[get("/topics/{topic_id}")]
pub async fn get_topic_detail(
    _pool: web::Data<DbPool>,
    _session: Session,
    _topic_id: web::Path<i32>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[patch("/topics/{topic_id}")]
pub async fn update_topic(
    _pool: web::Data<DbPool>,
    _session: Session,
    _topic_id: web::Path<i32>,
    _req: web::Json<TopicUpdateRequest>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[get("/assignments")]
pub async fn get_assignments(
    _pool: web::Data<DbPool>,
    _session: Session,
    _query: web::Query<PaginationQuery>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/assignments")]
pub async fn create_assignment(
    _pool: web::Data<DbPool>,
    _session: Session,
    _req: web::Json<AssignmentsPostRequest>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[patch("/assignments/{student_id}/{topic_id}")]
pub async fn update_assignment_status(
    _pool: web::Data<DbPool>,
    _session: Session,
    _path: web::Path<(i32, i32)>,
    _req: web::Json<AssignmentRecordPatchRequest>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[get("/progress_reports")]
pub async fn get_progress_reports(_pool: web::Data<DbPool>, _session: Session) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/progress_reports")]
pub async fn create_progress_report(
    _pool: web::Data<DbPool>,
    _session: Session,
    _req: web::Json<ProgressReportsPostRequest>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[patch("/progress_reports/{report_id}")]
pub async fn update_progress_report(
    _pool: web::Data<DbPool>,
    _session: Session,
    _report_id: web::Path<i32>,
    _req: web::Json<ProgressReportRecordPatchRequest>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[get("/final_defenses")]
pub async fn get_final_defenses(_pool: web::Data<DbPool>, _session: Session) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/final_defenses")]
pub async fn create_final_defense(
    _pool: web::Data<DbPool>,
    _session: Session,
    _req: web::Json<FinalDefensesPostRequest>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[patch("/final_defenses/{report_id}")]
pub async fn update_final_defense(
    _pool: web::Data<DbPool>,
    _session: Session,
    _report_id: web::Path<i32>,
    _req: web::Json<FinalDefensesRecordPatchRequest>,
) -> HttpResponse {
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
