use actix_session::Session;
use actix_web::{HttpResponse, ResponseError, get, patch, post, web};
use backend_database::DbPool;
use backend_database::model::*;
use chrono::Utc;
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
        .find(user_id)
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
                .find(user_id)
                .first::<Student>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to get student information")))?;
            Ok((UserRole::Student, Some(student.student_name)))
        };
        schema::teacher::dsl::teacher => Teacher => {
            let teacher = schema::teacher::dsl::teacher
                .find(user_id)
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
        let mut changeset = SysUserChangeset {
            user_password_hash: None,
            user_password_salt: None,
            user_avatar: req.avatar.clone().map(Some),
        };

        if let Some(ref password) = req.password {
            let (hash, salt) = hash_password(password)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to hash password")))?;
            changeset.user_password_hash = Some(hash);
            changeset.user_password_salt = Some(salt);
        }

        diesel::update(schema::sysuser::dsl::sysuser.find(user_id))
            .set(changeset)
            .execute(conn)
            .map_err(|_| {
                ApiError::InternalServerError(str!("Failed to update user information"))
            })?;

        if let Some(ref avatar) = req.avatar {
            diesel::update(schema::sysuser::dsl::sysuser.find(user_id))
                .set(schema::sysuser::columns::user_avatar.eq(avatar))
                .execute(conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to update avatar")))?;
        }

        if let Some(ref name) = req.name {
            // Is student?
            if diesel::select(diesel::dsl::exists(
                schema::student::dsl::student.find(user_id),
            ))
            .get_result::<bool>(conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to check user role")))?
            {
                diesel::update(schema::student::dsl::student.find(user_id))
                    .set(schema::student::columns::student_name.eq(name))
                    .execute(conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to update student name"))
                    })?;
            }
            // Is teacher?
            else if diesel::select(diesel::dsl::exists(
                schema::teacher::dsl::teacher.find(user_id),
            ))
            .get_result::<bool>(conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to check user role")))?
            {
                diesel::update(schema::teacher::dsl::teacher.find(user_id))
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
    pool: web::Data<DbPool>,
    session: Session,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let user_id = get_session_user_id(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user ID from session")))?;
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
            let student_info = schema::student::dsl::student
                .find(user_id)
                .first::<Student>(&mut conn)
                .map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to get student information"))
                })?;

            let query = schema::topic::table
                .inner_join(schema::teacher::table)
                .filter(schema::topic::columns::major_id.eq(student_info.major_id))
                .filter(
                    schema::topic::columns::topic_review_status
                        .eq(TopicReviewStatus::Approved as i16),
                );
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(schema::topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
        AuthInfoUserRole::Teacher => {
            // Teacher: Get their own topics
            let query = schema::topic::table
                .inner_join(schema::teacher::table)
                .filter(schema::topic::columns::user_id.eq(user_id));
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(schema::topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
        AuthInfoUserRole::Office | AuthInfoUserRole::DefenseBoard => {
            // Office and DefenseBoard: Get all topics
            // No additional filtering needed.
            let query = schema::topic::table.inner_join(schema::teacher::table);
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(schema::topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
    };

    let mut topic_briefs = Vec::new();
    for (topic, teacher) in topics_with_teacher {
        let current_student_count = schema::student::dsl::student
            .filter(schema::student::columns::topic_id.eq(topic.topic_id))
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
    use backend_database::schema;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;
    if !matches!(user_role, AuthInfoUserRole::Teacher) {
        // Only teachers can create topics
        return Err(ApiError::Forbidden);
    }

    let user_id = get_session_user_id(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user ID from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let has_such_teacher: i64 = schema::teacher::dsl::teacher
        .find(user_id)
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
            user_id,
            topic_name: &req.topic_name,
            topic_description: &req.topic_description,
            topic_max_students: req.topic_max_students,
            topic_type: req.topic_type as i16,
            topic_review_status: TopicReviewStatus::Pending as i16,
        };

        let inserted_topic = diesel::insert_into(schema::topic::dsl::topic)
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
    use backend_database::schema;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let user_id = get_session_user_id(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user ID from session")))?;
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
            let student_info = schema::student::dsl::student
                .find(user_id)
                .first::<Student>(&mut conn)
                .map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to get student information"))
                })?;

            let query = schema::topic::table
                .inner_join(schema::teacher::table)
                .filter(schema::topic::columns::major_id.eq(student_info.major_id))
                .filter(
                    schema::topic::columns::topic_review_status
                        .eq(TopicReviewStatus::Approved as i16),
                )
                .filter(schema::topic::columns::topic_name.like(&search_pattern));
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(schema::topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
        AuthInfoUserRole::Teacher => {
            // Teacher: Get their own topics with keyword search
            let query = schema::topic::table
                .inner_join(schema::teacher::table)
                .filter(schema::topic::columns::user_id.eq(user_id))
                .filter(schema::topic::columns::topic_name.like(&search_pattern));
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(schema::topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
        AuthInfoUserRole::Office | AuthInfoUserRole::DefenseBoard => {
            // Office and DefenseBoard: Get all topics with keyword search
            let query = schema::topic::table
                .inner_join(schema::teacher::table)
                .filter(schema::topic::columns::topic_name.like(&search_pattern));
            let total = query
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to count topics")))?;
            let topics_with_teacher = query
                .offset(offset)
                .limit(page_size)
                .order_by(schema::topic::columns::topic_id.desc())
                .load::<(Topic, Teacher)>(&mut conn)
                .map_err(|_| ApiError::InternalServerError(str!("Failed to load topics")))?;
            (total, topics_with_teacher)
        }
    };

    let mut topic_briefs = Vec::new();
    for (topic, teacher) in topics_with_teacher {
        let current_student_count = schema::student::dsl::student
            .filter(schema::student::columns::topic_id.eq(topic.topic_id))
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
    use backend_database::schema;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let user_id = get_session_user_id(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user ID from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    // Why boxed query here? Since we don't need it to be Copy, unlike the previous (count, records) pattern.
    let mut query_builder = schema::topic::table
        .inner_join(schema::teacher::table)
        .inner_join(schema::major::table)
        .filter(schema::topic::columns::topic_id.eq(*topic_id))
        .into_boxed();

    match user_role {
        AuthInfoUserRole::Student => {
            // Student: Get topics approved for their major
            let student_info = schema::student::dsl::student
                .filter(schema::student::columns::user_id.eq(user_id))
                .first::<Student>(&mut conn)
                .map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to get student information"))
                })?;

            query_builder = query_builder
                .filter(schema::topic::columns::major_id.eq(student_info.major_id))
                .filter(
                    schema::topic::columns::topic_review_status
                        .eq(TopicReviewStatus::Approved as i16),
                );
        }
        AuthInfoUserRole::Teacher => {
            // Teacher: Get topics created by themselves
            query_builder = query_builder.filter(schema::topic::columns::user_id.eq(user_id));
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

    let current_student_count: i64 = schema::student::dsl::student
        .filter(schema::student::columns::topic_id.eq(topic.topic_id))
        .count()
        .get_result(&mut conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to count students for topic")))?;

    // 构建响应
    let topic_details = TopicDetails {
        topic_id: topic.topic_id,
        major_id: topic.major_id,
        major_name: major.major_name,
        teacher_id: topic.user_id,
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
    use backend_database::schema;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let user_id = get_session_user_id(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user ID from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let result = conn.build_transaction().read_write().run(|conn| {
        let topic = schema::topic::dsl::topic
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
                if topic.user_id != user_id {
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
                        ApiError::InternalServerError(format!(
                            "Failed to update review status: {}",
                            e
                        ))
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
                        .set(
                            schema::topic::columns::topic_review_status
                                .eq(topic_review_status as i16),
                        )
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

        let teacher = schema::teacher::dsl::teacher
            .find(topic.user_id)
            .first::<Teacher>(conn)
            .map_err(|e| {
                ApiError::InternalServerError(format!("Failed to load topic teacher: {}", e))
            })?;
        let major = schema::major::dsl::major
            .find(topic.major_id)
            .first::<Major>(conn)
            .map_err(|e| {
                ApiError::InternalServerError(format!("Failed to load topic major: {}", e))
            })?;

        let current_student_count: i64 = schema::student::dsl::student
            .filter(schema::student::columns::topic_id.eq(topic.topic_id))
            .count()
            .get_result(conn)
            .map_err(|_| {
                ApiError::InternalServerError(str!("Failed to count students for topic"))
            })?;

        let topic_details = TopicDetails {
            topic_id: topic.topic_id,
            major_id: topic.major_id,
            major_name: major.major_name,
            teacher_id: topic.user_id,
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
    use backend_database::schema;
    use chrono::{TimeZone, Utc};

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let is_admin = is_session_admin(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Can't deserialize auth info")))?;

    let user_id = get_session_user_id(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user ID from session")))?;
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
    let epoch = Utc.timestamp_opt(0, 0).single().unwrap();

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    let pending_base = schema::assignmentrequest::table
        .inner_join(schema::student::table.inner_join(schema::major::table))
        .inner_join(schema::topic::table);

    let approved_base = schema::student::table
        .inner_join(schema::major::table)
        .inner_join(schema::topic::table)
        .filter(schema::student::columns::topic_id.is_not_null());

    let (pending_total, pending_rows, approved_total, approved_rows) = if is_admin {
        let pending_total: i64 = pending_base
            .count()
            .get_result(&mut conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to count assignments")))?;
        let pending_rows = pending_base
            .order(schema::assignmentrequest::columns::assn_req_time.desc())
            .limit(want)
            .select((
                schema::assignmentrequest::columns::user_id,
                schema::student::columns::student_name,
                schema::major::columns::major_name,
                schema::assignmentrequest::columns::topic_id,
                schema::topic::columns::topic_name,
                schema::assignmentrequest::columns::assn_req_time,
            ))
            .load::<(
                i32,
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
            .order(schema::student::columns::assn_time.desc())
            .limit(want)
            .select((
                schema::student::columns::user_id,
                schema::student::columns::student_name,
                schema::major::columns::major_name,
                schema::topic::columns::topic_id,
                schema::topic::columns::topic_name,
                schema::student::columns::assn_time,
            ))
            .load::<(
                i32,
                String,
                String,
                i32,
                String,
                Option<chrono::DateTime<chrono::Utc>>,
            )>(&mut conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to load assignments")))?;

        (pending_total, pending_rows, approved_total, approved_rows)
    } else {
        match user_role {
            Some(AuthInfoUserRole::Student) => {
                let pending_q =
                    pending_base.filter(schema::assignmentrequest::columns::user_id.eq(user_id));
                let pending_total: i64 = pending_q.count().get_result(&mut conn).map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to count assignments"))
                })?;
                let pending_rows = pending_q
                    .order(schema::assignmentrequest::columns::assn_req_time.desc())
                    .limit(want)
                    .select((
                        schema::assignmentrequest::columns::user_id,
                        schema::student::columns::student_name,
                        schema::major::columns::major_name,
                        schema::assignmentrequest::columns::topic_id,
                        schema::topic::columns::topic_name,
                        schema::assignmentrequest::columns::assn_req_time,
                    ))
                    .load::<(
                        i32,
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
                    approved_base.filter(schema::student::columns::user_id.eq(user_id));
                let approved_total: i64 =
                    approved_q.count().get_result(&mut conn).map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to count assignments"))
                    })?;
                let approved_rows = approved_q
                    .order(schema::student::columns::assn_time.desc())
                    .limit(want)
                    .select((
                        schema::student::columns::user_id,
                        schema::student::columns::student_name,
                        schema::major::columns::major_name,
                        schema::topic::columns::topic_id,
                        schema::topic::columns::topic_name,
                        schema::student::columns::assn_time,
                    ))
                    .load::<(
                        i32,
                        String,
                        String,
                        i32,
                        String,
                        Option<chrono::DateTime<chrono::Utc>>,
                    )>(&mut conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to load assignments"))
                    })?;

                (pending_total, pending_rows, approved_total, approved_rows)
            }
            Some(AuthInfoUserRole::Teacher) => {
                let pending_q = pending_base.filter(schema::topic::columns::user_id.eq(user_id));
                let pending_total: i64 = pending_q.count().get_result(&mut conn).map_err(|_| {
                    ApiError::InternalServerError(str!("Failed to count assignments"))
                })?;
                let pending_rows = pending_q
                    .order(schema::assignmentrequest::columns::assn_req_time.desc())
                    .limit(want)
                    .select((
                        schema::assignmentrequest::columns::user_id,
                        schema::student::columns::student_name,
                        schema::major::columns::major_name,
                        schema::assignmentrequest::columns::topic_id,
                        schema::topic::columns::topic_name,
                        schema::assignmentrequest::columns::assn_req_time,
                    ))
                    .load::<(
                        i32,
                        String,
                        String,
                        i32,
                        String,
                        chrono::DateTime<chrono::Utc>,
                    )>(&mut conn)
                    .map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to load assignments"))
                    })?;

                let approved_q = approved_base.filter(schema::topic::columns::user_id.eq(user_id));
                let approved_total: i64 =
                    approved_q.count().get_result(&mut conn).map_err(|_| {
                        ApiError::InternalServerError(str!("Failed to count assignments"))
                    })?;
                let approved_rows = approved_q
                    .order(schema::student::columns::assn_time.desc())
                    .limit(want)
                    .select((
                        schema::student::columns::user_id,
                        schema::student::columns::student_name,
                        schema::major::columns::major_name,
                        schema::topic::columns::topic_id,
                        schema::topic::columns::topic_name,
                        schema::student::columns::assn_time,
                    ))
                    .load::<(
                        i32,
                        String,
                        String,
                        i32,
                        String,
                        Option<chrono::DateTime<chrono::Utc>>,
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
        |(student_id, student_name, student_major, topic_id, topic_name, request_time)| {
            Assignment {
                student_id,
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
        |(student_id, student_name, student_major, topic_id, topic_name, assn_time)| Assignment {
            student_id,
            student_name,
            student_major,
            topic_id,
            topic_name,
            request_time: assn_time.unwrap_or(epoch),
            status: AssignmentStatus::Approved,
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
    use backend_database::schema;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let user_id = get_session_user_id(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user ID from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;
    if !matches!(user_role, AuthInfoUserRole::Student) {
        return Err(ApiError::Forbidden);
    }

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    conn.build_transaction().read_write().run(|conn| {
        let student = schema::student::dsl::student
            .find(user_id)
            .first::<Student>(conn)
            .map_err(|_| ApiError::NotFound)?;

        if student.topic_id.is_some() {
            return Err(ApiError::Conflict(str!("Student already has a topic")));
        }

        let topic = schema::topic::dsl::topic
            .find(req.topic_id)
            .first::<Topic>(conn)
            .map_err(|_| ApiError::NotFound)?;

        if topic.topic_review_status != TopicReviewStatus::Approved as i16
            || topic.major_id != student.major_id
        {
            return Err(ApiError::Forbidden);
        }

        let current_student_count: i64 = schema::student::dsl::student
            .filter(schema::student::columns::topic_id.eq(topic.topic_id))
            .count()
            .get_result(conn)
            .map_err(|_| ApiError::InternalServerError(str!("Failed to count students")))?;
        if current_student_count >= topic.topic_max_students as i64 {
            return Err(ApiError::Conflict(str!("Topic is full")));
        }

        let has_pending = diesel::select(diesel::dsl::exists(
            schema::assignmentrequest::dsl::assignmentrequest
                .filter(schema::assignmentrequest::columns::user_id.eq(user_id))
                .filter(schema::assignmentrequest::columns::topic_id.eq(req.topic_id)),
        ))
        .get_result(conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to check existing request")))?;
        if has_pending {
            return Err(ApiError::Conflict(str!(
                "Assignment request already exists"
            )));
        }

        diesel::insert_into(schema::assignmentrequest::dsl::assignmentrequest)
            .values(NewAssignmentRequest {
                user_id,
                topic_id: req.topic_id,
                assn_req_time: Utc::now(),
            })
            .execute(conn)
            .map_err(|_| ApiError::Conflict(str!("Failed to create assignment request")))?;

        Ok::<_, ApiError>(())
    })?;

    Ok(HttpResponse::Created().finish())
}

#[patch("/assignments/{student_id}/{topic_id}")]
pub async fn update_assignment_status(
    pool: web::Data<DbPool>,
    session: Session,
    path: web::Path<(i32, i32)>,
    req: web::Json<AssignmentRecordPatchRequest>,
) -> Result<HttpResponse, ApiError> {
    use backend_database::schema;

    if !is_session_authed(&session) {
        return Err(ApiError::Unauthorized);
    }

    let user_id = get_session_user_id(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user ID from session")))?;
    let user_role = get_session_user_role(&session)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get user role from session")))?;

    if !matches!(user_role, AuthInfoUserRole::Teacher) {
        // Only teachers can approve/reject assignment requests
        return Err(ApiError::Forbidden);
    }

    let (student_id, topic_id) = path.into_inner();

    let mut conn = pool
        .get()
        .map_err(|_| ApiError::InternalServerError(str!("Failed to get database connection")))?;

    conn.build_transaction().read_write().run(|conn| {
        let req_row = schema::assignmentrequest::dsl::assignmentrequest
            .find((student_id, topic_id))
            .first::<AssignmentRequest>(conn)
            .map_err(|_| ApiError::NotFound)?;

        let topic = schema::topic::dsl::topic
            .find(req_row.topic_id)
            .first::<Topic>(conn)
            .map_err(|_| ApiError::NotFound)?;
        if topic.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        if req.approved {
            let student = schema::student::dsl::student
                .find(student_id)
                .first::<Student>(conn)
                .map_err(|_| ApiError::NotFound)?;
            if student.topic_id.is_some() {
                return Err(ApiError::Conflict(str!("Student already has a topic")));
            }

            let current_student_count: i64 = schema::student::dsl::student
                .filter(schema::student::columns::topic_id.eq(topic.topic_id))
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
            schema::assignmentrequest::dsl::assignmentrequest.find((student_id, topic_id)),
        )
        .execute(conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to delete assignment request")))?;

        Ok::<_, ApiError>(())
    })?;

    Ok(HttpResponse::Ok().finish())
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
