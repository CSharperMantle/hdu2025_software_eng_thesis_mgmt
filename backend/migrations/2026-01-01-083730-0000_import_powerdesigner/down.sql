-- This file should undo anything in `up.sql`

/*==============================================================*/
/* DBMS name:      PostgreSQL 9.x                               */
/* Created on:     2026-01-01 16:50:20                          */
/*==============================================================*/

-- Drop tables in reverse order of creation to respect foreign key constraints
-- First drop tables that depend on others
drop table AssignmentRequest;
drop table FinalDefense;
drop table ProgressReport;
drop table Student;
drop table Topic;

-- Then drop tables that only depend on SysUser
drop table DefenseBoard;
drop table Office;
drop table SysAdmin;
drop table Teacher;

-- Drop the remaining tables
drop table Major;
drop table SysUser;
