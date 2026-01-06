<template>
  <AppBar />
  <TeacherDrawer :part="currentPart" />

  <UserInfoBar role="teacher" :user-info="userInfo" />

  <div class="d-flex flex-column pa-4">
    <v-card class="w-100">
      <template #title>
        <span class="font-weight-black">选题申请审核</span>
      </template>

      <v-card-text>
        <v-data-table-server
          v-model:items-per-page="itemsPerPage"
          v-model:page="page"
          :headers="headers"
          item-value="student_id"
          :items="assignments"
          :items-length="totalItems"
          :items-per-page-options="[5, 10, 15, 20]"
          :loading="loading"
          @update:options="loadAssignments"
        >
          <template #item.request_time="{ item }">
            {{ formatDateTime(item.request_time) }}
          </template>

          <template #item.status="{ item }">
            <v-chip :color="getAssignmentStatusColor(item.status)" size="small">
              {{ getAssignmentStatusName(item.status) }}
            </v-chip>
          </template>

          <template #item.actions="{ item }">
            <div v-if="item.status === 0" class="d-flex gap-2">
              <v-btn
                color="success"
                icon="mdi-check"
                size="small"
                variant="text"
                @click="approveAssignment(item.student_id, item.topic_id, true)"
              />
              <v-btn
                color="error"
                icon="mdi-close"
                size="small"
                variant="text"
                @click="approveAssignment(item.student_id, item.topic_id, false)"
              />
            </div>
            <span v-else class="text-grey">-</span>
          </template>
        </v-data-table-server>
      </v-card-text>
    </v-card>
  </div>

  <v-snackbar v-model="snackbar.show" :color="snackbar.color" :timeout="3000">
    {{ snackbar.message }}
  </v-snackbar>
</template>

<script lang="ts" setup>
import type { Assignment, UserGetResponse } from '@/api'
import { onMounted, ref } from 'vue'
import { ASSIGNMENT_STATUS_MAP, createApiClient, getErrorMessage } from '@/api'
import { API_BASE_URL } from '@/config'

const currentPart = 1 // Assignment review is part 1 in TeacherDrawer
const userInfo = ref<UserGetResponse | null>(null)
const assignments = ref<Assignment[]>([])
const loading = ref(false)
const page = ref(1)
const itemsPerPage = ref(20)
const totalItems = ref(0)

const snackbar = ref({
  show: false,
  message: '',
  color: 'success',
})

const headers = [
  { title: '学生姓名', key: 'student_name', sortable: false },
  { title: '学号', key: 'student_id', sortable: false },
  { title: '专业', key: 'student_major', sortable: false },
  { title: '课题名称', key: 'topic_name', sortable: false },
  { title: '申请时间', key: 'request_time', sortable: false },
  { title: '状态', key: 'status', sortable: false },
  { title: '操作', key: 'actions', sortable: false },
]

const apiClient = createApiClient(API_BASE_URL)

async function fetchUserInfo() {
  try {
    userInfo.value = await apiClient.auth.getCurrentUser()
  } catch (error) {
    console.error('Failed to fetch user info:', error)
  }
}

async function loadAssignments() {
  loading.value = true
  try {
    const response = await apiClient.assignments.getAssignments({
      page: page.value,
      page_size: itemsPerPage.value,
    })
    assignments.value = response.assignments
    totalItems.value = response.total
  } catch (error: any) {
    console.error('Failed to load assignments:', error)
    snackbar.value = {
      show: true,
      message: '加载选题申请失败',
      color: 'error',
    }
  } finally {
    loading.value = false
  }
}

function formatDateTime(dateTime: string): string {
  const date = new Date(dateTime)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function getAssignmentStatusName(status: number): string {
  return ASSIGNMENT_STATUS_MAP.get(status as 0 | 1 | 2) || '未知'
}

function getAssignmentStatusColor(status: number): string {
  const colors = { 0: 'warning', 1: 'success', 2: 'error' }
  return colors[status as 0 | 1 | 2] || 'default'
}

async function approveAssignment(studentId: number, topicId: number, approved: boolean) {
  try {
    await apiClient.assignments.updateAssignmentStatus(studentId, topicId, { approved })
    snackbar.value = {
      show: true,
      message: approved ? '已批准选题申请' : '已拒绝选题申请',
      color: 'success',
    }
    // Reload assignments
    await loadAssignments()
  } catch (error: any) {
    console.error('Failed to update assignment status:', error)
    snackbar.value = {
      show: true,
      message: getErrorMessage('assignment', error.statusCode),
      color: 'error',
    }
  }
}

onMounted(() => {
  fetchUserInfo()
})
</script>
