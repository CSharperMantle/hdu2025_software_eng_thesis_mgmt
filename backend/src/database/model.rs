use bigdecimal::BigDecimal;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::sysuser)]
pub struct SysUser {
    pub user_id: i32,
    pub user_login: String,
    pub user_password_hash: Vec<u8>,
    pub user_password_salt: Vec<u8>,
    pub user_avatar: Option<Vec<u8>>,
}

#[derive(Queryable, Selectable, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::major)]
pub struct Major {
    pub major_id: i32,
    pub major_name: String,
}

#[derive(Queryable, Selectable, Associations, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[diesel(belongs_to(SysUser, foreign_key = user_id))]
#[diesel(table_name = crate::schema::teacher)]
pub struct Teacher {
    pub user_id: i32,
    pub teacher_name: String,
}

#[derive(Queryable, Selectable, Associations, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[diesel(belongs_to(SysUser, foreign_key = user_id))]
#[diesel(table_name = crate::schema::sysadmin)]
pub struct SysAdmin {
    pub user_id: i32,
}

#[derive(Queryable, Selectable, Associations, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[diesel(belongs_to(SysUser, foreign_key = user_id))]
#[diesel(table_name = crate::schema::office)]
pub struct Office {
    pub user_id: i32,
}

#[derive(Queryable, Selectable, Associations, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[diesel(belongs_to(SysUser, foreign_key = user_id))]
#[diesel(table_name = crate::schema::defenseboard)]
pub struct DefenseBoard {
    pub user_id: i32,
}

#[derive(Queryable, Selectable, Associations, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[diesel(belongs_to(Major, foreign_key = major_id))]
#[diesel(belongs_to(Teacher, foreign_key = user_id))]
#[diesel(table_name = crate::schema::topic)]
pub struct Topic {
    pub topic_id: i32,
    pub major_id: i32,
    pub user_id: i32,
    pub topic_name: String,
    pub topic_description: String,
    pub topic_max_students: i32,
    pub topic_type: i16,
    pub topic_review_status: i16,
}

#[derive(Queryable, Selectable, Associations, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[diesel(belongs_to(SysUser, foreign_key = user_id))]
#[diesel(belongs_to(Major, foreign_key = major_id))]
#[diesel(belongs_to(Topic, foreign_key = topic_id))]
#[diesel(table_name = crate::schema::student)]
pub struct Student {
    pub user_id: i32,
    pub topic_id: Option<i32>,
    pub major_id: i32,
    pub student_name: String,
    pub assn_time: chrono::NaiveDate,
}

#[derive(Queryable, Selectable, Associations, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[diesel(belongs_to(Student, foreign_key = user_id))]
#[diesel(belongs_to(Topic, foreign_key = topic_id))]
#[diesel(table_name = crate::schema::assignmentrequest)]
pub struct AssignmentRequest {
    pub user_id: i32,
    pub topic_id: i32,
    pub assn_req_time: chrono::NaiveDate,
}

#[derive(Queryable, Selectable, Associations, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[diesel(belongs_to(Student, foreign_key = user_id))]
#[diesel(belongs_to(Topic, foreign_key = topic_id))]
#[diesel(table_name = crate::schema::progressreport)]
pub struct ProgressReport {
    pub prog_report_id: i32,
    pub topic_id: i32,
    pub user_id: i32,
    pub prog_report_type: i16,
    pub prog_report_time: chrono::NaiveDate,
    pub prog_report_attachment: Vec<u8>,
    pub prog_report_outcome: i16,
    pub prog_report_comment: Option<String>,
    pub prog_report_grade: Option<BigDecimal>,
}

#[derive(Queryable, Selectable, Associations, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[diesel(belongs_to(Student, foreign_key = user_id))]
#[diesel(belongs_to(Topic, foreign_key = topic_id))]
#[diesel(belongs_to(DefenseBoard, foreign_key = def_user_id))]
#[diesel(table_name = crate::schema::finaldefense)]
pub struct FinalDefense {
    pub final_def_id: i32,
    pub topic_id: i32,
    pub user_id: i32,
    pub def_user_id: i32,
    pub final_def_time: chrono::NaiveDate,
    pub final_def_attachment: Vec<u8>,
    pub final_def_outcome: Option<bool>,
    pub final_def_comment: Option<String>,
    pub final_def_grade: Option<BigDecimal>,
}

// Insertable structs for creating new records

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::sysuser)]
pub struct NewSysUser<'a> {
    pub user_login: &'a str,
    pub user_password_hash: &'a [u8],
    pub user_password_salt: &'a [u8],
    pub user_avatar: Option<&'a [u8]>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::major)]
pub struct NewMajor<'a> {
    pub major_id: i32,
    pub major_name: &'a str,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::teacher)]
pub struct NewTeacher<'a> {
    pub user_id: i32,
    pub teacher_name: &'a str,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::sysadmin)]
pub struct NewSysAdmin {
    pub user_id: i32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::office)]
pub struct NewOffice {
    pub user_id: i32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::defenseboard)]
pub struct NewDefenseBoard {
    pub user_id: i32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::topic)]
pub struct NewTopic<'a> {
    pub topic_id: i32,
    pub major_id: i32,
    pub user_id: i32,
    pub topic_name: &'a str,
    pub topic_description: &'a str,
    pub topic_max_students: i32,
    pub topic_type: i16,
    pub topic_review_status: i16,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::student)]
pub struct NewStudent<'a> {
    pub user_id: i32,
    pub topic_id: Option<i32>,
    pub major_id: i32,
    pub student_name: &'a str,
    pub assn_time: chrono::NaiveDate,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::assignmentrequest)]
pub struct NewAssignmentRequest {
    pub user_id: i32,
    pub topic_id: i32,
    pub assn_req_time: chrono::NaiveDate,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::progressreport)]
pub struct NewProgressReport<'a> {
    pub topic_id: i32,
    pub user_id: i32,
    pub prog_report_type: i16,
    pub prog_report_time: chrono::NaiveDate,
    pub prog_report_attachment: &'a [u8],
    pub prog_report_outcome: i16,
    pub prog_report_comment: Option<&'a str>,
    pub prog_report_grade: Option<BigDecimal>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::finaldefense)]
pub struct NewFinalDefense<'a> {
    pub final_def_id: i32,
    pub topic_id: i32,
    pub user_id: i32,
    pub def_user_id: i32,
    pub final_def_time: chrono::NaiveDate,
    pub final_def_attachment: &'a [u8],
    pub final_def_outcome: Option<bool>,
    pub final_def_comment: Option<&'a str>,
    pub final_def_grade: Option<BigDecimal>,
}
