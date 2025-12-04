# thesis_mgmt

(2025-2026-1)-S0512280-09 《软件工程课程设计》课程项目：高校毕业设计管理系统

## 项目结构

* [docs/](docs/)：文档性内容
  * [module-structure.md](docs/module-structure.md)：系统模块结构图
  * [progress/](docs/progress/)：过程性内容
* *TBA*

## 提交规范

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
