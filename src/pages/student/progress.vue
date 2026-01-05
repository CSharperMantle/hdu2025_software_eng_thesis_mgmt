<template>
  <AppBar />
  <StudentDrawer :part="currentPart" />

  <UserInfoBar :user-info="userInfo" role="student" />

  <div class="d-flex flex-column pa-4">
    <v-card class="w-100">
      <template #title>
        <span class="font-weight-black">毕业设计进度管理</span>
      </template>

      <v-card-text>
        <v-stepper v-model="currentStep" :items="stepItems" alt-labels>
          <!-- Step 1: Topic Selection -->
          <template #item.1>
            <v-card flat>
              <v-card-title>选题</v-card-title>
              <v-card-text>
                <div v-if="assignedTopic">
                  <div class="text-subtitle-2 text-grey mb-2">已选课题</div>
                  <div class="text-h6 mb-2">{{ assignedTopic.topic_name }}</div>
                  <div class="text-body-2 text-grey mb-1">指导教师: {{ assignedTopic.teacher_name }}</div>
                  <div class="text-body-2 text-grey">分配时间: {{ formatDateTime(assignedTopic.assn_time) }}</div>
                </div>
                <div v-else class="text-grey">
                  您还未选择课题，请前往选题页面选择课题
                  <v-btn color="primary" class="mt-2" to="/student/select">
                    前往选题
                  </v-btn>
                </div>
              </v-card-text>
            </v-card>
          </template>

          <!-- Step 2: Initial Report -->
          <template #item.2>
            <v-card flat>
              <v-card-title>开题报告</v-card-title>
              <v-card-text>
                <div v-if="initialReport">
                  <v-chip :color="getProgressOutcomeColor(initialReport.prog_report_outcome)" class="mb-2">
                    {{ getProgressOutcomeName(initialReport.prog_report_outcome) }}
                  </v-chip>
                  <div v-if="initialReport.prog_report_grade" class="mb-2">
                    <span class="text-subtitle-2 text-grey">成绩: </span>
                    <span class="text-h6">{{ initialReport.prog_report_grade }}</span>
                  </div>
                  <div v-if="initialReport.prog_report_comment" class="mb-2">
                    <div class="text-subtitle-2 text-grey">教师意见:</div>
                    <div class="text-body-2">{{ initialReport.prog_report_comment }}</div>
                  </div>
                  <div class="text-caption text-grey">提交时间: {{ formatDateTime(initialReport.prog_report_time) }}</div>
                </div>
                <div v-else>
                  <v-btn color="primary" @click="openSubmitDialog(0)">
                    提交开题报告
                  </v-btn>
                </div>
              </v-card-text>
            </v-card>
          </template>

          <!-- Step 3: Mid-term Report -->
          <template #item.3>
            <v-card flat>
              <v-card-title>中期检查</v-card-title>
              <v-card-text>
                <div v-if="midtermReport">
                  <v-chip :color="getProgressOutcomeColor(midtermReport.prog_report_outcome)" class="mb-2">
                    {{ getProgressOutcomeName(midtermReport.prog_report_outcome) }}
                  </v-chip>
                  <div v-if="midtermReport.prog_report_grade" class="mb-2">
                    <span class="text-subtitle-2 text-grey">成绩: </span>
                    <span class="text-h6">{{ midtermReport.prog_report_grade }}</span>
                  </div>
                  <div v-if="midtermReport.prog_report_comment" class="mb-2">
                    <div class="text-subtitle-2 text-grey">教师意见:</div>
                    <div class="text-body-2">{{ midtermReport.prog_report_comment }}</div>
                  </div>
                  <div class="text-caption text-grey">提交时间: {{ formatDateTime(midtermReport.prog_report_time) }}</div>
                </div>
                <div v-else>
                  <v-btn color="primary" @click="openSubmitDialog(1)" :disabled="!initialReport || initialReport.prog_report_outcome !== 1">
                    提交中期检查
                  </v-btn>
                  <div v-if="!initialReport || initialReport.prog_report_outcome !== 1" class="text-caption text-grey mt-2">
                    需要先通过开题报告
                  </div>
                </div>
              </v-card-text>
            </v-card>
          </template>

          <!-- Step 4: Final Defense -->
          <template #item.4>
            <v-card flat>
              <v-card-title>答辩</v-card-title>
              <v-card-text>
                <div v-if="finalDefense">
                  <v-chip v-if="finalDefense.final_def_outcome !== null" :color="finalDefense.final_def_outcome ? 'success' : 'error'" class="mb-2">
                    {{ finalDefense.final_def_outcome ? '通过' : '未通过' }}
                  </v-chip>
                  <v-chip v-else color="warning" class="mb-2">
                    待答辩
                  </v-chip>
                  <div v-if="finalDefense.final_def_grade" class="mb-2">
                    <span class="text-subtitle-2 text-grey">成绩: </span>
                    <span class="text-h6">{{ finalDefense.final_def_grade }}</span>
                  </div>
                  <div v-if="finalDefense.final_def_comment" class="mb-2">
                    <div class="text-subtitle-2 text-grey">答辩组意见:</div>
                    <div class="text-body-2">{{ finalDefense.final_def_comment }}</div>
                  </div>
                  <div class="text-caption text-grey">提交时间: {{ formatDateTime(finalDefense.final_def_time) }}</div>
                </div>
                <div v-else>
                  <v-btn color="primary" @click="openFinalDefenseDialog" :disabled="!midtermReport || midtermReport.prog_report_outcome !== 1">
                    提交答辩申请
                  </v-btn>
                  <div v-if="!midtermReport || midtermReport.prog_report_outcome !== 1" class="text-caption text-grey mt-2">
                    需要先通过中期检查
                  </div>
                </div>
              </v-card-text>
            </v-card>
          </template>
        </v-stepper>
      </v-card-text>
    </v-card>
  </div>

  <!-- Submit Progress Report Dialog -->
  <v-dialog v-model="submitDialogVisible" max-width="600">
    <v-card>
      <v-card-title class="d-flex justify-space-between align-center">
        <span class="text-h5">{{ submitType === 0 ? '提交开题报告' : '提交中期检查' }}</span>
        <v-btn icon="mdi-close" variant="text" @click="submitDialogVisible = false" />
      </v-card-title>

      <v-card-text>
        <v-form ref="submitFormRef">
          <v-file-input
            v-model="attachmentFile"
            label="附件"
            variant="outlined"
            accept=".pdf,.doc,.docx,.zip"
            prepend-icon="mdi-attachment"
            :rules="[v => !!v && v.length > 0 || '请上传附件']"
            @change="handleAttachmentChange"
          />
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="submitDialogVisible = false">
          取消
        </v-btn>
        <v-btn color="primary" @click="submitProgressReport">
          提交
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- Submit Final Defense Dialog -->
  <v-dialog v-model="finalDefenseDialogVisible" max-width="600">
    <v-card>
      <v-card-title class="d-flex justify-space-between align-center">
        <span class="text-h5">提交答辩申请</span>
        <v-btn icon="mdi-close" variant="text" @click="finalDefenseDialogVisible = false" />
      </v-card-title>

      <v-card-text>
        <v-form ref="finalDefenseFormRef">
          <v-file-input
            v-model="defenseAttachmentFile"
            label="答辩材料"
            variant="outlined"
            accept=".pdf,.doc,.docx,.zip"
            prepend-icon="mdi-attachment"
            :rules="[v => !!v && v.length > 0 || '请上传答辩材料']"
            @change="handleDefenseAttachmentChange"
          />
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="finalDefenseDialogVisible = false">
          取消
        </v-btn>
        <v-btn color="primary" @click="submitFinalDefense">
          提交
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-snackbar v-model="snackbar.show" :color="snackbar.color" :timeout="3000">
    {{ snackbar.message }}
  </v-snackbar>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue'
import {
	PROGRESS_OUTCOME_MAP,
	createApiClient,
	getErrorMessage,
} from '@/api'
import type { FinalDefenseDetails, ProgressReportDetailResponse, UserGetResponse } from '@/api'
import { API_BASE_URL } from '@/config'

const currentPart = 1 // Progress management is part 1 in StudentDrawer
const userInfo = ref<UserGetResponse | null>(null)
const assignedTopic = ref<any>(null)
const progressReports = ref<ProgressReportDetailResponse[]>([])
const finalDefense = ref<FinalDefenseDetails | null>(null)
const submitDialogVisible = ref(false)
const finalDefenseDialogVisible = ref(false)
const submitFormRef = ref<any>(null)
const finalDefenseFormRef = ref<any>(null)
const submitType = ref<0 | 1>(0) // 0: initial, 1: midterm
const attachmentFile = ref<File[]>([])
const attachmentData = ref<string>('')
const defenseAttachmentFile = ref<File[]>([])
const defenseAttachmentData = ref<string>('')

const snackbar = ref({
	show: false,
	message: '',
	color: 'success',
})

const stepItems = [
	{ title: '选题', value: 1 },
	{ title: '开题', value: 2 },
	{ title: '中期', value: 3 },
	{ title: '答辩', value: 4 },
]

const apiClient = createApiClient(API_BASE_URL)

const initialReport = computed(() => {
	return progressReports.value.find(r => r.prog_report_type === 0)
})

const midtermReport = computed(() => {
	return progressReports.value.find(r => r.prog_report_type === 1)
})

const currentStep = computed(() => {
	if (!assignedTopic.value)
		return 1
	if (!initialReport.value || initialReport.value.prog_report_outcome !== 1)
		return 2
	if (!midtermReport.value || midtermReport.value.prog_report_outcome !== 1)
		return 3
	return 4
})

async function fetchUserInfo() {
	try {
		userInfo.value = await apiClient.auth.getCurrentUser()
		// Check if student has assigned topic
		// This would require a student info endpoint, for now we'll skip
	}
	catch (error) {
		console.error('Failed to fetch user info:', error)
	}
}

async function loadProgressReports() {
	try {
		const response = await apiClient.progressReports.getProgressReports()
		progressReports.value = response.reports
	}
	catch (error: any) {
		console.error('Failed to load progress reports:', error)
	}
}

async function loadFinalDefense() {
	try {
		const response = await apiClient.finalDefenses.getFinalDefenses()
		if (response.defenses && response.defenses.length > 0) {
			finalDefense.value = response.defenses[0]
		}
	}
	catch (error: any) {
		console.error('Failed to load final defense:', error)
	}
}

function formatDateTime(dateTime: string | null): string {
	if (!dateTime)
		return '-'
	const date = new Date(dateTime)
	return date.toLocaleString('zh-CN', {
		year: 'numeric',
		month: '2-digit',
		day: '2-digit',
		hour: '2-digit',
		minute: '2-digit',
	})
}

function getProgressOutcomeName(outcome: number): string {
	return PROGRESS_OUTCOME_MAP.get(outcome as 0 | 1 | 2) || '未知'
}

function getProgressOutcomeColor(outcome: number): string {
	const colors = { 0: 'warning', 1: 'success', 2: 'error' }
	return colors[outcome as 0 | 1 | 2] || 'default'
}

function openSubmitDialog(type: 0 | 1) {
	submitType.value = type
	attachmentFile.value = []
	attachmentData.value = ''
	submitDialogVisible.value = true
}

function openFinalDefenseDialog() {
	defenseAttachmentFile.value = []
	defenseAttachmentData.value = ''
	finalDefenseDialogVisible.value = true
}

function handleAttachmentChange() {
	if (attachmentFile.value && attachmentFile.value.length > 0) {
		const file = attachmentFile.value[0]
		const reader = new FileReader()
		reader.onload = (e) => {
			attachmentData.value = e.target?.result as string
		}
		reader.readAsDataURL(file)
	}
}

function handleDefenseAttachmentChange() {
	if (defenseAttachmentFile.value && defenseAttachmentFile.value.length > 0) {
		const file = defenseAttachmentFile.value[0]
		const reader = new FileReader()
		reader.onload = (e) => {
			defenseAttachmentData.value = e.target?.result as string
		}
		reader.readAsDataURL(file)
	}
}

async function submitProgressReport() {
	const { valid } = await submitFormRef.value.validate()
	if (!valid || !attachmentData.value)
		return

	try {
		await apiClient.progressReports.createProgressReport({
			attachment: attachmentData.value,
		})
		snackbar.value = {
			show: true,
			message: submitType.value === 0 ? '开题报告提交成功' : '中期检查提交成功',
			color: 'success',
		}
		submitDialogVisible.value = false
		await loadProgressReports()
	}
	catch (error: any) {
		console.error('Failed to submit progress report:', error)
		snackbar.value = {
			show: true,
			message: getErrorMessage('progress', error.statusCode),
			color: 'error',
		}
	}
}

async function submitFinalDefense() {
	const { valid } = await finalDefenseFormRef.value.validate()
	if (!valid || !defenseAttachmentData.value)
		return

	try {
		await apiClient.finalDefenses.createFinalDefense({
			attachment: defenseAttachmentData.value,
		})
		snackbar.value = {
			show: true,
			message: '答辩申请提交成功',
			color: 'success',
		}
		finalDefenseDialogVisible.value = false
		await loadFinalDefense()
	}
	catch (error: any) {
		console.error('Failed to submit final defense:', error)
		snackbar.value = {
			show: true,
			message: getErrorMessage('defense', error.statusCode),
			color: 'error',
		}
	}
}

onMounted(() => {
	fetchUserInfo()
	loadProgressReports()
	loadFinalDefense()
})
</script>
