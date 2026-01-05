-- Create an office account
-- Login: office / Password: office123 (placeholder - needs proper hash)

INSERT INTO sysuser(user_login, user_password_hash, user_password_salt, user_avatar)
VALUES
    (
        'board',
        decode('51FE98F2DDC4784E44268BBB716253B4C04EDD5370F4F519191AF8C5B4419328', 'hex'),
        decode('4B71318C5BF52C622665CEB1DA686C29', 'hex'),
        NULL
    );

-- Create the office entry
INSERT INTO defenseboard(user_id)
VALUES
    (
        (SELECT user_id FROM sysuser WHERE user_login = 'board')
    );
