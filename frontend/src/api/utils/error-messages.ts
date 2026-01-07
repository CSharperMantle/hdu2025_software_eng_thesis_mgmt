// Error messages based on API specification for each endpoint

export const ERROR_MESSAGES = {
  // Login errors
  login: {
    400: '请求体格式错误',
    401: '认证失败，用户名或密码错误',
  },

  // Logout errors
  logout: {
    401: '未登录',
  },

  // User operations errors
  user: {
    get: {
      401: '未登录',
    },
    patch: {
      400: '请求体格式错误',
      401: '未登录',
    },
    post: {
      400: '请求体格式错误',
      401: '未登录',
      403: '权限不足',
      409: '用户创建失败，同名用户已存在',
    },
  },

  // Topics errors
  topics: {
    get: {
      401: '未登录',
      403: '权限不足',
    },
    post: {
      400: '请求体格式错误',
      401: '未登录',
      403: '权限不足',
    },
    getById: {
      401: '未登录',
      403: '权限不足',
      404: '未找到指定课题',
    },
    patch: {
      400: '请求体格式错误',
      401: '未登录',
      403: '权限不足',
      404: '未找到指定课题',
    },
    search: {
      400: '请求体格式错误',
      401: '未登录',
      403: '权限不足',
    },
  },

  // Assignments errors
  assignments: {
    get: {
      401: '未登录',
      403: '权限不足',
    },
    post: {
      400: '请求体格式错误',
      401: '未登录',
      403: '权限不足',
      409: '选题申请失败，课题已满或已存在申请记录',
    },
    patch: {
      400: '请求体格式错误',
      401: '未登录',
      403: '权限不足',
      404: '未找到指定选题申请或学生',
      409: '选题申请状态更新失败，学生已有课题或名额已满',
    },
  },

  // Progress reports errors
  progressReports: {
    get: {
      401: '未登录',
      403: '权限不足',
      404: '无当前进展',
    },
    post: {
      400: '请求体格式错误',
      401: '未登录',
      403: '权限不足',
      404: '未找到指定进展',
      409: '进展更新失败，状态存在冲突',
    },
    patch: {
      400: '请求体格式错误',
      401: '未登录',
      403: '权限不足',
      404: '未找到指定进展',
      409: '进展更新失败，状态存在冲突',
    },
  },

  // Final defenses errors
  finalDefenses: {
    get: {
      401: '未登录',
      403: '权限不足',
      404: '无结项答辩进展',
    },
    post: {
      401: '未登录',
      403: '权限不足',
      409: '结项答辩申请提交失败，状态存在冲突',
    },
    patch: {
      400: '请求体格式错误',
      401: '未登录',
      403: '权限不足',
      404: '未找到指定结项答辩进展',
      409: '结项答辩进展更新失败，状态存在冲突',
    },
  },

  // Default error messages
  default: {
    400: '请求格式错误',
    401: '认证失败',
    403: '权限不足',
    404: '资源不存在',
    409: '操作冲突',
    422: '数据验证失败',
    500: '服务器内部错误',
    502: '网关错误',
    503: '服务暂时不可用',
  },
} as const

export function getErrorMessage (endpoint: string, statusCode?: number): string {
  if (!statusCode) {
    return '网络错误，请检查网络连接'
  }

  // Parse endpoint path to get message mapping
  const parts = endpoint.split('.')
  let messages: any = ERROR_MESSAGES

  for (const part of parts) {
    if (messages[part]) {
      messages = messages[part]
    } else {
      messages = ERROR_MESSAGES.default
      break
    }
  }

  // Get specific message or fallback to default
  const message
    = (messages as Record<number, string>)[statusCode]
      || (ERROR_MESSAGES.default as Record<number, string>)[statusCode]

  return message || '操作失败，请稍后重试'
}
