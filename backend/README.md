# thesis_mgmt/backend

毕业设计管理系统后端

## Quirks

* 暂时没有为管理 `Major` 编写接口，现在只能在数据库里面直接创建；
* [../docs/api/thesis_mgmt.yml](../docs/api/thesis_mgmt.yml) 里面还有一些 TODO；
* 考虑把测试数据放到 migrations 里面自动创建？
  * PowerDesigner 有测试数据生成功能，待试验

```sql
INSERT INTO major(/* major_id, */ major_name)
VALUES
    ( /* 1, */ '计算机科学与技术'),
    ( /* 2, */ '软件工程'),
    ( /* 3, */ '人工智能');

INSERT INTO sysuser(/* user_id, */ user_login, user_password_hash, user_password_salt, user_avatar)
VALUES
    (
        /* admin/admin */
        /* 1, */
        'admin',
        decode('F6879937A648379E2B17BD3EA45D967DF40508B265483B118AB2D1D0ED151303', 'hex'),
        decode('8DBD7DE6437F6C0868E56BEBBB5DE4C6', 'hex'),
        NULL
    ), (
        /* 10001/10001 */
        /* 2, */
        '10001',
        decode('7B72F0BB1C482340B9644DD76018066C3A4293015695F9A70289A8452256E843', 'hex'),
        decode('3147790DD2635F71FAB00F471A00E747', 'hex'),
        NULL
    ), (
        /* 10002/10002 */
        /* 3, */
        '10002',
        decode('FB672B5F5AE210F50D3EB1930B4FD0C4CEF31542A4613F83BEA331C85FE03EFE', 'hex'),
        decode('DC8AED90B0D2AC6FE00A88DFE77126B5', 'hex'),
        NULL
    ), (
        /* 10003/10003 */
        /* 4, */
        '10003',
        decode('51FE98F2DDC4784E44268BBB716253B4C04EDD5370F4F519191AF8C5B4419328', 'hex'),
        decode('4B71318C5BF52C622665CEB1DA686C29', 'hex'),
        NULL
    );
INSERT INTO sysadmin(user_id)
VALUES
    (1);
INSERT INTO teacher(user_id, teacher_name)
VALUES
    (2, '张吉惟'),
    (3, '林国瑞'),
    (4, '林玟书');
```
