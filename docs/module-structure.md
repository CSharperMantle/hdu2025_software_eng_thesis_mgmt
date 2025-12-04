# 毕业设计管理系统 - 模块结构图

## 模块结构图

```mermaid
graph TB
    Root[毕业设计管理系统]
    
    Root --> UIM[用户信息管理]
    Root --> TS[教师端]
    Root --> SS[学生端]
    Root --> AO[教科办端]
    Root --> DG[答辩组端]
    
    UIM --> UIM1[用户认证]
    UIM --> UIM2[权限管理]
    UIM --> UIM3[用户信息维护]
    
    TS --> TS1[题目管理]
    TS --> TS2[学生管理]
    TS --> TS3[论文评审]
    TS --> TS4[答辩评分]
    
    SS --> SS1[选题申请]
    SS --> SS2[材料提交]
    SS --> SS3[进度查询]
    SS --> SS4[成绩查询]
    
    AO --> AO1[流程管理]
    AO --> AO2[师生管理]
    AO --> AO3[题目审核]
    AO --> AO4[数据统计]
    AO --> AO5[答辩安排]
    
    DG --> DG1[答辩名单]
    DG --> DG2[答辩记录]
    DG --> DG3[成绩评定]
    DG --> DG4[结果提交]
    
    style Root fill:#e1f5ff
    style UIM fill:#fff4e1
    style TS fill:#ffe1f0
    style SS fill:#e1ffe1
    style AO fill:#f0e1ff
    style DG fill:#ffe1e1
```

## 模块说明

| 模块 | 负责人 | 主要功能 |
|------|--------|----------|
| 用户信息管理 | @woshiluo | 用户认证、权限管理、用户信息维护 |
| 教师端 | @CSharperMantle | 题目管理、学生管理、论文评审、答辩评分 |
| 学生端 | @Renxiejing | 选题申请、材料提交、进度查询、成绩查询 |
| 教科办端 | @woshiluo | 流程管理、师生管理、题目审核、数据统计、答辩安排 |
| 答辩组端 | @cbhuo123456-create | 答辩名单、答辩记录、成绩评定、结果提交 |
