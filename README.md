# thesis_mgmt

(2025-2026-1)-S0512280-09 《软件工程课程设计》课程项目：高校毕业设计管理系统

## 项目结构

* [docs/](docs/)：文档性内容
  * [pd/](docs/pd/)：PowerDesigner 数据库工程文件
  * [progress/](docs/progress/)：过程性内容
  * [api/](docs/api/)：前后端接口文档
* [scripts/](scripts/)：辅助脚本，如 Git Hook 等
* [frontend/](frontend/)：前端代码
* [backend/](backend/)：后端代码

## 部署

### Docker 部署（推荐）

使用 Docker Compose 快速部署完整应用：

```powershell
# 1. 初始化环境配置
Copy-Item .env.example .env
Copy-Item frontend\.env.example frontend\.env.local

# 2. 编辑 .env 配置数据库密码和其他设置

# 3. （可选）创建初始化 SQL 脚本
Copy-Item init.sql.example init.sql
# 编辑 init.sql 添加初始管理员用户

# 4. 构建并启动服务
docker-compose build
docker-compose up -d

# 5. 查看日志
docker-compose logs -f
```

应用将在 http://localhost:8080 上运行。

更多 Docker 相关命令和配置，请参阅 [DOCKER.md](DOCKER.md)。

### 手动部署

参见各子项目的 README：
- [frontend/README.md](frontend/README.md)
- [backend/README.md](backend/README.md)

## 提交规范

### Git 设置

```console
$ git config --local core.autocrlf 'false'
$ cp scripts/git-hooks-pre-commit .git/hooks/pre-commit
$ chmod +x .git/hooks/pre-commit
```

### 提交消息格式

采用[Conventional Commits](https://www.conventionalcommits.org/zh-hans/v1.0.0/#%E6%A6%82%E8%BF%B0)规范，每个提交标题（subject）和正文（body）使用以下格式用英文提交：

```plain-text
<提交类别>[组件]: <一句话提交说明，首字母小写>

[提交正文]
```

例如：

```plain-text
docs(progress): add weekly report for week 9

Summary:
* Finish project design.
* Start coding.

Co-authored-by: Someone <someone@company.com>
```

#### 已定义的类别

* `docs`：文档类更改
* `feat`：新功能
* `fix`：问题修复
* `infra`：基础设施修改
* `chore`：无法归类的其他内容

### 提交守则

1. **不要使用`git push --force`。** 使用`git push --force-with-lease`；
2. 使用`git rebase`或`git cherry-pick`保持线性历史，不允许引入merge commit。
