<template>
  <AppBar />
  <StudentDrawer :part="currentPart" />

  <UserInfoBar role="student" :user-info="userInfo" />

  <div class="main-container d-flex flex-column pa-4">
    <div class="text-h5 font-weight-black mb-4">毕业设计进度管理</div>
    <v-stepper alt-labels hide-actions :items="stepItems" :model-value="currentStep">
      <!-- Step 1: Topic Selection -->
      <template #item.1>
        <div class="pa-4">
          <div class="text-h6 mb-3">选题</div>
          <div v-if="assignedTopic">
            <div class="text-subtitle-2 text-grey mb-2">已选课题</div>
            <div class="text-h6 mb-2">{{ assignedTopic.topic_name }}</div>
            <div class="text-body-2 text-grey mb-1">指导教师: {{ assignedTopic.teacher_name }}</div>
            <div class="text-body-2 text-grey">
              分配时间: {{ formatDateTime(assignedTopic.assn_time) }}
            </div>
          </div>
          <div v-else class="text-grey">
            您还未选择课题，请前往选题页面选择课题
            <v-btn class="mt-2" color="primary" to="/student/select"> 前往选题 </v-btn>
          </div>
        </div>
      </template>

      <!-- Step 2: Initial Report -->
      <template #item.2>
        <div class="pa-4">
          <div class="text-h6 mb-3">开题报告</div>
          <div v-if="initialReport">
            <v-chip
              class="mb-2"
              :color="getProgressOutcomeColor(initialReport.prog_report_outcome)"
            >
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
            <div class="text-caption text-grey mb-2">
              提交时间: {{ formatDateTime(initialReport.prog_report_time) }}
            </div>
            <div class="d-flex justify-space-between">
              <v-btn
                v-if="initialReport.prog_report_attachment"
                color="info"
                @click="downloadAttachment(initialReport.prog_report_attachment, '开题报告')"
              >
                <v-icon start>mdi-download</v-icon>
                下载附件
              </v-btn>
              <v-spacer v-else />
              <v-btn
                v-if="initialReport.prog_report_outcome === 2"
                color="primary"
                @click="openSubmitDialog(0)"
              >
                重新提交
              </v-btn>
            </div>
          </div>
          <div v-else>
            <v-btn color="primary" @click="openSubmitDialog(0)"> 提交开题报告 </v-btn>
          </div>
        </div>
      </template>

      <!-- Step 3: Mid-term Report -->
      <template #item.3>
        <div class="pa-4">
          <div class="text-h6 mb-3">中期检查</div>
          <div v-if="midtermReport">
            <v-chip
              class="mb-2"
              :color="getProgressOutcomeColor(midtermReport.prog_report_outcome)"
            >
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
            <div class="text-caption text-grey mb-2">
              提交时间: {{ formatDateTime(midtermReport.prog_report_time) }}
            </div>
            <div class="d-flex justify-space-between">
              <v-btn
                v-if="midtermReport.prog_report_attachment"
                color="info"
                @click="downloadAttachment(midtermReport.prog_report_attachment, '中期检查')"
              >
                <v-icon start>mdi-download</v-icon>
                下载附件
              </v-btn>
              <v-spacer v-else />
              <v-btn
                v-if="midtermReport.prog_report_outcome === 2"
                color="primary"
                @click="openSubmitDialog(1)"
              >
                重新提交
              </v-btn>
            </div>
          </div>
          <div v-else>
            <v-btn
              color="primary"
              :disabled="!initialReport || initialReport.prog_report_outcome !== 1"
              @click="openSubmitDialog(1)"
            >
              提交中期检查
            </v-btn>
            <div
              v-if="!initialReport || initialReport.prog_report_outcome !== 1"
              class="text-caption text-grey mt-2"
            >
              需要先通过开题报告
            </div>
          </div>
        </div>
      </template>

      <!-- Step 4: Final Defense -->
      <template #item.4>
        <div class="pa-4">
          <div class="text-h6 mb-3">答辩</div>
          <div v-if="finalDefense">
            <v-chip
              v-if="finalDefense.final_def_outcome !== null"
              class="mb-2"
              :color="finalDefense.final_def_outcome ? 'success' : 'error'"
            >
              {{ finalDefense.final_def_outcome ? '通过' : '未通过' }}
            </v-chip>
            <v-chip v-else class="mb-2" color="warning"> 待答辩 </v-chip>
            <div v-if="finalDefense.final_def_grade" class="mb-2">
              <span class="text-subtitle-2 text-grey">成绩: </span>
              <span class="text-h6">{{ finalDefense.final_def_grade }}</span>
            </div>
            <div v-if="finalDefense.final_def_comment" class="mb-2">
              <div class="text-subtitle-2 text-grey">答辩组意见:</div>
              <div class="text-body-2">{{ finalDefense.final_def_comment }}</div>
            </div>
            <div class="text-caption text-grey mb-2">
              提交时间: {{ formatDateTime(finalDefense.final_def_time) }}
            </div>
            <div class="d-flex justify-space-between">
              <v-btn
                v-if="finalDefense.final_def_attachment"
                color="info"
                @click="downloadAttachment(finalDefense.final_def_attachment, '答辩材料')"
              >
                <v-icon start>mdi-download</v-icon>
                下载附件
              </v-btn>
              <v-spacer v-else />
              <v-btn
                v-if="finalDefense.final_def_outcome === false"
                color="primary"
                @click="openFinalDefenseDialog"
              >
                重新提交
              </v-btn>
            </div>
          </div>
          <div v-else>
            <v-btn
              color="primary"
              :disabled="!midtermReport || midtermReport.prog_report_outcome !== 1"
              @click="openFinalDefenseDialog"
            >
              提交答辩申请
            </v-btn>
            <div
              v-if="!midtermReport || midtermReport.prog_report_outcome !== 1"
              class="text-caption text-grey mt-2"
            >
              需要先通过中期检查
            </div>
          </div>
        </div>
      </template>
    </v-stepper>
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
            accept=".pdf,.doc,.docx,.zip"
            label="附件"
            prepend-icon="mdi-attachment"
            :rules="[(v) => !!v || '请上传附件']"
            variant="outlined"
            @change="handleAttachmentChange"
          />
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="submitDialogVisible = false"> 取消 </v-btn>
        <v-btn color="primary" @click="submitProgressReport"> 提交 </v-btn>
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
            accept=".pdf,.doc,.docx,.zip"
            label="答辩材料"
            prepend-icon="mdi-attachment"
            :rules="[(v) => !!v || '请上传答辩材料']"
            variant="outlined"
            @change="handleDefenseAttachmentChange"
          />
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="finalDefenseDialogVisible = false"> 取消 </v-btn>
        <v-btn color="primary" @click="submitFinalDefense"> 提交 </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

</template>

<script lang="ts" setup>
  import type { FinalDefenseDetails, ProgressReportDetailResponse, UserGetResponse } from '@/api'
  import { computed, onMounted, ref } from 'vue'
  import { createApiClient, getErrorMessage, PROGRESS_OUTCOME_MAP } from '@/api'
  import { API_BASE_URL } from '@/config'
  import { useSnackbar } from '@/composables/useSnackbar'

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
  const attachmentFile = ref<File | null>(null)
  const attachmentData = ref<string>('')
  const defenseAttachmentFile = ref<File | null>(null)
  const defenseAttachmentData = ref<string>('')

  const { showSuccess, showError } = useSnackbar()

  const stepItems = computed(() => [
    { title: '选题', value: 1, props: { editable: true } },
    { title: '开题', value: 2, props: { editable: currentStep.value >= 2 } },
    { title: '中期', value: 3, props: { editable: currentStep.value >= 3 } },
    { title: '答辩', value: 4, props: { editable: currentStep.value >= 4 } },
  ])

  const apiClient = createApiClient(API_BASE_URL)

  const initialReport = computed(() => {
    const reports = progressReports.value.filter(r => r.prog_report_type === 0)
    if (reports.length === 0) return undefined
    // Return the most recent report (latest submission)
    return reports.sort(
      (a, b) => new Date(b.prog_report_time).getTime() - new Date(a.prog_report_time).getTime(),
    )[0]
  })

  const midtermReport = computed(() => {
    const reports = progressReports.value.filter(r => r.prog_report_type === 1)
    if (reports.length === 0) return undefined
    // Return the most recent report (latest submission)
    return reports.sort(
      (a, b) => new Date(b.prog_report_time).getTime() - new Date(a.prog_report_time).getTime(),
    )[0]
  })

  const currentStep = computed(() => {
    if (!assignedTopic.value) return 1
    if (!initialReport.value || initialReport.value.prog_report_outcome !== 1) return 2
    if (!midtermReport.value || midtermReport.value.prog_report_outcome !== 1) return 3
    return 4
  })

  async function fetchUserInfo () {
    try {
      userInfo.value = await apiClient.auth.getCurrentUser()
    } catch (error) {
      console.error('Failed to fetch user info:', error)
    }
  }

  async function loadAssignedTopic () {
    try {
      // Get student's assignments to find approved topic
      const response = await apiClient.assignments.getAssignments()
      const approvedAssignment = response.assignments.find(a => a.status === 1)
      if (approvedAssignment) {
        assignedTopic.value = {
          topic_name: approvedAssignment.topic_name,
          teacher_name: '', // Not available in assignment response
          assn_time: approvedAssignment.request_time,
        }
      }
    } catch (error: any) {
      console.error('Failed to load assigned topic:', error)
    }
  }

  async function loadProgressReports () {
    try {
      const response = await apiClient.progressReports.getProgressReports()
      progressReports.value = response.reports
    } catch (error: any) {
      console.error('Failed to load progress reports:', error)
    }
  }

  async function loadFinalDefense () {
    try {
      const response = await apiClient.finalDefenses.getFinalDefenses()
      if (response.defenses && response.defenses.length > 0) {
        finalDefense.value = response.defenses[0] ?? null
      }
    } catch (error: any) {
      console.error('Failed to load final defense:', error)
    }
  }

  function formatDateTime (dateTime: string | null): string {
    if (!dateTime) return '-'
    const date = new Date(dateTime)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  function getProgressOutcomeName (outcome: number): string {
    return PROGRESS_OUTCOME_MAP.get(outcome as 0 | 1 | 2) || '未知'
  }

  function getProgressOutcomeColor (outcome: number): string {
    const colors = { 0: 'warning', 1: 'success', 2: 'error' }
    return colors[outcome as 0 | 1 | 2] || 'default'
  }

  function openSubmitDialog (type: 0 | 1) {
    submitType.value = type
    attachmentFile.value = null
    attachmentData.value = ''
    submitDialogVisible.value = true
  }

  function openFinalDefenseDialog () {
    defenseAttachmentFile.value = null
    defenseAttachmentData.value = ''
    finalDefenseDialogVisible.value = true
  }

  function handleAttachmentChange () {
    console.log(attachmentFile.value)
    if (attachmentFile.value !== null) {
      console.log(attachmentFile.value)
      const file = attachmentFile.value
      const reader = new FileReader()
      reader.addEventListener('load', e => {
        attachmentData.value = e.target?.result as string
      })
      reader.readAsDataURL(file)
    }
  }

  function handleDefenseAttachmentChange () {
    if (defenseAttachmentFile.value !== null) {
      const file = defenseAttachmentFile.value
      const reader = new FileReader()
      reader.addEventListener('load', e => {
        defenseAttachmentData.value = e.target?.result as string
      })
      reader.readAsDataURL(file)
    }
  }

  async function submitProgressReport () {
    const { valid } = await submitFormRef.value.validate()
    if (!valid || !attachmentData.value) return

    try {
      await apiClient.progressReports.createProgressReport({
        attachment: attachmentData.value,
      })
      showSuccess(submitType.value === 0 ? '开题报告提交成功' : '中期检查提交成功')
      submitDialogVisible.value = false
      await loadProgressReports()
    } catch (error: any) {
      console.error('Failed to submit progress report:', error)
      showError(getErrorMessage('progress', error.statusCode))
    }
  }

  async function submitFinalDefense () {
    const { valid } = await finalDefenseFormRef.value.validate()
    if (!valid || !defenseAttachmentData.value) return

    try {
      await apiClient.finalDefenses.createFinalDefense({
        attachment: defenseAttachmentData.value,
      })
      showSuccess('答辩申请提交成功')
      finalDefenseDialogVisible.value = false
      await loadFinalDefense()
    } catch (error: any) {
      console.error('Failed to submit final defense:', error)
      showError(getErrorMessage('defense', error.statusCode))
    }
  }

  function downloadAttachment (attachment: string, fileName: string) {
    try {
      // Create a temporary link element
      const link = document.createElement('a')
      link.href = attachment
      link.download = fileName
      document.body.append(link)
      link.click()
      link.remove()
    } catch (error) {
      console.error('Failed to download attachment:', error)
      showError('下载失败')
    }
  }

  onMounted(() => {
    fetchUserInfo()
    loadAssignedTopic()
    loadProgressReports()
    loadFinalDefense()
  })
</script>
