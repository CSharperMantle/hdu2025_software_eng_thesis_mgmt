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

            let query = schema::topic::dsl::topic
                .inner_join(schema::teacher::table)
                .filter(
                    schema::topic::columns::major_id
                        .eq(student_info.major_id)
                        .and(
                            schema::topic::columns::topic_review_status
                                .eq(TopicReviewStatus::Approved as i16),
                        ),
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
            let query = schema::topic::dsl::topic
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
            let query = schema::topic::dsl::topic.inner_join(schema::teacher::table);
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

            let query = schema::topic::dsl::topic
                .inner_join(schema::teacher::table)
                .filter(
                    schema::topic::columns::major_id
                        .eq(student_info.major_id)
                        .and(
                            schema::topic::columns::topic_review_status
                                .eq(TopicReviewStatus::Approved as i16),
                        )
                        .and(schema::topic::columns::topic_name.like(&search_pattern)),
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
            // Teacher: Get their own topics with keyword search
            let query = schema::topic::dsl::topic
                .inner_join(schema::teacher::table)
                .filter(
                    schema::topic::columns::user_id
                        .eq(user_id)
                        .and(schema::topic::columns::topic_name.like(&search_pattern)),
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
        AuthInfoUserRole::Office | AuthInfoUserRole::DefenseBoard => {
            // Office and DefenseBoard: Get all topics with keyword search
            let query = schema::topic::dsl::topic
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
    let mut query_builder = schema::topic::dsl::topic
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

    // First, check if the topic exists and get current topic info
    let topic = schema::topic::dsl::topic
        .find(*topic_id)
        .first::<Topic>(&mut conn)
        .map_err(|e| {
            if let diesel::result::Error::NotFound = e {
                ApiError::NotFound
            } else {
                ApiError::InternalServerError(str!("Failed to load topic"))
            }
        })?;

    // Check permissions based on user role
    match user_role {
        AuthInfoUserRole::Teacher => {
            // Teacher can only update topics they created
            if topic.user_id != user_id {
                return Err(ApiError::Forbidden);
            }

            // For teacher updates, we need to ensure the request is Teacher type
            if req.topic_review_status.is_some() {
                return Err(ApiError::BadRequest(str!(
                    "Teachers cannot update review status"
                )));
            }

            conn.build_transaction().read_write().run(|conn| {
                // Build update fields using a tuple approach
                let mut update_fields: Vec<
                    Box<dyn diesel::query_builder::QueryFragment<diesel::pg::Pg> + Send>,
                > = Vec::new();

                if let Some(topic_name) = &req.topic_name {
                    update_fields.push(Box::new(schema::topic::columns::topic_name.eq(topic_name)));
                }
                if let Some(topic_description) = &req.topic_description {
                    update_fields.push(Box::new(
                        schema::topic::columns::topic_description.eq(topic_description),
                    ));
                }
                if let Some(topic_max_students) = req.topic_max_students {
                    update_fields.push(Box::new(
                        schema::topic::columns::topic_max_students.eq(topic_max_students),
                    ));
                }
                if let Some(topic_type) = req.topic_type {
                    update_fields.push(Box::new(
                        schema::topic::columns::topic_type.eq(topic_type as i16),
                    ));
                }

                // When teacher updates, set review status to Pending
                update_fields.push(Box::new(
                    schema::topic::columns::topic_review_status
                        .eq(TopicReviewStatus::Pending as i16),
                ));

                // The update_fields vector is no longer used with the new approach
                // We'll keep it declared but unused to avoid changing too much code structure

                // Instead, let's use a simpler approach with multiple update statements
                if let Some(topic_name) = &req.topic_name {
                    diesel::update(schema::topic::dsl::topic.find(*topic_id))
                        .set(schema::topic::columns::topic_name.eq(topic_name))
                        .execute(conn)
                        .map_err(|e| {
                            ApiError::InternalServerError(format!(
                                "Failed to update topic name: {}",
                                e
                            ))
                        })?;
                }
                if let Some(topic_description) = &req.topic_description {
                    diesel::update(schema::topic::dsl::topic.find(*topic_id))
                        .set(schema::topic::columns::topic_description.eq(topic_description))
                        .execute(conn)
                        .map_err(|e| {
                            ApiError::InternalServerError(format!(
                                "Failed to update topic description: {}",
                                e
                            ))
                        })?;
                }
                if let Some(topic_max_students) = req.topic_max_students {
                    diesel::update(schema::topic::dsl::topic.find(*topic_id))
                        .set(schema::topic::columns::topic_max_students.eq(topic_max_students))
                        .execute(conn)
                        .map_err(|e| {
                            ApiError::InternalServerError(format!(
                                "Failed to update max students: {}",
                                e
                            ))
                        })?;
                }
                if let Some(topic_type) = req.topic_type {
                    diesel::update(schema::topic::dsl::topic.find(*topic_id))
                        .set(schema::topic::columns::topic_type.eq(topic_type as i16))
                        .execute(conn)
                        .map_err(|e| {
                            ApiError::InternalServerError(format!(
                                "Failed to update topic type: {}",
                                e
                            ))
                        })?;
                }

                // Always set review status to Pending when teacher updates
                diesel::update(schema::topic::dsl::topic.find(*topic_id))
                    .set(
                        schema::topic::columns::topic_review_status
                            .eq(TopicReviewStatus::Pending as i16),
                    )
                    .execute(conn)
                    .map_err(|e| {
                        ApiError::InternalServerError(format!(
                            "Failed to update review status: {}",
                            e
                        ))
                    })?;

                Ok::<(), ApiError>(())
            })?;
        }
        AuthInfoUserRole::Office => {
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

                diesel::update(schema::topic::dsl::topic.find(*topic_id))
                    .set(schema::topic::columns::topic_review_status.eq(topic_review_status as i16))
                    .execute(&mut conn)
                    .map_err(|e| {
                        ApiError::InternalServerError(format!(
                            "Failed to update topic review status: {}",
                            e
                        ))
                    })?;
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
    }

    let updated_topic = schema::topic::dsl::topic
        .find(*topic_id)
        .inner_join(schema::teacher::table)
        .inner_join(schema::major::table)
        .first::<(Topic, Teacher, Major)>(&mut conn)
        .map_err(|e| {
            ApiError::InternalServerError(format!("Failed to load updated topic: {}", e))
        })?;

    let (topic, teacher, major) = updated_topic;

    let current_student_count: i64 = schema::student::dsl::student
        .filter(schema::student::columns::topic_id.eq(topic.topic_id))
        .count()
        .get_result(&mut conn)
        .map_err(|_| ApiError::InternalServerError(str!("Failed to count students for topic")))?;

    // Build response
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
