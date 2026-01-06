# thesis_mgmt/backend

毕业设计管理系统后端

## Quirks

* 暂时没有为管理 `Major` 编写接口，现在只能在数据库里面直接创建；
* [../docs/api/thesis_mgmt.yml](../docs/api/thesis_mgmt.yml) 里面还有一些 TODO；
* 考虑把测试数据放到 migrations 里面自动创建？
  * PowerDesigner 有测试数据生成功能，待试验
* GET `/topics/search?keyword=` 只检索标题，不检索内容
