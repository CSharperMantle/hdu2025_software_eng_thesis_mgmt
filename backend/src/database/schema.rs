// @generated automatically by Diesel CLI.

diesel::table! {
    assignmentrequest (student_user_name, topic_id) {
        #[max_length = 16]
        student_user_name -> Varchar,
        topic_id -> Int4,
        assn_req_time -> Timestamptz,
    }
}

diesel::table! {
    defenseboard (user_name) {
        #[max_length = 16]
        user_name -> Varchar,
    }
}

diesel::table! {
    finaldefense (final_def_id) {
        final_def_id -> Int4,
        topic_id -> Int4,
        #[max_length = 16]
        student_user_name -> Varchar,
        #[max_length = 16]
        def_board_user_name -> Nullable<Varchar>,
        final_def_time -> Timestamptz,
        final_def_attachment -> Text,
        final_def_outcome -> Nullable<Bool>,
        final_def_comment -> Nullable<Text>,
        final_def_grade -> Nullable<Numeric>,
    }
}

diesel::table! {
    major (major_id) {
        major_id -> Int4,
        #[max_length = 16]
        major_name -> Varchar,
    }
}

diesel::table! {
    office (user_name) {
        #[max_length = 16]
        user_name -> Varchar,
    }
}

diesel::table! {
    progressreport (prog_report_id) {
        prog_report_id -> Int4,
        topic_id -> Int4,
        #[max_length = 16]
        student_user_name -> Varchar,
        prog_report_type -> Int2,
        prog_report_time -> Timestamptz,
        prog_report_attachment -> Text,
        prog_report_outcome -> Int2,
        prog_report_comment -> Nullable<Text>,
        prog_report_grade -> Nullable<Numeric>,
    }
}

diesel::table! {
    student (user_name) {
        #[max_length = 16]
        user_name -> Varchar,
        topic_id -> Nullable<Int4>,
        major_id -> Int4,
        #[max_length = 16]
        student_name -> Varchar,
        assn_time -> Timestamptz,
    }
}

diesel::table! {
    sysadmin (user_name) {
        #[max_length = 16]
        user_name -> Varchar,
    }
}

diesel::table! {
    sysuser (user_name) {
        #[max_length = 16]
        user_name -> Varchar,
        user_password_hash -> Bytea,
        user_password_salt -> Bytea,
        user_avatar -> Nullable<Text>,
    }
}

diesel::table! {
    teacher (user_name) {
        #[max_length = 16]
        user_name -> Varchar,
        #[max_length = 16]
        teacher_name -> Varchar,
    }
}

diesel::table! {
    topic (topic_id) {
        topic_id -> Int4,
        major_id -> Int4,
        #[max_length = 16]
        teacher_user_name -> Varchar,
        #[max_length = 128]
        topic_name -> Varchar,
        topic_description -> Text,
        topic_max_students -> Int4,
        topic_type -> Int2,
        topic_review_status -> Int2,
    }
}

diesel::joinable!(assignmentrequest -> student (student_user_name));
diesel::joinable!(assignmentrequest -> topic (topic_id));
diesel::joinable!(defenseboard -> sysuser (user_name));
diesel::joinable!(finaldefense -> defenseboard (def_board_user_name));
diesel::joinable!(finaldefense -> student (student_user_name));
diesel::joinable!(finaldefense -> topic (topic_id));
diesel::joinable!(office -> sysuser (user_name));
diesel::joinable!(progressreport -> student (student_user_name));
diesel::joinable!(progressreport -> topic (topic_id));
diesel::joinable!(student -> major (major_id));
diesel::joinable!(student -> sysuser (user_name));
diesel::joinable!(student -> topic (topic_id));
diesel::joinable!(sysadmin -> sysuser (user_name));
diesel::joinable!(teacher -> sysuser (user_name));
diesel::joinable!(topic -> major (major_id));
diesel::joinable!(topic -> teacher (teacher_user_name));

diesel::allow_tables_to_appear_in_same_query!(
    assignmentrequest,
    defenseboard,
    finaldefense,
    major,
    office,
    progressreport,
    student,
    sysadmin,
    sysuser,
    teacher,
    topic,
);
