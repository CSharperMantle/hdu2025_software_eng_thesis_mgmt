<template>
  <AppBar />
  <StudentDrawer :part="currentPart" />

  <v-sheet class="w-100 d-flex justify-space-between pa-3 elevation-2 rounded-0">
    <span>
      你好, {{ userInfo?.name || userInfo?.username || '用户' }}
    </span>
    <span>
      工号: {{ userInfo?.username || '-' }}
    </span>
  </v-sheet>

  <div class="d-flex flex-column pa-4">
    <v-card class="w-100 mb-4">
      <template #title>
        <span class="font-weight-black">创建新课题</span>
      </template>

      <v-card-text>
        <v-form ref="formRef" @submit.prevent="createTopic">
          <v-select
            v-model="newTopic.major_id"
            :items="majors"
            item-title="name"
            item-value="id"
            label="专业"
            :rules="[v => !!v || '请选择专业']"
            required
          />

          <v-text-field
            v-model="newTopic.topic_name"
            label="课题名称"
            :rules="[v => !!v || '请输入课题名称']"
            required
          />

          <v-textarea
            v-model="newTopic.topic_description"
            label="课题描述"
            :rules="[v => !!v || '请输入课题描述']"
            required
          />

          <v-text-field
            v-model.number="newTopic.topic_max_students"
            label="最大学生数"
            type="number"
            :rules="[v => v > 0 || '最大学生数必须大于0']"
            required
          />

          <v-select
            v-model="newTopic.topic_type"
            :items="TOPIC_TYPES"
            item-title="name"
            item-value="value"
            label="课题类型"
            :rules="[v => v !== null && v !== undefined || '请选择课题类型']"
            required
          />

          <v-btn
            type="submit"
            color="primary"
            block
            class="mt-2"
          >
            创建课题
          </v-btn>
        </v-form>
      </v-card-text>
    </v-card>

    <v-card class="w-100">
      <template #title>
        <span class="font-weight-black">我的课题列表</span>
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

          <template #item.topic_review_status="{ item }">
            <v-chip :color="getTopicReviewStatusColor(item.topic_review_status)" size="small">
              {{ getTopicReviewStatusName(item.topic_review_status) }}
            </v-chip>
          </template>

          <template #item.availability="{ item }">
            {{ item.current_student_count }} / {{ item.topic_max_students }}
          </template>

          <template #item.actions="{ item }">
            <v-btn
              color="primary"
              size="small"
              variant="text"
              @click="viewTopic(item.topic_id)"
            >
              查看
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
import {
	TOPIC_TYPES,
	createApiClient,
	getErrorMessage,
	getTopicReviewStatusColor,
	getTopicReviewStatusName,
	getTopicTypeName,
} from '@/api'
import type { TopicBrief, TopicsPostRequest } from '@/api'
import { API_BASE_URL } from '@/config'

const currentPart = 0
const userInfo = ref<any>(null)
const topics = ref<TopicBrief[]>([])
const loading = ref(false)
const page = ref(1)
const itemsPerPage = ref(20)
const totalItems = ref(0)
const search = ref('')
const formRef = ref<any>(null)

const newTopic = ref<TopicsPostRequest>({
	major_id: null as any,
	topic_name: '',
	topic_description: '',
	topic_max_students: 1,
	topic_type: null as any,
})

const majors = [
	{ id: 1, name: '计算机科学与技术' },
	{ id: 2, name: '软件工程' },
	{ id: 3, name: '人工智能' },
]

const snackbar = ref({
	show: false,
	message: '',
	color: 'success',
})

const headers = [
	{ title: '课题名称', key: 'topic_name', sortable: false },
	{ title: '课题类型', key: 'topic_type', sortable: false },
	{ title: '审核状态', key: 'topic_review_status', sortable: false },
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
	page.value = 1
	loadTopics()
}

async function createTopic() {
	const { valid } = await formRef.value.validate()
	if (!valid)
		return

	try {
		await apiClient.topics.createTopic(newTopic.value)
		snackbar.value = {
			show: true,
			message: '课题创建成功，等待审核',
			color: 'success',
		}

		// Reset form
		newTopic.value = {
			major_id: null as any,
			topic_name: '',
			topic_description: '',
			topic_max_students: 1,
			topic_type: null as any,
		}
		formRef.value.reset()

		// Reload topics
		await loadTopics()
	}
	catch (error: any) {
		console.error('Failed to create topic:', error)
		snackbar.value = {
			show: true,
			message: getErrorMessage('topic', error.statusCode),
			color: 'error',
		}
	}
}

function viewTopic(topicId: number) {
	// TODO: Navigate to topic detail page or show dialog
	console.log('View topic:', topicId)
}

onMounted(() => {
	fetchUserInfo()
})
</script>
