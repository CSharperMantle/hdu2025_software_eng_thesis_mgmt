-- Create a student user (20001/10003 as login/password)
INSERT INTO sysuser(user_login, user_password_hash, user_password_salt, user_avatar)
VALUES
    (
        '20001',
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
        (SELECT user_id FROM sysuser WHERE user_login = '20001'),
        NULL,  -- No topic assigned yet
        2,     -- Major: 软件工程
        '王小明',
        NULL   -- No assignment time yet
    );

-- Create topics for the three existing teachers
-- Topics for teacher 张吉惟 (user_id = 2)
INSERT INTO topic(major_id, user_id, topic_name, topic_description, topic_max_students, topic_type, topic_review_status)
VALUES
    (
        1,  -- 计算机科学与技术
        2,  -- 张吉惟
        '基于深度学习的图像识别系统设计与实现',
        '本课题旨在研究深度学习在图像识别领域的应用，设计并实现一个高效的图像识别系统。学生需要掌握卷积神经网络的原理，使用主流深度学习框架进行模型训练和优化。',
        2,  -- 最多2个学生
        0,  -- Topic type (0 might represent regular topic)
        1   -- Review status (1 might represent approved)
    ),
    (
        2,  -- 软件工程
        2,  -- 张吉惟
        '分布式系统中的数据一致性研究',
        '研究分布式系统中的数据一致性问题，分析CAP理论和各种一致性协议，并在实际系统中实现和验证一致性算法。',
        1,
        0,
        1
    );

-- Topics for teacher 林国瑞 (user_id = 3)
INSERT INTO topic(major_id, user_id, topic_name, topic_description, topic_max_students, topic_type, topic_review_status)
VALUES
    (
        2,  -- 软件工程
        3,  -- 林国瑞
        '敏捷开发流程管理系统的设计与实现',
        '设计并实现一个支持敏捷开发的项目管理系统，包括需求管理、迭代规划、任务跟踪等功能。系统应支持多人协作和实时更新。',
        3,
        0,
        1
    ),
    (
        3,  -- 人工智能
        3,  -- 林国瑞
        '自然语言处理在智能客服中的应用',
        '研究自然语言处理技术在智能客服系统中的应用，实现基于意图识别和实体抽取的对话系统，提高客服效率和用户体验。',
        2,
        0,
        1
    );

-- Topics for teacher 林玟书 (user_id = 4)
INSERT INTO topic(major_id, user_id, topic_name, topic_description, topic_max_students, topic_type, topic_review_status)
VALUES
    (
        1,  -- 计算机科学与技术
        4,  -- 林玟书
        '区块链技术在供应链管理中的应用研究',
        '探索区块链技术在供应链管理中的应用场景，设计基于区块链的溯源系统，确保供应链信息的透明性和不可篡改性。',
        2,
        0,
        1
    ),
    (
        2,  -- 软件工程
        4,  -- 林玟书
        '微服务架构下的API网关设计与实现',
        '研究微服务架构中API网关的作用和实现方式，设计并实现一个高性能的API网关，支持路由、负载均衡、认证授权等功能。',
        2,
        0,
        1
    ),
    (
        3,  -- 人工智能
        4,  -- 林玟书
        '基于强化学习的游戏AI设计',
        '使用强化学习算法设计智能游戏AI，让AI通过自我对弈不断学习和提升策略。学生需要掌握Q-learning、DQN等强化学习算法。',
        1,
        0,
        1
    );
