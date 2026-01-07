use actix_session::Session;
use actix_web::{HttpResponse, get, patch, post, web};
use backend_database::DbPool;
use backend_database::model::*;
use chrono::Utc;
use diesel::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use str_macro::str;

use crate::auth::*;
use crate::dto::*;
use crate::map_schema_role;

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

#[get("/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().json(str!("pong"))
}

#[post("/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if is_session_authed(&session) {
        return Err(ApiError::BadRequest(str!("Already logged in")));
    }

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;
    let user = sysuser::dsl::sysuser
        .filter(sysuser::columns::user_name.eq(&req.username))
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
            &mut conn, &user.user_name, Err(ApiError::InternalServerError(str!("User not in any role"))),
            sysadmin::dsl::sysadmin => SysAdmin => Ok(AuthInfo::SysAdmin {
                username: user.user_name,
                impersonating: None,
            });
            student::dsl::student => Student => Ok(AuthInfo::User {
                username: user.user_name,
                role: AuthInfoUserRole::Student,
            });
            teacher::dsl::teacher => Teacher => Ok(AuthInfo::User {
                username: user.user_name,
                role: AuthInfoUserRole::Teacher,
            });
            defenseboard::dsl::defenseboard => DefenseBoard => Ok(AuthInfo::User {
                username: user.user_name,
                role: AuthInfoUserRole::DefenseBoard,
            });
            office::dsl::office => Office => Ok(AuthInfo::User {
                username: user.user_name,
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
pub async fn logout(session: Session) -> Result<HttpResponse, ApiError> {
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
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let sys_user = sysuser::dsl::sysuser
        .find(&username)
        .first::<SysUser>(&mut conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user information")))?;

    // Get user role and name based on role
    let (role, name) = map_schema_role!(
        &mut conn, &username, Err(ApiError::InternalServerError(str!("User not in any role"))),
        sysadmin::dsl::sysadmin => SysAdmin => {
            Ok((UserRole::Admin, None))
        };
        student::dsl::student => Student => {
            let student = student::dsl::student
                .find(&username)
                .first::<Student>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to get student information")))?;
            Ok((UserRole::Student, Some(student.student_name)))
        };
        teacher::dsl::teacher => Teacher => {
            let teacher = teacher::dsl::teacher
                .find(&username)
                .first::<Teacher>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to get teacher information")))?;
            Ok((UserRole::Teacher, Some(teacher.teacher_name)))
        };
        defenseboard::dsl::defenseboard => DefenseBoard => {
            Ok((UserRole::DefenseBoard, None))
        };
        office::dsl::office => Office => {
            Ok((UserRole::Office, None))
        };
    )?;

    Ok(HttpResponse::Ok().json(UserGetResponse {
        username: sys_user.user_name.clone(),
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
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    conn.build_transaction().read_write().run(|conn| {
        if let Some(ref password) = req.password {
            let (hash, salt) = hash_password(password)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to hash password")))?;

            diesel::update(sysuser::dsl::sysuser.find(&username))
                .set(SysUserPasswordChangeset {
                    user_password_hash: Some(hash),
                    user_password_salt: Some(salt),
                })
                .execute(conn)
                .map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to update user information"))
                })?;
        }

        if let Some(ref avatar) = req.avatar {
            diesel::update(sysuser::dsl::sysuser.find(&username))
                .set(sysuser::columns::user_avatar.eq(avatar))
                .execute(conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to update avatar")))?;
        }

        if let Some(ref name) = req.name {
            // Is student?
            if diesel::select(diesel::dsl::exists(student::dsl::student.find(&username)))
                .get_result::<bool>(conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to check user role")))?
            {
                diesel::update(student::dsl::student.find(&username))
                    .set(student::columns::student_name.eq(name))
                    .execute(conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to update student name"))
                    })?;
            }
            // Is teacher?
            else if diesel::select(diesel::dsl::exists(teacher::dsl::teacher.find(&username)))
                .get_result::<bool>(conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to check user role")))?
            {
                diesel::update(teacher::dsl::teacher.find(&username))
                    .set(teacher::columns::teacher_name.eq(name))
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
    use backend_database::schema::*;

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

        let new_sys_user = diesel::insert_into(sysuser::dsl::sysuser)
            .values(NewSysUser {
                user_name: &req.username,
                user_password_hash: &hash,
                user_password_salt: &salt,
                user_avatar: req.avatar.as_deref(),
            })
            .get_result::<SysUser>(conn)
            .map_err(|_| ApiError::Conflict(str!("Failed to create new user")))?;
        match req.role {
            UserRole::Admin => diesel::insert_into(sysadmin::dsl::sysadmin)
                .values(NewSysAdmin {
                    user_name: &new_sys_user.user_name,
                })
                .execute(conn),
            UserRole::Student => diesel::insert_into(student::dsl::student)
                .values(NewStudent {
                    user_name: &new_sys_user.user_name,
                    topic_id: None,
                    major_id: req.major_id.ok_or(ApiError::BadRequest(str!(
                        "Major ID is required for student role"
                    )))?,
                    student_name: name.as_deref().ok_or(ApiError::BadRequest(str!(
                        "Name is required for student role"
                    )))?,
                    assn_time: Utc::now(),
                })
                .execute(conn),
            UserRole::Teacher => diesel::insert_into(teacher::dsl::teacher)
                .values(NewTeacher {
                    user_name: &new_sys_user.user_name,
                    teacher_name: name.as_deref().ok_or(ApiError::BadRequest(str!(
                        "Name is required for teacher role"
                    )))?,
                })
                .execute(conn),
            UserRole::DefenseBoard => diesel::insert_into(defenseboard::dsl::defenseboard)
                .values(NewDefenseBoard {
                    user_name: &new_sys_user.user_name,
                })
                .execute(conn),
            UserRole::Office => diesel::insert_into(office::dsl::office)
                .values(NewOffice {
                    user_name: &new_sys_user.user_name,
                })
                .execute(conn),
        }
        .map_err(|_| ApiError::InternalServerError(str!("Failed to assign role")))?;
        Ok::<_, ApiError>((new_sys_user, name))
    })?;

    Ok(HttpResponse::Ok().json(UserGetResponse {
        username: new_sys_user.user_name,
        role: req.role,
        name,
        avatar: req.avatar.clone(),
    }))
}

#[get("/topics")]
pub async fn get_topics(
    pool: web::Data<DbPool>,
    session: Session,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    // FIXME: Negative checks
    let offset = (page - 1) * page_size;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let (total, topics_with_teacher) = match user_role {
        AuthInfoUserRole::Student => {
            // Student: Get topics approved for their major
            let student_info = student::dsl::student
                .find(&username)
                .first::<Student>(&mut conn)
                .map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to get student information"))
                })?;

            let query = topic::table
                .inner_join(teacher::table)
                .filter(topic::columns::major_id.eq(student_info.major_id))
                .filter(topic::columns::topic_review_status.eq(TopicReviewStatus::Approved as i16));
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
        AuthInfoUserRole::Teacher => {
            // Teacher: Get their own topics
            let query = topic::table
                .inner_join(teacher::table)
                .filter(topic::columns::teacher_user_name.eq(&username));
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
        AuthInfoUserRole::Office | AuthInfoUserRole::DefenseBoard => {
            // Office and DefenseBoard: Get all topics
            // No additional filtering needed.
            let query = topic::table.inner_join(teacher::table);
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
    };

    let mut topic_briefs = Vec::new();
    for (topic, teacher) in topics_with_teacher {
        let current_student_count = student::dsl::student
            .filter(student::columns::topic_id.eq(topic.topic_id))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|_| {
                ApiError::InternalServerError(str!("Failed to count students for topic"))
            })?;

        topic_briefs.push(TopicBrief {
            topic_id: topic.topic_id,
            teacher_name: teacher.teacher_name,
            topic_name: topic.topic_name,
            topic_max_students: topic.topic_max_students,
            topic_type: TopicType::try_from(topic.topic_type)
                .map_err(|_| ApiError::InternalServerError(str!("Invalid topic type")))?,
            topic_review_status: TopicReviewStatus::try_from(topic.topic_review_status)
                .map_err(|_| ApiError::InternalServerError(str!("Invalid topic review status")))?,
            current_student_count: current_student_count as i32,
        });
    }

    Ok(HttpResponse::Ok().json(TopicsGetResponse {
        total,
        page,
        page_size,
        topics: topic_briefs,
    }))
}

#[post("/topics")]
pub async fn create_topic(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<TopicsPostRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;
    if !matches!(user_role, AuthInfoUserRole::Teacher) {
        // Only teachers can create topics
        return Err(ApiError::Forbidden);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let has_such_teacher: i64 = teacher::dsl::teacher
        .find(&username)
        .count()
        .get_result(&mut conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to check teacher existence")))?;
    if has_such_teacher == 0 {
        return Err(ApiError::InternalServerError(str!(
            "Teacher record not found for user"
        )));
    }

    let new_topic = conn.build_transaction().read_write().run(|conn| {
        let new_topic = NewTopic {
            major_id: req.major_id,
            teacher_user_name: &username,
            topic_name: &req.topic_name,
            topic_description: &req.topic_description,
            topic_max_students: req.topic_max_students,
            topic_type: req.topic_type as i16,
            topic_review_status: TopicReviewStatus::Pending as i16,
        };

        let inserted_topic = diesel::insert_into(topic::dsl::topic)
            .values(&new_topic)
            .get_result::<Topic>(conn)
            .map_err(|e| {
                if e.to_string().contains("duplicate key") {
                    ApiError::Conflict(str!("Topic with similar name already exists"))
                } else {
                    ApiError::InternalServerError(str!("Failed to create topic"))
                }
            })?;

        Ok::<_, ApiError>(inserted_topic)
    })?;

    Ok(HttpResponse::Created().json(TopicCreateResponse {
        topic_id: new_topic.topic_id,
    }))
}

#[get("/topics/search")]
pub async fn search_topics(
    pool: web::Data<DbPool>,
    session: Session,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    // FIXME: Negative checks
    let offset = (page - 1) * page_size;
    let search_pattern = format!("%{}%", query.keyword.as_deref().unwrap_or(""));

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let (total, topics_with_teacher) = match user_role {
        AuthInfoUserRole::Student => {
            // Student: Get topics approved for their major with keyword search
            let student_info = student::dsl::student
                .find(&username)
                .first::<Student>(&mut conn)
                .map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to get student information"))
                })?;

            let query = topic::table
                .inner_join(teacher::table)
                .filter(topic::columns::major_id.eq(student_info.major_id))
                .filter(topic::columns::topic_review_status.eq(TopicReviewStatus::Approved as i16))
                .filter(topic::columns::topic_name.like(&search_pattern));
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
        AuthInfoUserRole::Teacher => {
            // Teacher: Get their own topics with keyword search
            let query = topic::table
                .inner_join(teacher::table)
                .filter(topic::columns::teacher_user_name.eq(&username))
                .filter(topic::columns::topic_name.like(&search_pattern));
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
        AuthInfoUserRole::Office | AuthInfoUserRole::DefenseBoard => {
            // Office and DefenseBoard: Get all topics with keyword search
            let query = topic::table
                .inner_join(teacher::table)
                .filter(topic::columns::topic_name.like(&search_pattern));
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
    };

    let mut topic_briefs = Vec::new();
    for (topic, teacher) in topics_with_teacher {
        let current_student_count = student::dsl::student
            .filter(student::columns::topic_id.eq(topic.topic_id))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|_| {
                ApiError::InternalServerError(str!("Failed to count students for topic"))
            })?;
        topic_briefs.push(TopicBrief {
            topic_id: topic.topic_id,
            teacher_name: teacher.teacher_name,
            topic_name: topic.topic_name,
            topic_max_students: topic.topic_max_students,
            topic_type: TopicType::try_from(topic.topic_type)
                .map_err(|_| ApiError::InternalServerError(str!("Invalid topic type")))?,
            topic_review_status: TopicReviewStatus::try_from(topic.topic_review_status)
                .map_err(|_| ApiError::InternalServerError(str!("Invalid topic review status")))?,
            current_student_count: current_student_count as i32,
        });
    }

    Ok(HttpResponse::Ok().json(TopicsGetResponse {
        total,
        page,
        page_size,
        topics: topic_briefs,
    }))
}

#[get("/topics/{topic_id}")]
pub async fn get_topic_detail(
    pool: web::Data<DbPool>,
    session: Session,
    topic_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    // Why boxed query here? Since we don't need it to be Copy, unlike the previous (count, records) pattern.
    let mut query_builder = topic::table
        .inner_join(teacher::table)
        .inner_join(major::table)
        .filter(topic::columns::topic_id.eq(*topic_id))
        .into_boxed();

    match user_role {
        AuthInfoUserRole::Student => {
            // Student: Get topics approved for their major
            let student_info = student::dsl::student
                .find(&username)
                .first::<Student>(&mut conn)
                .map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to get student information"))
                })?;

            query_builder = query_builder
                .filter(topic::columns::major_id.eq(student_info.major_id))
                .filter(topic::columns::topic_review_status.eq(TopicReviewStatus::Approved as i16));
        }
        AuthInfoUserRole::Teacher => {
            // Teacher: Get topics created by themselves
            query_builder = query_builder.filter(topic::columns::teacher_user_name.eq(&username));
        }
        AuthInfoUserRole::Office | AuthInfoUserRole::DefenseBoard => {
            // Office and Defense Board: Get all topics without additional filtering
        }
    }

    let (topic, teacher, major): (Topic, Teacher, Major) =
        query_builder.first(&mut conn).map_err(|e| {
            if let diesel::result::Error::NotFound = e {
                ApiError::NotFound
            } else {
                ApiError::InternalServerError(str!("Failed to load topic details"))
            }
        })?;

    let current_student_count: i64 = student::dsl::student
        .filter(student::columns::topic_id.eq(topic.topic_id))
        .count()
        .get_result(&mut conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to count students for topic")))?;

    // Build response
    let topic_details = TopicDetails {
        topic_id: topic.topic_id,
        major_id: topic.major_id,
        major_name: major.major_name,
        teacher_user_name: topic.teacher_user_name,
        teacher_name: teacher.teacher_name,
        topic_name: topic.topic_name,
        topic_description: topic.topic_description,
        topic_max_students: topic.topic_max_students,
        topic_type: TopicType::try_from(topic.topic_type)
            .map_err(|_| ApiError::InternalServerError(str!("Invalid topic type")))?,
        topic_review_status: TopicReviewStatus::try_from(topic.topic_review_status)
            .map_err(|_| ApiError::InternalServerError(str!("Invalid topic review status")))?,
        current_student_count: current_student_count as i32,
    };

    Ok(HttpResponse::Ok().json(topic_details))
}

#[patch("/topics/{topic_id}")]
pub async fn update_topic(
    pool: web::Data<DbPool>,
    session: Session,
    topic_id: web::Path<i32>,
    req: web::Json<TopicPatchRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let result = conn.build_transaction().read_write().run(|conn| {
        let topic = topic::dsl::topic
            .find(*topic_id)
            .first::<Topic>(conn)
            .map_err(|e| {
                if let diesel::result::Error::NotFound = e {
                    ApiError::NotFound
                } else {
                    ApiError::InternalServerError(str!("Failed to load topic"))
                }
            })?;

        let topic = match user_role {
            AuthInfoUserRole::Teacher => {
                // Is a teacher!
                if topic.teacher_user_name != username {
                    // Teacher can only update topics they created
                    return Err(ApiError::Forbidden);
                }

                if req.topic_review_status.is_some() {
                    // Teacher can not update review status
                    return Err(ApiError::BadRequest(str!(
                        "Teachers cannot update review status"
                    )));
                }

                let changeset = TopicChangeset {
                    topic_name: req.topic_name.clone(),
                    topic_description: req.topic_description.clone(),
                    topic_max_students: req.topic_max_students,
                    topic_type: req.topic_type.map(|t| t as i16),
                    // topic_review_status is not settable by teacher, but will be reset to Pending on any update
                    topic_review_status: Some(TopicReviewStatus::Pending as i16),
                };

                diesel::update(&topic)
                    .set(changeset)
                    .get_result::<Topic>(conn)
                    .map_err(|e| {
                        ApiError::InternalServerError(format!("Failed to update topic: {}", e))
                    })?
            }
            AuthInfoUserRole::Office => {
                // Is an office user!
                // Office can only update review status
                if req.topic_name.is_some()
                    || req.topic_description.is_some()
                    || req.topic_max_students.is_some()
                    || req.topic_type.is_some()
                {
                    return Err(ApiError::BadRequest(str!(
                        "Office can only update review status"
                    )));
                }

                if let Some(topic_review_status) = req.topic_review_status {
                    if topic_review_status == TopicReviewStatus::Pending {
                        return Err(ApiError::BadRequest(str!(
                            "Cannot explicitly set review status to Pending"
                        )));
                    }

                    diesel::update(&topic)
                        .set(topic::columns::topic_review_status.eq(topic_review_status as i16))
                        .get_result::<Topic>(conn)
                        .map_err(|e| {
                            ApiError::InternalServerError(format!(
                                "Failed to update topic review status: {}",
                                e
                            ))
                        })?
                } else {
                    return Err(ApiError::BadRequest(str!(
                        "Review status must be provided by Office"
                    )));
                }
            }
            AuthInfoUserRole::DefenseBoard | AuthInfoUserRole::Student => {
                // These roles are not allowed to update topics
                return Err(ApiError::Forbidden);
            }
        };

        let teacher = teacher::dsl::teacher
            .find(&topic.teacher_user_name)
            .first::<Teacher>(conn)
            .map_err(|e| {
                ApiError::InternalServerError(format!("Failed to load topic teacher: {}", e))
            })?;
        let major = major::dsl::major
            .find(topic.major_id)
            .first::<Major>(conn)
            .map_err(|e| {
                ApiError::InternalServerError(format!("Failed to load topic major: {}", e))
            })?;

        let current_student_count: i64 = student::dsl::student
            .filter(student::columns::topic_id.eq(topic.topic_id))
            .count()
            .get_result(conn)
            .map_err(|_| {
                ApiError::InternalServerError(str!("Failed to count students for topic"))
            })?;

        let topic_details = TopicDetails {
            topic_id: topic.topic_id,
            major_id: topic.major_id,
            major_name: major.major_name,
            teacher_user_name: topic.teacher_user_name,
            teacher_name: teacher.teacher_name,
            topic_name: topic.topic_name,
            topic_description: topic.topic_description,
            topic_max_students: topic.topic_max_students,
            topic_type: TopicType::try_from(topic.topic_type)
                .map_err(|_| ApiError::InternalServerError(str!("Invalid topic type")))?,
            topic_review_status: TopicReviewStatus::try_from(topic.topic_review_status)
                .map_err(|_| ApiError::InternalServerError(str!("Invalid topic review status")))?,
            current_student_count: current_student_count as i32,
        };

        Ok::<_, ApiError>(topic_details)
    })?;

    Ok(HttpResponse::Ok().json(result))
}

#[get("/assignments")]
pub async fn get_assignments(
    pool: web::Data<DbPool>,
    session: Session,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let is_admin = is_session_admin(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Can't deserialize auth info")))?;

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;
    let user_role = if is_admin {
        None
    } else {
        Some(get_session_user_role(&session).map_err(|_| {
            ApiError::InternalServerError(str!("Failed to get user role from session"))
        })?)
    };

    if !is_admin {
        match user_role {
            Some(AuthInfoUserRole::Student | AuthInfoUserRole::Teacher) => {}
            _ => return Err(ApiError::Forbidden),
        }
    }

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    // FIXME: Negative checks
    let offset = (page - 1) * page_size;
    let want = offset + page_size;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let pending_base = assignmentrequest::table
        .inner_join(student::table.inner_join(major::table))
        .inner_join(topic::table);

    let approved_base = student::table
        .inner_join(major::table)
        .inner_join(topic::table)
        .filter(student::columns::topic_id.is_not_null());

    let (pending_total, pending_rows, approved_total, approved_rows) = if is_admin {
        let pending_total: i64 = pending_base
            .count()
            .get_result(&mut conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to count assignments")))?;
        let pending_rows = pending_base
            .order(assignmentrequest::columns::assn_req_time.desc())
            .limit(want)
            .select((
                assignmentrequest::columns::student_user_name,
                student::columns::student_name,
                major::columns::major_name,
                assignmentrequest::columns::topic_id,
                topic::columns::topic_name,
                assignmentrequest::columns::assn_req_time,
            ))
            .load::<(
                String,
                String,
                String,
                i32,
                String,
                chrono::DateTime<chrono::Utc>,
            )>(&mut conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to load assignments")))?;

        let approved_total: i64 = approved_base
            .count()
            .get_result(&mut conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to count assignments")))?;
        let approved_rows = approved_base
            .order(student::columns::assn_time.desc())
            .limit(want)
            .select((
                student::columns::user_name,
                student::columns::student_name,
                major::columns::major_name,
                topic::columns::topic_id,
                topic::columns::topic_name,
                student::columns::assn_time,
            ))
            .load::<(
                String,
                String,
                String,
                i32,
                String,
                chrono::DateTime<chrono::Utc>,
            )>(&mut conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to load assignments")))?;

        (pending_total, pending_rows, approved_total, approved_rows)
    } else {
        match user_role {
            Some(AuthInfoUserRole::Student) => {
                let pending_q = pending_base
                    .filter(assignmentrequest::columns::student_user_name.eq(&username));
                let pending_total: i64 = pending_q.count().get_result(&mut conn).map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to count assignments"))
                })?;
                let pending_rows = pending_q
                    .order(assignmentrequest::columns::assn_req_time.desc())
                    .limit(want)
                    .select((
                        assignmentrequest::columns::student_user_name,
                        student::columns::student_name,
                        major::columns::major_name,
                        assignmentrequest::columns::topic_id,
                        topic::columns::topic_name,
                        assignmentrequest::columns::assn_req_time,
                    ))
                    .load::<(
                        String,
                        String,
                        String,
                        i32,
                        String,
                        chrono::DateTime<chrono::Utc>,
                    )>(&mut conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to load assignments"))
                    })?;

                let approved_q = approved_base.filter(student::columns::user_name.eq(&username));
                let approved_total: i64 =
                    approved_q.count().get_result(&mut conn).map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to count assignments"))
                    })?;
                let approved_rows = approved_q
                    .order(student::columns::assn_time.desc())
                    .limit(want)
                    .select((
                        student::columns::user_name,
                        student::columns::student_name,
                        major::columns::major_name,
                        topic::columns::topic_id,
                        topic::columns::topic_name,
                        student::columns::assn_time,
                    ))
                    .load::<(
                        String,
                        String,
                        String,
                        i32,
                        String,
                        chrono::DateTime<chrono::Utc>,
                    )>(&mut conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to load assignments"))
                    })?;

                (pending_total, pending_rows, approved_total, approved_rows)
            }
            Some(AuthInfoUserRole::Teacher) => {
                let pending_q =
                    pending_base.filter(topic::columns::teacher_user_name.eq(&username));
                let pending_total: i64 = pending_q.count().get_result(&mut conn).map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to count assignments"))
                })?;
                let pending_rows = pending_q
                    .order(assignmentrequest::columns::assn_req_time.desc())
                    .limit(want)
                    .select((
                        assignmentrequest::columns::student_user_name,
                        student::columns::student_name,
                        major::columns::major_name,
                        assignmentrequest::columns::topic_id,
                        topic::columns::topic_name,
                        assignmentrequest::columns::assn_req_time,
                    ))
                    .load::<(
                        String,
                        String,
                        String,
                        i32,
                        String,
                        chrono::DateTime<chrono::Utc>,
                    )>(&mut conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to load assignments"))
                    })?;

                let approved_q =
                    approved_base.filter(topic::columns::teacher_user_name.eq(&username));
                let approved_total: i64 =
                    approved_q.count().get_result(&mut conn).map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to count assignments"))
                    })?;
                let approved_rows = approved_q
                    .order(student::columns::assn_time.desc())
                    .limit(want)
                    .select((
                        student::columns::user_name,
                        student::columns::student_name,
                        major::columns::major_name,
                        topic::columns::topic_id,
                        topic::columns::topic_name,
                        student::columns::assn_time,
                    ))
                    .load::<(
                        String,
                        String,
                        String,
                        i32,
                        String,
                        chrono::DateTime<chrono::Utc>,
                    )>(&mut conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to load assignments"))
                    })?;

                (pending_total, pending_rows, approved_total, approved_rows)
            }
            _ => return Err(ApiError::Forbidden),
        }
    };

    let total = pending_total + approved_total;

    let pending = pending_rows.into_iter().map(
        |(student_user_name, student_name, student_major, topic_id, topic_name, request_time)| {
            Assignment {
                student_user_name,
                student_name,
                student_major,
                topic_id,
                topic_name,
                request_time,
                status: AssignmentStatus::Pending,
            }
        },
    );
    let approved = approved_rows.into_iter().map(
        |(student_user_name, student_name, student_major, topic_id, topic_name, assn_time)| {
            Assignment {
                student_user_name,
                student_name,
                student_major,
                topic_id,
                topic_name,
                request_time: assn_time,
                status: AssignmentStatus::Approved,
            }
        },
    );

    // We assume this won't be too long.
    let assignments = pending
        .chain(approved)
        .skip(offset as usize)
        .take(page_size as usize)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(AssignmentsGetResponse {
        total,
        page,
        page_size,
        assignments,
    }))
}

#[post("/assignments")]
pub async fn create_assignment(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<AssignmentsPostRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;
    if !matches!(user_role, AuthInfoUserRole::Student) {
        return Err(ApiError::Forbidden);
    }

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    conn.build_transaction().read_write().run(|conn| {
        let student = student::dsl::student
            .find(&username)
            .first::<Student>(conn)
            .map_err(|_| ApiError::NotFound)?;

        if student.topic_id.is_some() {
            return Err(ApiError::Conflict(str!("Student already has a topic")));
        }

        let topic = topic::dsl::topic
            .find(req.topic_id)
            .first::<Topic>(conn)
            .map_err(|_| ApiError::NotFound)?;

        if topic.topic_review_status != TopicReviewStatus::Approved as i16
            || topic.major_id != student.major_id
        {
            return Err(ApiError::Forbidden);
        }

        let current_student_count: i64 = student::dsl::student
            .filter(student::columns::topic_id.eq(topic.topic_id))
            .count()
            .get_result(conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to count students")))?;
        if current_student_count >= topic.topic_max_students as i64 {
            return Err(ApiError::Conflict(str!("Topic is full")));
        }

        let has_pending = diesel::select(diesel::dsl::exists(
            assignmentrequest::dsl::assignmentrequest
                .filter(assignmentrequest::columns::student_user_name.eq(&username))
                .filter(assignmentrequest::columns::topic_id.eq(req.topic_id)),
        ))
        .get_result(conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to check existing request")))?;
        if has_pending {
            return Err(ApiError::Conflict(str!(
                "Assignment request already exists"
            )));
        }

        diesel::insert_into(assignmentrequest::dsl::assignmentrequest)
            .values(NewAssignmentRequest {
                student_user_name: &username,
                topic_id: req.topic_id,
                assn_req_time: Utc::now(),
            })
            .execute(conn)
            .map_err(|_| ApiError::Conflict(str!("Failed to create assignment request")))?;

        Ok::<_, ApiError>(())
    })?;

    Ok(HttpResponse::Created().finish())
}

#[patch("/assignments/{student_username}/{topic_id}")]
pub async fn update_assignment_status(
    pool: web::Data<DbPool>,
    session: Session,
    path: web::Path<(String, i32)>,
    req: web::Json<AssignmentRecordPatchRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;

    if !matches!(user_role, AuthInfoUserRole::Teacher) {
        // Only teachers can approve/reject assignment requests
        return Err(ApiError::Forbidden);
    }

    let (student_username, topic_id) = path.into_inner();

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    conn.build_transaction().read_write().run(|conn| {
        let req_row = assignmentrequest::dsl::assignmentrequest
            .find((&student_username, topic_id))
            .first::<AssignmentRequest>(conn)
            .map_err(|_| ApiError::NotFound)?;

        let topic = topic::dsl::topic
            .find(req_row.topic_id)
            .first::<Topic>(conn)
            .map_err(|_| ApiError::NotFound)?;
        if topic.teacher_user_name != username {
            return Err(ApiError::Forbidden);
        }

        if req.approved {
            let student = student::dsl::student
                .find(&student_username)
                .first::<Student>(conn)
                .map_err(|_| ApiError::NotFound)?;
            if student.topic_id.is_some() {
                return Err(ApiError::Conflict(str!("Student already has a topic")));
            }

            let current_student_count: i64 = student::dsl::student
                .filter(student::columns::topic_id.eq(topic.topic_id))
                .count()
                .get_result(conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count students")))?;
            if current_student_count >= topic.topic_max_students as i64 {
                return Err(ApiError::Conflict(str!("Topic is full")));
            }

            let changeset = StudentAssignmentChangeset {
                topic_id: topic.topic_id,
                assn_time: req_row.assn_req_time,
            };
            diesel::update(&student)
                .set(changeset)
                .execute(conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to assign topic")))?;
        }

        diesel::delete(
            assignmentrequest::dsl::assignmentrequest.find((&student_username, topic_id)),
        )
        .execute(conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to delete assignment request")))?;

        Ok::<_, ApiError>(())
    })?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/progress_reports")]
pub async fn get_progress_reports(
    pool: web::Data<DbPool>,
    session: Session,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;

    match user_role {
        AuthInfoUserRole::Student | AuthInfoUserRole::Teacher => {}
        _ => return Err(ApiError::Forbidden),
    }

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let mut base = progressreport::table
        .inner_join(student::table)
        .inner_join(topic::table)
        .into_boxed();

    match user_role {
        AuthInfoUserRole::Student => {
            base = base.filter(progressreport::columns::student_user_name.eq(&username));
        }
        AuthInfoUserRole::Teacher => {
            base = base.filter(topic::columns::teacher_user_name.eq(&username));
        }
        _ => unreachable!(),
    }

    let rows = base
        .order(progressreport::columns::prog_report_time.desc())
        .select((progressreport::all_columns, student::columns::student_name))
        .load::<(ProgressReport, String)>(&mut conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to load progress reports")))?;

    if rows.is_empty() {
        return Err(ApiError::NotFound);
    }

    let reports = rows
        .into_iter()
        .map(|(r, student_name)| {
            Ok::<_, ApiError>(ProgressReportDetailResponse {
                prog_report_id: r.prog_report_id,
                topic_id: r.topic_id,
                student_user_name: r.student_user_name,
                student_name,
                prog_report_type: ProgressReportType::try_from(r.prog_report_type).map_err(
                    |_| ApiError::InternalServerError(str!("Invalid progress report type")),
                )?,
                prog_report_time: r.prog_report_time,
                prog_report_attachment: r.prog_report_attachment,
                prog_report_outcome: ProgressOutcome::try_from(r.prog_report_outcome)
                    .map_err(|_| ApiError::InternalServerError(str!("Invalid progress outcome")))?,
                prog_report_comment: r.prog_report_comment,
                prog_report_grade: r.prog_report_grade,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(HttpResponse::Ok().json(ProgressReportsGetResponse { reports }))
}

#[post("/progress_reports")]
pub async fn create_progress_report(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<ProgressReportsPostRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    conn.build_transaction().read_write().run(|conn| {
        let student = student::dsl::student
            .find(&username)
            .first::<Student>(conn)
            .map_err(|_| ApiError::NotFound)?;
        let topic_id = student
            .topic_id
            .ok_or_else(|| ApiError::Conflict(str!("Student has no assigned topic")))?;

        // Determine report type: proposal first, then midterm after proposal passed.
        let has_passed_proposal = diesel::select(diesel::dsl::exists(
            progressreport::dsl::progressreport
                .filter(progressreport::columns::student_user_name.eq(&username))
                .filter(
                    progressreport::columns::prog_report_type
                        .eq(ProgressReportType::Proposal as i16),
                )
                .filter(
                    progressreport::columns::prog_report_outcome.eq(ProgressOutcome::Passed as i16),
                ),
        ))
        .get_result::<bool>(conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to check proposal status")))?;

        let report_type = if has_passed_proposal {
            ProgressReportType::Midterm
        } else {
            ProgressReportType::Proposal
        };

        // Enforce: per (student, type) at most one pending and one passed.
        let has_pending = diesel::select(diesel::dsl::exists(
            progressreport::dsl::progressreport
                .filter(progressreport::columns::student_user_name.eq(&username))
                .filter(progressreport::columns::prog_report_type.eq(report_type as i16))
                .filter(
                    progressreport::columns::prog_report_outcome
                        .eq(ProgressOutcome::NoConclusion as i16),
                ),
        ))
        .get_result(conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to check pending reports")))?;
        if has_pending {
            return Err(ApiError::Conflict(str!("A pending report already exists")));
        }

        let has_passed = diesel::select(diesel::dsl::exists(
            progressreport::dsl::progressreport
                .filter(progressreport::columns::student_user_name.eq(&username))
                .filter(progressreport::columns::prog_report_type.eq(report_type as i16))
                .filter(
                    progressreport::columns::prog_report_outcome.eq(ProgressOutcome::Passed as i16),
                ),
        ))
        .get_result(conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to check passed reports")))?;
        if has_passed {
            return Err(ApiError::Conflict(str!("A passed report already exists")));
        }

        diesel::insert_into(progressreport::dsl::progressreport)
            .values(NewProgressReport {
                topic_id,
                student_user_name: &username,
                prog_report_type: report_type as i16,
                prog_report_time: Utc::now(),
                prog_report_attachment: &req.attachment,
                prog_report_outcome: ProgressOutcome::NoConclusion as i16,
                prog_report_comment: None,
                prog_report_grade: None,
            })
            .execute(conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to create progress report")))?;

        Ok::<_, ApiError>(())
    })?;

    Ok(HttpResponse::Ok().finish())
}

#[patch("/progress_reports/{report_id}")]
pub async fn update_progress_report(
    pool: web::Data<DbPool>,
    session: Session,
    report_id: web::Path<i32>,
    req: web::Json<ProgressReportRecordPatchRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;
    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let result = conn.build_transaction().read_write().run(|conn| {
        let (report, student_name, teacher_username) = progressreport::table
            .inner_join(student::table)
            .inner_join(topic::table)
            .filter(progressreport::columns::prog_report_id.eq(*report_id))
            .select((
                progressreport::all_columns,
                student::columns::student_name,
                topic::columns::teacher_user_name,
            ))
            .first::<(ProgressReport, String, String)>(conn)
            .map_err(|_| ApiError::NotFound)?;

        if teacher_username != username {
            return Err(ApiError::Forbidden);
        }

        // Enforce per (student, type): at most one pending + one passed.
        match req.outcome {
            ProgressOutcome::NoConclusion => {
                let other_pending = diesel::select(diesel::dsl::exists(
                    progressreport::dsl::progressreport
                        .filter(
                            progressreport::columns::student_user_name
                                .eq(&report.student_user_name),
                        )
                        .filter(
                            progressreport::columns::prog_report_type.eq(report.prog_report_type),
                        )
                        .filter(
                            progressreport::columns::prog_report_outcome
                                .eq(ProgressOutcome::NoConclusion as i16),
                        )
                        .filter(progressreport::columns::prog_report_id.ne(report.prog_report_id)),
                ))
                .get_result::<bool>(conn)
                .map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to check pending reports"))
                })?;
                if other_pending {
                    return Err(ApiError::Conflict(str!(
                        "Another pending report already exists"
                    )));
                }
            }
            ProgressOutcome::Passed => {
                let other_passed = diesel::select(diesel::dsl::exists(
                    progressreport::dsl::progressreport
                        .filter(
                            progressreport::columns::student_user_name
                                .eq(&report.student_user_name),
                        )
                        .filter(
                            progressreport::columns::prog_report_type.eq(report.prog_report_type),
                        )
                        .filter(
                            progressreport::columns::prog_report_outcome
                                .eq(ProgressOutcome::Passed as i16),
                        )
                        .filter(progressreport::columns::prog_report_id.ne(report.prog_report_id)),
                ))
                .get_result::<bool>(conn)
                .map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to check passed reports"))
                })?;
                if other_passed {
                    return Err(ApiError::Conflict(str!(
                        "Another passed report already exists"
                    )));
                }
            }
            ProgressOutcome::Rejected => {}
        }

        diesel::update(progressreport::dsl::progressreport.find(report.prog_report_id))
            .set((
                progressreport::columns::prog_report_outcome.eq(req.outcome as i16),
                progressreport::columns::prog_report_comment.eq(req.comment.clone()),
                progressreport::columns::prog_report_grade.eq(req.grade.clone()),
            ))
            .execute(conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to update progress report")))?;

        let updated = progressreport::dsl::progressreport
            .find(report.prog_report_id)
            .first::<ProgressReport>(conn)
            .map_err(|_| ApiError::NotFound)?;

        let result = ProgressReportDetailResponse {
            prog_report_id: updated.prog_report_id,
            topic_id: updated.topic_id,
            student_user_name: updated.student_user_name,
            student_name,
            prog_report_type: ProgressReportType::try_from(updated.prog_report_type)
                .map_err(|_| ApiError::InternalServerError(str!("Invalid progress report type")))?,
            prog_report_time: updated.prog_report_time,
            prog_report_attachment: updated.prog_report_attachment,
            prog_report_outcome: ProgressOutcome::try_from(updated.prog_report_outcome)
                .map_err(|_| ApiError::InternalServerError(str!("Invalid progress outcome")))?,
            prog_report_comment: updated.prog_report_comment,
            prog_report_grade: updated.prog_report_grade,
        };

        Ok::<_, ApiError>(result)
    })?;

    Ok(HttpResponse::Ok().json(result))
}

#[get("/final_defenses")]
pub async fn get_final_defenses(
    pool: web::Data<DbPool>,
    session: Session,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;

    match user_role {
        AuthInfoUserRole::Student | AuthInfoUserRole::Teacher | AuthInfoUserRole::DefenseBoard => {}
        _ => return Err(ApiError::Forbidden),
    }

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let mut base = finaldefense::table
        .inner_join(student::table)
        .inner_join(topic::table)
        .into_boxed();

    match user_role {
        AuthInfoUserRole::Student => {
            base = base.filter(finaldefense::columns::student_user_name.eq(&username));
        }
        AuthInfoUserRole::Teacher => {
            base = base.filter(topic::columns::teacher_user_name.eq(&username));
        }
        AuthInfoUserRole::DefenseBoard => {
            base = base.filter(finaldefense::columns::def_board_user_name.eq(&username));
        }
        _ => unreachable!(),
    }

    let rows = base
        .order(finaldefense::columns::final_def_time.desc())
        .select((
            finaldefense::all_columns,
            student::columns::student_name,
            topic::columns::topic_name,
        ))
        .load::<(FinalDefense, String, String)>(&mut conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to load final defenses")))?;

    if rows.is_empty() {
        return Err(ApiError::NotFound);
    }

    let defenses = rows
        .into_iter()
        .map(|(d, student_name, topic_name)| FinalDefenseDetails {
            final_def_id: d.final_def_id,
            topic_id: d.topic_id,
            topic_name,
            student_user_name: d.student_user_name,
            student_name,
            def_board_user_name: d.def_board_user_name,
            final_def_time: d.final_def_time,
            final_def_attachment: d.final_def_attachment,
            final_def_outcome: d.final_def_outcome,
            final_def_comment: d.final_def_comment,
            final_def_grade: d.final_def_grade,
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(FinalDefensesGetResponse { defenses }))
}

#[post("/final_defenses")]
pub async fn create_final_defense(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<FinalDefensesPostRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    conn.build_transaction().read_write().run(|conn| {
        let student = student::dsl::student
            .find(&username)
            .first::<Student>(conn)
            .map_err(|_| ApiError::NotFound)?;
        let topic_id = student
            .topic_id
            .ok_or_else(|| ApiError::Conflict(str!("Student has no assigned topic")))?;

        // Enforce: per student, at most one pending (NULL) and one passed (true).
        let has_pending = diesel::select(diesel::dsl::exists(
            finaldefense::dsl::finaldefense
                .filter(finaldefense::columns::student_user_name.eq(&username))
                .filter(finaldefense::columns::final_def_outcome.is_null()),
        ))
        .get_result::<bool>(conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to check pending defenses")))?;
        if has_pending {
            return Err(ApiError::Conflict(str!(
                "A pending final defense already exists"
            )));
        }

        let has_passed = diesel::select(diesel::dsl::exists(
            finaldefense::dsl::finaldefense
                .filter(finaldefense::columns::student_user_name.eq(&username))
                .filter(finaldefense::columns::final_def_outcome.eq(true)),
        ))
        .get_result::<bool>(conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to check passed defenses")))?;
        if has_passed {
            return Err(ApiError::Conflict(str!(
                "A passed final defense already exists"
            )));
        }

        diesel::insert_into(finaldefense::dsl::finaldefense)
            .values(NewFinalDefense {
                topic_id,
                student_user_name: &username,
                def_board_user_name: None,
                final_def_time: Utc::now(),
                final_def_attachment: &req.attachment,
                final_def_outcome: None,
                final_def_comment: None,
                final_def_grade: None,
            })
            .execute(conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to create final defense")))?;

        Ok::<_, ApiError>(())
    })?;

    Ok(HttpResponse::Ok().finish())
}

#[patch("/final_defenses/{report_id}")]
pub async fn update_final_defense(
    pool: web::Data<DbPool>,
    session: Session,
    report_id: web::Path<i32>,
    req: web::Json<FinalDefensesRecordPatchRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema::*;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let username = get_session_username(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get username from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let result = conn.build_transaction().read_write().run(|conn| {
        match (user_role, req.into_inner()) {
            (AuthInfoUserRole::Teacher, FinalDefensesRecordPatchRequest::Teacher(req)) => {
                // Teacher reviews the request: reject -> delete; approve -> assign least-loaded defense group.
                let (defense, student_name, topic_name, teacher_username): (
                    FinalDefense,
                    String,
                    String,
                    String,
                ) = finaldefense::table
                    .inner_join(student::table)
                    .inner_join(topic::table)
                    .filter(finaldefense::columns::final_def_id.eq(*report_id))
                    .select((
                        finaldefense::all_columns,
                        student::columns::student_name,
                        topic::columns::topic_name,
                        topic::columns::teacher_user_name,
                    ))
                    .first(conn)
                    .map_err(|_| ApiError::NotFound)?;

                if teacher_username != username {
                    return Err(ApiError::Forbidden);
                }

                // Only allow reviewing pending applications.
                if defense.final_def_outcome.is_some() {
                    return Err(ApiError::Conflict(str!("Final defense already finalized")));
                }

                if !req.approved {
                    diesel::delete(finaldefense::dsl::finaldefense.find(defense.final_def_id))
                        .execute(conn)
                        .map_err(|_| {
                            ApiError::InternalServerError(str!("Failed to delete final defense"))
                        })?;

                    // Return details of the deleted record (pre-delete).
                    return Ok(FinalDefenseDetails {
                        final_def_id: defense.final_def_id,
                        topic_id: defense.topic_id,
                        topic_name,
                        student_user_name: defense.student_user_name,
                        student_name,
                        def_board_user_name: defense.def_board_user_name,
                        final_def_time: defense.final_def_time,
                        final_def_attachment: defense.final_def_attachment,
                        final_def_outcome: defense.final_def_outcome,
                        final_def_comment: defense.final_def_comment,
                        final_def_grade: defense.final_def_grade,
                    });
                }

                if defense.def_board_user_name.is_some() {
                    return Err(ApiError::Conflict(str!("Final defense already assigned")));
                }

                // Choose the defense group with the least number of pending tasks.
                // (Pending task = assigned to a defense group AND final outcome is NULL.)
                // We compute counts via GROUP BY, then pick the minimum in Rust to keep the Diesel types simple.

                let defense_board_usernames = defenseboard::dsl::defenseboard
                    .select(defenseboard::columns::user_name)
                    .order(defenseboard::columns::user_name.asc())
                    .load::<String>(conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to load defense boards"))
                    })?;
                if defense_board_usernames.is_empty() {
                    return Err(ApiError::Conflict(str!("No defense board available")));
                }

                let pending_counts = finaldefense::dsl::finaldefense
                    .filter(finaldefense::columns::final_def_outcome.is_null())
                    .filter(finaldefense::columns::def_board_user_name.is_not_null())
                    .group_by(finaldefense::columns::def_board_user_name)
                    .select((
                        finaldefense::columns::def_board_user_name,
                        diesel::dsl::count_star(),
                    ))
                    .load::<(Option<String>, i64)>(conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to count defense board tasks"))
                    })?;

                let mut count_by_board: HashMap<String, i64> = HashMap::new();
                for (board_username, c) in pending_counts {
                    if let Some(board_username) = board_username {
                        count_by_board.insert(board_username, c);
                    }
                }

                let mut assigned_defense_board_username = defense_board_usernames[0].clone();
                let mut assigned_count = *count_by_board
                    .get(&assigned_defense_board_username)
                    .unwrap_or(&0);
                for board_username in defense_board_usernames.into_iter().skip(1) {
                    let c = *count_by_board.get(&board_username).unwrap_or(&0);
                    if c < assigned_count {
                        assigned_defense_board_username = board_username;
                        assigned_count = c;
                    }
                }

                diesel::update(finaldefense::dsl::finaldefense.find(defense.final_def_id))
                    .set(
                        finaldefense::columns::def_board_user_name
                            .eq(Some(assigned_defense_board_username)),
                    )
                    .execute(conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to assign defense board"))
                    })?;

                let updated = finaldefense::dsl::finaldefense
                    .find(defense.final_def_id)
                    .first::<FinalDefense>(conn)
                    .map_err(|_| ApiError::NotFound)?;

                Ok(FinalDefenseDetails {
                    final_def_id: updated.final_def_id,
                    topic_id: updated.topic_id,
                    topic_name,
                    student_user_name: updated.student_user_name,
                    student_name,
                    def_board_user_name: updated.def_board_user_name,
                    final_def_time: updated.final_def_time,
                    final_def_attachment: updated.final_def_attachment,
                    final_def_outcome: updated.final_def_outcome,
                    final_def_comment: updated.final_def_comment,
                    final_def_grade: updated.final_def_grade,
                })
            }
            (
                AuthInfoUserRole::DefenseBoard,
                FinalDefensesRecordPatchRequest::DefenseBoard(req),
            ) => {
                let (defense, student_name, topic_name): (FinalDefense, String, String) =
                    finaldefense::table
                        .inner_join(student::table)
                        .inner_join(topic::table)
                        .filter(finaldefense::columns::final_def_id.eq(*report_id))
                        .select((
                            finaldefense::all_columns,
                            student::columns::student_name,
                            topic::columns::topic_name,
                        ))
                        .first(conn)
                        .map_err(|_| ApiError::NotFound)?;

                if defense.def_board_user_name != Some(username) {
                    return Err(ApiError::Forbidden);
                }

                // Enforce: per student, at most one passed (true).
                if req.outcome {
                    let other_passed = diesel::select(diesel::dsl::exists(
                        finaldefense::dsl::finaldefense
                            .filter(
                                finaldefense::columns::student_user_name
                                    .eq(&defense.student_user_name),
                            )
                            .filter(finaldefense::columns::final_def_outcome.eq(true))
                            .filter(finaldefense::columns::final_def_id.ne(defense.final_def_id)),
                    ))
                    .get_result::<bool>(conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to check passed defenses"))
                    })?;
                    if other_passed {
                        return Err(ApiError::Conflict(str!(
                            "Another passed final defense already exists"
                        )));
                    }
                }

                diesel::update(finaldefense::dsl::finaldefense.find(defense.final_def_id))
                    .set((
                        finaldefense::columns::final_def_outcome.eq(Some(req.outcome)),
                        finaldefense::columns::final_def_comment.eq(Some(req.comment.clone())),
                        finaldefense::columns::final_def_grade.eq(Some(req.grade.clone())),
                    ))
                    .execute(conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to update final defense"))
                    })?;

                let updated = finaldefense::dsl::finaldefense
                    .find(defense.final_def_id)
                    .first::<FinalDefense>(conn)
                    .map_err(|_| ApiError::NotFound)?;

                Ok(FinalDefenseDetails {
                    final_def_id: updated.final_def_id,
                    topic_id: updated.topic_id,
                    topic_name,
                    student_user_name: updated.student_user_name,
                    student_name,
                    def_board_user_name: updated.def_board_user_name,
                    final_def_time: updated.final_def_time,
                    final_def_attachment: updated.final_def_attachment,
                    final_def_outcome: updated.final_def_outcome,
                    final_def_comment: updated.final_def_comment,
                    final_def_grade: updated.final_def_grade,
                })
            }
            (AuthInfoUserRole::Teacher, _) => Err(ApiError::BadRequest(str!(
                "Teacher patch request must be { approved: bool }"
            ))),
            (AuthInfoUserRole::DefenseBoard, _) => Err(ApiError::BadRequest(str!(
                "Defense board patch request must be { outcome, comment, grade }"
            ))),
            _ => Err(ApiError::Forbidden),
        }
    })?;

    Ok(HttpResponse::Ok().json(result))
}
