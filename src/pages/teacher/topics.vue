<template>
  <AppBar />
  <TeacherDrawer :part="currentPart" />

  <UserInfoBar :user-info="userInfo" role="teacher" />

  <div class="d-flex flex-column pa-4">
    <v-card class="w-100">
      <template #title>
        <span class="font-weight-black">我的课题列表</span>
      </template>

      <v-card-text>
        <div class="d-flex justify-end mb-4">
          <v-btn
            color="primary"
            prepend-icon="mdi-plus"
            @click="openCreateDialog"
          >
            创建课题
          </v-btn>
        </div>

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
              icon="mdi-pencil"
              size="small"
              variant="text"
              @click="viewTopic(item.topic_id)"
            />
          </template>
        </v-data-table-server>
      </v-card-text>
    </v-card>
  </div>

  <v-snackbar v-model="snackbar.show" :color="snackbar.color" :timeout="3000">
    {{ snackbar.message }}
  </v-snackbar>

  <v-dialog v-model="createDialogVisible" max-width="800">
    <v-card>
      <v-card-title class="d-flex justify-space-between align-center">
        <span class="text-h5">创建新课题</span>
        <v-btn icon="mdi-close" variant="text" @click="createDialogVisible = false" />
      </v-card-title>

      <v-card-text>
        <v-form ref="formRef">
          <v-select
            v-model="newTopic.major_id"
            :items="majors"
            item-title="name"
            item-value="id"
            label="专业"
            variant="outlined"
            :rules="[v => !!v || '请选择专业']"
          />

          <v-text-field
            v-model="newTopic.topic_name"
            label="课题名称"
            variant="outlined"
            :rules="[v => !!v || '请输入课题名称']"
          />

          <v-textarea
            v-model="newTopic.topic_description"
            label="课题描述"
            variant="outlined"
            :rules="[v => !!v || '请输入课题描述']"
            rows="5"
          />

          <v-text-field
            v-model.number="newTopic.topic_max_students"
            label="最大学生数"
            type="number"
            variant="outlined"
            :rules="[v => v > 0 || '最大学生数必须大于0']"
          />

          <v-select
            v-model="newTopic.topic_type"
            :items="TOPIC_TYPES"
            item-title="name"
            item-value="value"
            label="课题类型"
            variant="outlined"
            :rules="[v => v !== null && v !== undefined || '请选择课题类型']"
          />
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="createDialogVisible = false">
          取消
        </v-btn>
        <v-btn color="primary" @click="createTopic">
          创建
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="editDialogVisible" max-width="800">
    <v-card v-if="selectedTopic">
      <v-card-title class="d-flex justify-space-between align-center">
        <span class="text-h5">编辑课题</span>
        <v-btn icon="mdi-close" variant="text" @click="editDialogVisible = false" />
      </v-card-title>

      <v-card-text>
        <v-form ref="editFormRef">
          <v-row>
            <v-col cols="12">
              <v-text-field
                v-model="editForm.topic_name"
                label="课题名称"
                variant="outlined"
                :rules="[v => !!v || '请输入课题名称']"
              />
            </v-col>

            <v-col cols="12" md="6">
              <v-text-field
                :model-value="selectedTopic.major_name"
                label="专业"
                readonly
                variant="outlined"
              />
            </v-col>

            <v-col cols="12" md="6">
              <v-select
                v-model="editForm.topic_type"
                :items="TOPIC_TYPES"
                item-title="name"
                item-value="value"
                label="课题类型"
                variant="outlined"
                :rules="[v => v !== null && v !== undefined || '请选择课题类型']"
              />
            </v-col>

            <v-col cols="12" md="6">
              <div class="text-subtitle-2 text-grey mb-2">审核状态</div>
              <v-chip :color="getTopicReviewStatusColor(selectedTopic.topic_review_status)" size="small">
                {{ getTopicReviewStatusName(selectedTopic.topic_review_status) }}
              </v-chip>
            </v-col>

            <v-col cols="12" md="6">
              <v-text-field
                v-model.number="editForm.topic_max_students"
                label="最大学生数"
                type="number"
                variant="outlined"
                :rules="[v => v > 0 || '最大学生数必须大于0']"
              />
            </v-col>

            <v-col cols="12">
              <div class="text-subtitle-2 text-grey mb-2">当前选择人数</div>
              <div class="text-body-1">{{ selectedTopic.current_student_count }} / {{ selectedTopic.topic_max_students }}</div>
            </v-col>

            <v-col cols="12">
              <v-textarea
                v-model="editForm.topic_description"
                label="课题描述"
                variant="outlined"
                :rules="[v => !!v || '请输入课题描述']"
                rows="5"
              />
            </v-col>
          </v-row>
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="editDialogVisible = false">
          取消
        </v-btn>
        <v-btn color="primary" @click="saveChanges">
          保存
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
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
import type { TopicBrief, TopicDetails, TopicsPostRequest } from '@/api'
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
const createDialogVisible = ref(false)
const editDialogVisible = ref(false)
const selectedTopic = ref<TopicDetails | null>(null)
const editFormRef = ref<any>(null)
const editForm = ref({
	topic_name: '',
	topic_description: '',
	topic_max_students: 1,
	topic_type: null as any,
})

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

function openCreateDialog() {
	createDialogVisible.value = true
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
		createDialogVisible.value = false

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

async function viewTopic(topicId: number) {
	try {
		selectedTopic.value = await apiClient.topics.getTopicById(topicId)
		editForm.value = {
			topic_name: selectedTopic.value.topic_name,
			topic_description: selectedTopic.value.topic_description,
			topic_max_students: selectedTopic.value.topic_max_students,
			topic_type: selectedTopic.value.topic_type,
		}
		editDialogVisible.value = true
	}
	catch (error: any) {
		console.error('Failed to load topic details:', error)
		snackbar.value = {
			show: true,
			message: '加载课题详情失败',
			color: 'error',
		}
	}
}

async function saveChanges() {
	const { valid } = await editFormRef.value.validate()
	if (!valid)
		return

	try {
		await apiClient.topics.updateTopicAsTeacher(selectedTopic.value!.topic_id, editForm.value)
		snackbar.value = {
			show: true,
			message: '课题更新成功，等待重新审核',
			color: 'success',
		}
		editDialogVisible.value = false
		// Reload topics
		await loadTopics()
	}
	catch (error: any) {
		console.error('Failed to update topic:', error)
		snackbar.value = {
			show: true,
			message: getErrorMessage('topic', error.statusCode),
			color: 'error',
		}
	}
}

onMounted(() => {
	fetchUserInfo()
})
</script>
