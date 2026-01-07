POST /api/login 登入
POST /api/logout 删除
GET /api/user 获取用户信息
POST /api/user 更新用户信息

GET /api/topics 获取课题列表（教师，教科办，学生）
PUT /api/topic 创建新选题（教师）
GET /api/topic/search 课题搜索（教师，教科版，学生）
GET /api/topic/:id 获取课题
POST /api/topic/:id 修改课题（教师，教科版）

GET /api/assignments 获取选题列表 （教师）
PUT /api/assignment 创建选题（学生）
POST /api/assignment/:student_id/:topic_id 更新选题状态（教师）

GET /api/progress_report 获取开题报告和中期检查进度（学生，教师）
GET /api/final_defense 获取结项答辩进度（学生，教师，答辩组）
POST /api/progress_report/:id 更改过程报告信息（学生，教师）
POST /api/final_defense/:id 更改结项答辩（学生，教师，答辩组）
