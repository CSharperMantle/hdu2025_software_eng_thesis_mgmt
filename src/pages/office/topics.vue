<template>
  <AppBar />
  <OfficeDrawer :part="currentPart" />

  <UserInfoBar role="office" :user-info="userInfo" />

  <div class="d-flex flex-column pa-4">
    <v-card class="w-100">
      <template #title>
        <span class="font-weight-black">课题审核</span>
      </template>

      <v-card-text>
        <v-text-field
          v-model="search"
          class="mb-4"
          clearable
          label="搜索课题名称或描述"
          prepend-inner-icon="mdi-magnify"
          @update:model-value="onSearchChange"
        />

        <v-data-table-server
          v-model:items-per-page="itemsPerPage"
          v-model:page="page"
          :headers="headers"
          item-value="topic_id"
          :items="topics"
          :items-length="totalItems"
          :items-per-page-options="[5, 10, 15, 20]"
          :loading="loading"
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
              icon="mdi-eye"
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

  <v-dialog v-model="dialogVisible" max-width="800">
    <v-card v-if="selectedTopic">
      <v-card-title class="d-flex justify-space-between align-center">
        <span class="text-h5">课题审核</span>
        <v-btn icon="mdi-close" variant="text" @click="dialogVisible = false" />
      </v-card-title>

      <v-card-text>
        <v-row>
          <v-col cols="12">
            <div class="text-subtitle-2 text-grey">课题名称</div>
            <div class="text-body-1">{{ selectedTopic.topic_name }}</div>
          </v-col>

          <v-col cols="12" md="6">
            <div class="text-subtitle-2 text-grey">指导教师</div>
            <div class="text-body-1">{{ selectedTopic.teacher_name }}</div>
          </v-col>

          <v-col cols="12" md="6">
            <div class="text-subtitle-2 text-grey">专业</div>
            <div class="text-body-1">{{ selectedTopic.major_name }}</div>
          </v-col>

          <v-col cols="12" md="6">
            <div class="text-subtitle-2 text-grey">课题类型</div>
            <div class="text-body-1">{{ getTopicTypeName(selectedTopic.topic_type) }}</div>
          </v-col>

          <v-col cols="12" md="6">
            <div class="text-subtitle-2 text-grey">最大学生数</div>
            <div class="text-body-1">{{ selectedTopic.topic_max_students }}</div>
          </v-col>

          <v-col cols="12" md="6">
            <div class="text-subtitle-2 text-grey">当前审核状态</div>
            <v-chip :color="getTopicReviewStatusColor(selectedTopic.topic_review_status)" size="small">
              {{ getTopicReviewStatusName(selectedTopic.topic_review_status) }}
            </v-chip>
          </v-col>

          <v-col cols="12" md="6">
            <div class="text-subtitle-2 text-grey">已选学生数</div>
            <div class="text-body-1">{{ selectedTopic.current_student_count }}</div>
          </v-col>

          <v-col cols="12">
            <div class="text-subtitle-2 text-grey">课题描述</div>
            <div class="text-body-1" style="white-space: pre-wrap;">{{ selectedTopic.topic_description }}</div>
          </v-col>
        </v-row>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn
          v-if="selectedTopic.topic_review_status === 0"
          color="error"
          variant="text"
          @click="updateTopicStatus(2)"
        >
          拒绝
        </v-btn>
        <v-btn
          v-if="selectedTopic.topic_review_status === 0"
          color="success"
          @click="updateTopicStatus(1)"
        >
          通过
        </v-btn>
        <v-btn v-else @click="dialogVisible = false">
          关闭
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
  import type { TopicBrief, TopicDetails, UserGetResponse } from '@/api'
  import { onMounted, ref } from 'vue'
  import {
    createApiClient,
    getErrorMessage,
    getTopicReviewStatusColor,
    getTopicReviewStatusName,
    getTopicTypeName,
  } from '@/api'
  import { API_BASE_URL } from '@/config'

  const currentPart = 0
  const userInfo = ref<UserGetResponse | null>(null)
  const topics = ref<TopicBrief[]>([])
  const loading = ref(false)
  const page = ref(1)
  const itemsPerPage = ref(20)
  const totalItems = ref(0)
  const search = ref('')
  const dialogVisible = ref(false)
  const selectedTopic = ref<TopicDetails | null>(null)

  const snackbar = ref({
    show: false,
    message: '',
    color: 'success',
  })

  const headers = [
    { title: '课题名称', key: 'topic_name', sortable: false },
    { title: '指导教师', key: 'teacher_name', sortable: false },
    { title: '课题类型', key: 'topic_type', sortable: false },
    { title: '审核状态', key: 'topic_review_status', sortable: false },
    { title: '名额', key: 'availability', sortable: false },
    { title: '操作', key: 'actions', sortable: false },
  ]

  const apiClient = createApiClient(API_BASE_URL)

  async function fetchUserInfo () {
    try {
      userInfo.value = await apiClient.auth.getCurrentUser()
    } catch (error) {
      console.error('Failed to fetch user info:', error)
    }
  }

  async function loadTopics () {
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
    } catch (error: any) {
      console.error('Failed to load topics:', error)
      snackbar.value = {
        show: true,
        message: '加载课题列表失败',
        color: 'error',
      }
    } finally {
      loading.value = false
    }
  }

  function onSearchChange () {
    page.value = 1
    loadTopics()
  }

  async function viewTopic (topicId: number) {
    try {
      selectedTopic.value = await apiClient.topics.getTopicById(topicId)
      dialogVisible.value = true
    } catch (error: any) {
      console.error('Failed to load topic details:', error)
      snackbar.value = {
        show: true,
        message: '加载课题详情失败',
        color: 'error',
      }
    }
  }

  async function updateTopicStatus (status: 0 | 1 | 2) {
    if (!selectedTopic.value)
      return

    try {
      await apiClient.topics.updateTopicAsOffice(selectedTopic.value.topic_id, {
        topic_review_status: status,
      })
      snackbar.value = {
        show: true,
        message: status === 1 ? '课题已通过审核' : '课题已被拒绝',
        color: 'success',
      }
      dialogVisible.value = false
      // Reload topics
      await loadTopics()
    } catch (error: any) {
      console.error('Failed to update topic status:', error)
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
