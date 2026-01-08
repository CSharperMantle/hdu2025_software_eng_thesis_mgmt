# thesis_mgmt

(2025-2026-1)-S0512280-09 《软件工程课程设计》课程项目：高校毕业设计管理系统

## 项目结构

* [docs/](docs/)：文档性内容
  * [pd/](docs/pd/)：PowerDesigner 数据库工程文件
  * [progress/](docs/progress/)：周报、每周提交材料等过程性内容
  * [api/](docs/api/)：前后端接口文档
* [scripts/](scripts/)：辅助脚本，如 Git Hook 等
* [frontend/](frontend/)：前端代码
* [backend/](backend/)：后端代码
* [docker/](docker/)：部署用脚本

## 任务管理

<https://github.com/users/CSharperMantle/projects/5>

## 部署

使用 [Docker Compose](https://docs.docker.com/compose/) 部署。

```console
$ # Edit deployment configs
$ vim docker-compose.yml

$ # Edit init data
$ vim docker/init.sql

$ docker compose up
```

## 提交规范

### Git 设置

```console
$ git config --local core.autocrlf 'false'
$ cp scripts/git-hooks-pre-commit .git/hooks/pre-commit
$ chmod +x .git/hooks/pre-commit
```

### 提交消息格式

采用 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/v1.0.0/#%E6%A6%82%E8%BF%B0) 规范，每个提交标题（subject）和正文（body）使用以下格式用英文提交：

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

1. **不要使用 `git push --force`。** 使用 `git push --force-with-lease`；
2. 使用 `git rebase` 或 `git cherry-pick` 保持线性历史，不允许引入merge commit。

## 许可协议

### 源代码

Copyright &copy; 2025-2026 Rong Bao <<rong.bao@csmantle.top>>.

Copyright &copy; 2025-2026 Bo Chu.

Copyright &copy; 2025-2026 Woshiluo Luo <<woshiluo.luo@outlook.com>>.

Copyright &copy; 2025-2026 Xiejing Ren <<760540083@qq.com>>.

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but **WITHOUT ANY WARRANTY**; without even the implied warranty of **MERCHANTABILITY** or **FITNESS FOR A PARTICULAR PURPOSE**. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see [LICENSE-GPL-3.0-or-later](LICENSE-GPL-3.0-or-later) or <https://www.gnu.org/licenses/>.

### 设计笔记与文档

Copyright &copy; 2025-2026 Rong Bao <<rong.bao@csmantle.top>>.

Copyright &copy; 2025-2026 Bo Chu.

Copyright &copy; 2025-2026 Woshiluo Luo <<woshiluo.luo@outlook.com>>.

Copyright &copy; 2025-2026 Xiejing Ren <<760540083@qq.com>>.

This work is licensed under Creative Commons Attribution-ShareAlike 4.0 International. To view a copy of this license, see [LICENSE-CC-BY-SA-4.0](LICENSE-CC-BY-SA-4.0) or visit <https://creativecommons.org/licenses/by-sa/4.0/>.
