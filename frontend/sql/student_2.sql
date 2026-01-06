-- Create a student user (20001/10003 as login/password)
INSERT INTO sysuser(user_login, user_password_hash, user_password_salt, user_avatar)
VALUES
    (
        '20002',
        decode('51FE98F2DDC4784E44268BBB716253B4C04EDD5370F4F519191AF8C5B4419328', 'hex'),
        decode('4B71318C5BF52C622665CEB1DA686C29', 'hex'),
        NULL
    );

-- Create the student entry
-- Assuming the user_id will be auto-generated (e.g., 5 if continuing from the example)
-- and the student is in major 2 (软件工程)
INSERT INTO student(user_id, topic_id, major_id, student_name, assn_time)
VALUES
    (
        (SELECT user_id FROM sysuser WHERE user_login = '20002'),
        NULL,  -- No topic assigned yet
        2,     -- Major: 软件工程
        '王大明',
        NULL   -- No assignment time yet
    );

