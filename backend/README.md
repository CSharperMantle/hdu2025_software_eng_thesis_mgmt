# thesis_mgmt/backend

毕业设计管理系统后端

## Quirks

* 暂时没有为管理 `Major` 编写接口，现在只能在数据库里面直接创建；
* [../docs/api/thesis_mgmt.yml](../docs/api/thesis_mgmt.yml) 里面还有一些 TODO；
* 考虑把测试数据放到 migrations 里面自动创建？
  * PowerDesigner 有测试数据生成功能，待试验

```sql
INSERT INTO major(major_id, major_name)
VALUES
    (0, '计算机科学与技术'),
    (1, '软件工程'),
    (2, '人工智能');

-- admin/admin
INSERT INTO sysuser(user_id, user_login, user_password_hash, user_password_salt, user_avatar)
VALUES
    (1, 'admin', decode('F6879937A648379E2B17BD3EA45D967DF40508B265483B118AB2D1D0ED151303', 'hex'), decode('8DBD7DE6437F6C0868E56BEBBB5DE4C6', 'hex'), NULL);
```
