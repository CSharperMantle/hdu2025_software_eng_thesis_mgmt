<template>
  <AppBar />
  <StudentDrawer :part="currentPart" />

  <UserInfoBar :user-info="userInfo" role="student" />
  <div class="d-flex flex-column pa-4">
    <v-card class="w-100">
      <template #title>
        <span class="font-weight-black">可选课题列表</span>
      </template>

      <v-card-text>
        <v-text-field
          v-model="search"
          label="搜索课题名称或描述"
          prepend-inner-icon="mdi-magnify"
          clearable
          class="mb-4"
          @update:model-value="onSearchChange"
        />

        <v-data-table-server
          v-model:items-per-page="itemsPerPage"
          v-model:page="page"
          :headers="headers"
          :items="topics"
          :items-length="totalItems"
          :loading="loading"
          :items-per-page-options="[5, 10, 15, 20]"
          item-value="topic_id"
          @update:options="loadTopics"
        >
          <template #item.topic_type="{ item }">
            {{ getTopicTypeName(item.topic_type) }}
          </template>

          <template #item.availability="{ item }">
            {{ item.current_student_count }} / {{ item.topic_max_students }}
          </template>

          <template #item.actions="{ item }">
            <v-btn
              color="primary"
              size="small"
              :disabled="item.current_student_count >= item.topic_max_students || isTopicSelected(item.topic_id)"
              @click="selectTopic(item.topic_id)"
            >
              {{ isTopicSelected(item.topic_id) ? '已选择' : '选择' }}
            </v-btn>
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
import { onMounted, ref } from 'vue'
import { createApiClient, getErrorMessage, getTopicTypeName } from '@/api'
import type { TopicBrief, UserGetResponse } from '@/api'
import { API_BASE_URL } from '@/config'

const currentPart = 0
const userInfo = ref<UserGetResponse | null>(null)
const topics = ref<TopicBrief[]>([])
const selectedTopicIds = ref<Set<number>>(new Set())
const loading = ref(false)
const page = ref(1)
const itemsPerPage = ref(20)
const totalItems = ref(0)
const search = ref('')

const snackbar = ref({
	show: false,
	message: '',
	color: 'success',
})

const headers = [
	{ title: '课题名称', key: 'topic_name', sortable: false },
	{ title: '指导教师', key: 'teacher_name', sortable: false },
	{ title: '课题类型', key: 'topic_type', sortable: false },
	{ title: '名额', key: 'availability', sortable: false },
	{ title: '操作', key: 'actions', sortable: false },
]

const apiClient = createApiClient(API_BASE_URL)

async function fetchUserInfo() {
	try {
		userInfo.value = await apiClient.auth.getCurrentUser()
	}
	catch (error) {
		console.error('Failed to fetch user info:', error)
	}
}

async function loadAssignments() {
	try {
		const response = await apiClient.assignments.getAssignments()
		// Track all selected topic IDs from assignments
		selectedTopicIds.value = new Set(
			response.assignments.map(assignment => assignment.topic_id),
		)
	}
	catch (error: any) {
		console.error('Failed to load assignments:', error)
	}
}

async function loadTopics() {
	loading.value = true
	try {
		const params = {
			page: page.value,
			page_size: itemsPerPage.value,
		}

		const response = search.value
			? await apiClient.topics.searchTopics(search.value, params)
			: await apiClient.topics.getTopics(params)

		topics.value = response.topics
		totalItems.value = response.total
	}
	catch (error: any) {
		console.error('Failed to load topics:', error)
		snackbar.value = {
			show: true,
			message: '加载课题列表失败',
			color: 'error',
		}
	}
	finally {
		loading.value = false
	}
}

function onSearchChange() {
	// Reset to first page when search changes
	page.value = 1
	loadTopics()
}

function isTopicSelected(topicId: number): boolean {
	return selectedTopicIds.value.has(topicId)
}

async function selectTopic(topicId: number) {
	try {
		await apiClient.assignments.createAssignment({ topic_id: topicId })
		snackbar.value = {
			show: true,
			message: '选题申请已提交',
			color: 'success',
		}
		// Reload assignments and topics to update availability
		await Promise.all([loadAssignments(), loadTopics()])
	}
	catch (error: any) {
		console.error('Failed to select topic:', error)
		snackbar.value = {
			show: true,
			message: getErrorMessage('assignment', error.statusCode),
			color: 'error',
		}
	}
}

onMounted(() => {
	fetchUserInfo()
	loadAssignments()
})
</script>
