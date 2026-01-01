/*==============================================================*/
/* DBMS name:      PostgreSQL 9.x                               */
/* Created on:     2026-01-01 16:50:20                          */
/*==============================================================*/

/*==============================================================*/
/* Table: SysUser                                               */
/*==============================================================*/
create table SysUser (
   user_id              SERIAL not null,
   user_login           VARCHAR(16)          not null,
   user_password_hash   BYTEA                not null,
   user_password_salt   BYTEA                not null,
   user_avatar          BYTEA                null,
   constraint PK_SYSUSER primary key (user_id)
);

/*==============================================================*/
/* Index: User_PK                                               */
/*==============================================================*/
create unique index User_PK on SysUser (
user_id
);

/*==============================================================*/
/* Table: Major                                                 */
/*==============================================================*/
create table Major (
   major_id             INT4                 not null,
   major_name           VARCHAR(16)          not null,
   constraint PK_MAJOR primary key (major_id)
);

/*==============================================================*/
/* Index: Major_PK                                              */
/*==============================================================*/
create unique index Major_PK on Major (
major_id
);

/*==============================================================*/
/* Table: Teacher                                               */
/*==============================================================*/
create table Teacher (
   user_id              INT4                 not null,
   teacher_name         VARCHAR(16)          not null,
   constraint PK_TEACHER primary key (user_id),
   constraint FK_TEACHER_USERIS3_SYSUSER foreign key (user_id)
      references SysUser (user_id)
      on delete restrict on update restrict
);

/*==============================================================*/
/* Index: Teacher_PK                                            */
/*==============================================================*/
create unique index Teacher_PK on Teacher (
user_id
);

/*==============================================================*/
/* Table: SysAdmin                                              */
/*==============================================================*/
create table SysAdmin (
   user_id              INT4                 not null,
   constraint PK_SYSADMIN primary key (user_id),
   constraint FK_SYSADMIN_USERIS5_SYSUSER foreign key (user_id)
      references SysUser (user_id)
      on delete restrict on update restrict
);

/*==============================================================*/
/* Index: SysAdmin_PK                                           */
/*==============================================================*/
create unique index SysAdmin_PK on SysAdmin (
user_id
);

/*==============================================================*/
/* Table: Office                                                */
/*==============================================================*/
create table Office (
   user_id              INT4                 not null,
   constraint PK_OFFICE primary key (user_id),
   constraint FK_OFFICE_USERIS4_SYSUSER foreign key (user_id)
      references SysUser (user_id)
      on delete restrict on update restrict
);

/*==============================================================*/
/* Index: Office_PK                                             */
/*==============================================================*/
create unique index Office_PK on Office (
user_id
);

/*==============================================================*/
/* Table: DefenseBoard                                          */
/*==============================================================*/
create table DefenseBoard (
   user_id              INT4                 not null,
   constraint PK_DEFENSEBOARD primary key (user_id),
   constraint FK_DEFENSEB_USERIS2_SYSUSER foreign key (user_id)
      references SysUser (user_id)
      on delete restrict on update restrict
);

/*==============================================================*/
/* Index: DefenseBoard_PK                                       */
/*==============================================================*/
create unique index DefenseBoard_PK on DefenseBoard (
user_id
);

/*==============================================================*/
/* Table: Topic                                                 */
/*==============================================================*/
create table Topic (
   topic_id             INT4                 not null,
   major_id             INT4                 not null,
   user_id              INT4                 not null,
   topic_name           VARCHAR(128)         not null,
   topic_description    TEXT                 not null,
   topic_max_students   INT4                 not null,
   topic_type           INT2                 not null
      constraint CKC_TOPIC_TYPE_TOPIC check (topic_type in (0,1,2,3,4)),
   topic_review_status  INT2                 not null default 0
      constraint CKC_TOPIC_REVIEW_STAT_TOPIC check (topic_review_status in (0,1,2)),
   constraint PK_TOPIC primary key (topic_id),
   constraint FK_TOPIC_OPENTO_MAJOR foreign key (major_id)
      references Major (major_id)
      on delete restrict on update restrict,
   constraint FK_TOPIC_TUTORING_TEACHER foreign key (user_id)
      references Teacher (user_id)
      on delete restrict on update restrict
);

/*==============================================================*/
/* Index: Topic_PK                                              */
/*==============================================================*/
create unique index Topic_PK on Topic (
topic_id
);

/*==============================================================*/
/* Index: Tutoring_FK                                           */
/*==============================================================*/
create  index Tutoring_FK on Topic (
user_id
);

/*==============================================================*/
/* Index: OpenTo_FK                                             */
/*==============================================================*/
create  index OpenTo_FK on Topic (
major_id
);

/*==============================================================*/
/* Table: Student                                               */
/*==============================================================*/
create table Student (
   user_id              INT4                 not null,
   topic_id             INT4                 null,
   major_id             INT4                 not null,
   student_name         VARCHAR(16)          not null,
   assn_time            TIMESTAMP WITH TIME ZONE not null,
   constraint PK_STUDENT primary key (user_id),
   constraint FK_STUDENT_ASSIGNMEN_TOPIC foreign key (topic_id)
      references Topic (topic_id)
      on delete restrict on update restrict,
   constraint FK_STUDENT_MAJORIN_MAJOR foreign key (major_id)
      references Major (major_id)
      on delete restrict on update restrict,
   constraint FK_STUDENT_USERIS_SYSUSER foreign key (user_id)
      references SysUser (user_id)
      on delete restrict on update restrict
);

/*==============================================================*/
/* Index: Student_PK                                            */
/*==============================================================*/
create unique index Student_PK on Student (
user_id
);

/*==============================================================*/
/* Index: Assignment_FK                                         */
/*==============================================================*/
create  index Assignment_FK on Student (
topic_id
);

/*==============================================================*/
/* Index: MajorIn_FK                                            */
/*==============================================================*/
create  index MajorIn_FK on Student (
major_id
);

/*==============================================================*/
/* Table: AssignmentRequest                                     */
/*==============================================================*/
create table AssignmentRequest (
   user_id              INT4                 not null,
   topic_id             INT4                 not null,
   assn_req_time        TIMESTAMP WITH TIME ZONE not null,
   constraint PK_ASSIGNMENTREQUEST primary key (user_id, topic_id),
   constraint FK_ASSIGNME_ASSIGNMEN_TOPIC foreign key (topic_id)
      references Topic (topic_id)
      on delete restrict on update restrict,
   constraint FK_ASSIGNME_ASSIGNMEN_STUDENT foreign key (user_id)
      references Student (user_id)
      on delete restrict on update restrict
);

/*==============================================================*/
/* Index: AssignmentRequest_PK                                  */
/*==============================================================*/
create unique index AssignmentRequest_PK on AssignmentRequest (
user_id,
topic_id
);

/*==============================================================*/
/* Index: AssignmentRequest2_FK                                 */
/*==============================================================*/
create  index AssignmentRequest2_FK on AssignmentRequest (
user_id
);

/*==============================================================*/
/* Index: AssignmentRequest_FK                                  */
/*==============================================================*/
create  index AssignmentRequest_FK on AssignmentRequest (
topic_id
);

/*==============================================================*/
/* Table: ProgressReport                                        */
/*==============================================================*/
create table ProgressReport (
   prog_report_id         SERIAL             not null,
   topic_id               INT4               not null,
   user_id                INT4               not null,
   prog_report_type       INT2               not null
      constraint CKC_PROG_REPORT_TYPE_PROGRESS check (prog_report_type in (0,1)),
   prog_report_time       TIMESTAMP WITH TIME ZONE not null,
   prog_report_attachment BYTEA              not null,
   prog_report_outcome    INT2               not null default 0
      constraint CKC_PROG_REPORT_OUTCO_PROGRESS check (prog_report_outcome in (0,1,2)),
   prog_report_comment    TEXT               null,
   prog_report_grade      DECIMAL            null,
   constraint PK_PROGRESSREPORT primary key (prog_report_id),
   constraint FK_PROGRESS_AUTHORING_STUDENT foreign key (user_id)
      references Student (user_id)
      on delete restrict on update restrict,
   constraint FK_PROGRESS_PROGRESSR_TOPIC foreign key (topic_id)
      references Topic (topic_id)
      on delete restrict on update restrict
);

/*==============================================================*/
/* Index: ProgressReport_PK                                     */
/*==============================================================*/
create unique index ProgressReport_PK on ProgressReport (
prog_report_id
);

/*==============================================================*/
/* Index: AuthoringProgressReport_FK                            */
/*==============================================================*/
create  index AuthoringProgressReport_FK on ProgressReport (
user_id
);

/*==============================================================*/
/* Index: ProgressReportInvolving_FK                            */
/*==============================================================*/
create  index ProgressReportInvolving_FK on ProgressReport (
topic_id
);

/*==============================================================*/
/* Table: FinalDefense                                          */
/*==============================================================*/
create table FinalDefense (
   final_def_id         INT4                 not null,
   topic_id             INT4                 not null,
   user_id              INT4                 not null,
   Def_user_id          INT4                 not null,
   final_def_time       TIMESTAMP WITH TIME ZONE not null,
   final_def_attachment BYTEA                not null,
   final_def_outcome    BOOL                 null,
   final_def_comment    TEXT                 null,
   final_def_grade      DECIMAL              null,
   constraint PK_FINALDEFENSE primary key (final_def_id),
   constraint FK_FINALDEF_AUTHORING_STUDENT foreign key (user_id)
      references Student (user_id)
      on delete restrict on update restrict,
   constraint FK_FINALDEF_FINALDEFE_TOPIC foreign key (topic_id)
      references Topic (topic_id)
      on delete restrict on update restrict,
   constraint FK_FINALDEF_LISTENING_DEFENSEB foreign key (Def_user_id)
      references DefenseBoard (user_id)
      on delete restrict on update restrict
);

/*==============================================================*/
/* Index: FinalDefense_PK                                       */
/*==============================================================*/
create unique index FinalDefense_PK on FinalDefense (
final_def_id
);

/*==============================================================*/
/* Index: ListeningTo_FK                                        */
/*==============================================================*/
create  index ListeningTo_FK on FinalDefense (
Def_user_id
);

/*==============================================================*/
/* Index: AuthoringFinalDefense_FK                              */
/*==============================================================*/
create  index AuthoringFinalDefense_FK on FinalDefense (
user_id
);

/*==============================================================*/
/* Index: FinalDefenseInvolving_FK                              */
/*==============================================================*/
create  index FinalDefenseInvolving_FK on FinalDefense (
topic_id
);
