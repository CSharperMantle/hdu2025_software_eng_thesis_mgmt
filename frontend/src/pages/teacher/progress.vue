<template>
  <AppBar />
  <TeacherDrawer :part="currentPart" />

  <UserInfoBar role="teacher" :user-info="userInfo" />

  <div class="main-container d-flex flex-column pa-4">
    <div class="text-h5 font-weight-black mb-4">学生进度管理</div>

    <v-expansion-panels v-model="expandedPanel">
      <v-expansion-panel
        v-for="report in groupedReports"
        :key="report.student_user_name"
        :value="report.student_user_name"
      >
        <v-expansion-panel-title>
          <div class="d-flex align-center justify-space-between w-100 pr-4">
            <div>
              <span class="font-weight-bold">{{ report.student_name }}</span>
              <span class="text-grey ml-2">课题: {{ report.topic_name }}</span>
            </div>
            <v-chip :color="getOverallStatusColor(report)" size="small">
              {{ getOverallStatus(report) }}
            </v-chip>
          </div>
        </v-expansion-panel-title>

        <v-expansion-panel-text>
          <v-stepper
            alt-labels
            elevation="0"
            hide-actions
            :items="getStepItems(report)"
            :model-value="getCurrentStep(report)"
          >
            <!-- Step 1: Topic Selection -->
            <template #item.1>
              <div class="pa-4">
                <div class="text-h6 mb-3">选题</div>
                <div class="text-body-2 text-grey">课题: {{ report.topic_name }}</div>
              </div>
            </template>

            <!-- Step 2: Initial Report -->
            <template #item.2>
              <div class="pa-4">
                <div class="text-h6 mb-3">开题报告</div>
                <div v-if="report.initial">
                  <v-chip
                    class="mb-2"
                    :color="getProgressOutcomeColor(report.initial.prog_report_outcome)"
                  >
                    {{ getProgressOutcomeName(report.initial.prog_report_outcome) }}
                  </v-chip>
                  <div v-if="report.initial.prog_report_grade" class="mb-2">
                    <span class="text-subtitle-2 text-grey">成绩: </span>
                    <span class="text-h6">{{ report.initial.prog_report_grade }}</span>
                  </div>
                  <div class="mb-2">
                    <div class="text-subtitle-2 text-grey mb-1">提交时间:</div>
                    <div class="text-body-2">
                      {{ formatDateTime(report.initial.prog_report_time) }}
                    </div>
                  </div>
                  <div v-if="report.initial.prog_report_comment" class="mb-2">
                    <div class="text-subtitle-2 text-grey mb-1">审核意见:</div>
                    <div class="text-body-2">{{ report.initial.prog_report_comment }}</div>
                  </div>
                  <div class="d-flex justify-space-between mt-2">
                    <v-btn
                      v-if="report.initial.prog_report_attachment"
                      color="info"
                      @click="
                        downloadAttachment(
                          report.initial.prog_report_attachment,
                          `${report.student_name}_开题报告`,
                        )
                      "
                    >
                      <v-icon start>mdi-download</v-icon>
                      下载附件
                    </v-btn>
                    <v-spacer v-else />
                    <v-btn
                      v-if="report.initial.prog_report_outcome === 0"
                      color="primary"
                      @click="openReviewDialog(report.initial)"
                    >
                      审核
                    </v-btn>
                  </div>
                </div>
                <div v-else class="text-grey">学生尚未提交</div>
              </div>
            </template>

            <!-- Step 3: Mid-term Report -->
            <template #item.3>
              <div class="pa-4">
                <div class="text-h6 mb-3">中期检查</div>
                <div v-if="report.midterm">
                  <v-chip
                    class="mb-2"
                    :color="getProgressOutcomeColor(report.midterm.prog_report_outcome)"
                  >
                    {{ getProgressOutcomeName(report.midterm.prog_report_outcome) }}
                  </v-chip>
                  <div v-if="report.midterm.prog_report_grade" class="mb-2">
                    <span class="text-subtitle-2 text-grey">成绩: </span>
                    <span class="text-h6">{{ report.midterm.prog_report_grade }}</span>
                  </div>
                  <div class="mb-2">
                    <div class="text-subtitle-2 text-grey mb-1">提交时间:</div>
                    <div class="text-body-2">
                      {{ formatDateTime(report.midterm.prog_report_time) }}
                    </div>
                  </div>
                  <div v-if="report.midterm.prog_report_comment" class="mb-2">
                    <div class="text-subtitle-2 text-grey mb-1">审核意见:</div>
                    <div class="text-body-2">{{ report.midterm.prog_report_comment }}</div>
                  </div>
                  <div class="d-flex justify-space-between mt-2">
                    <v-btn
                      v-if="report.midterm.prog_report_attachment"
                      color="info"
                      @click="
                        downloadAttachment(
                          report.midterm.prog_report_attachment,
                          `${report.student_name}_中期检查`,
                        )
                      "
                    >
                      <v-icon start>mdi-download</v-icon>
                      下载附件
                    </v-btn>
                    <v-spacer v-else />
                    <v-btn
                      v-if="report.midterm.prog_report_outcome === 0"
                      color="primary"
                      @click="openReviewDialog(report.midterm)"
                    >
                      审核
                    </v-btn>
                  </div>
                </div>
                <div v-else class="text-grey">学生尚未提交</div>
              </div>
            </template>

            <!-- Step 4: Final Defense -->
            <template #item.4>
              <div class="pa-4">
                <div class="text-h6 mb-3">答辩</div>
                <div v-if="report.defense">
                  <v-chip
                    v-if="report.defense.final_def_outcome !== null"
                    class="mb-2"
                    :color="report.defense.final_def_outcome ? 'success' : 'error'"
                  >
                    {{ report.defense.final_def_outcome ? '通过' : '未通过' }}
                  </v-chip>
                  <v-chip v-else class="mb-2" color="warning"> 待答辩 </v-chip>
                  <div v-if="report.defense.final_def_grade" class="mb-2">
                    <span class="text-subtitle-2 text-grey">成绩: </span>
                    <span class="text-h6">{{ report.defense.final_def_grade }}</span>
                  </div>
                  <div class="mb-2">
                    <div class="text-subtitle-2 text-grey mb-1">提交时间:</div>
                    <div class="text-body-2">
                      {{ formatDateTime(report.defense.final_def_time) }}
                    </div>
                  </div>
                  <div v-if="report.defense.final_def_comment" class="mb-2">
                    <div class="text-subtitle-2 text-grey mb-1">答辩组意见:</div>
                    <div class="text-body-2">{{ report.defense.final_def_comment }}</div>
                  </div>
                  <div class="d-flex justify-space-between mt-2">
                    <v-btn
                      v-if="report.defense.final_def_attachment"
                      color="info"
                      @click="
                        downloadAttachment(
                          report.defense.final_def_attachment,
                          `${report.student_name}_答辩材料`,
                        )
                      "
                    >
                      <v-icon start>mdi-download</v-icon>
                      下载附件
                    </v-btn>
                    <v-spacer v-else class="ga-2" />
                    <div
                      v-if="
                        (report.defense.final_def_outcome === null ||
                          report.defense.final_def_outcome === undefined) &&
                        !report.defense.def_board_user_name
                      "
                      class="d-flex gap-2"
                    >
                      <v-btn color="error" @click="rejectDefense(report.defense)"> 拒绝 </v-btn>
                      <v-btn class="ml-2" color="success" @click="approveDefense(report.defense)">
                        批准答辩
                      </v-btn>
                    </div>
                    <div
                      v-else-if="
                        report.defense.def_board_user_name &&
                        (report.defense.final_def_outcome === null ||
                          report.defense.final_def_outcome === undefined)
                      "
                      class="text-grey text-caption mt-2"
                    >
                      已批准答辩，等待答辩组评分
                    </div>
                  </div>
                </div>
                <div v-else class="text-grey">学生尚未提交</div>
              </div>
            </template>
          </v-stepper>
        </v-expansion-panel-text>
      </v-expansion-panel>
    </v-expansion-panels>

    <div v-if="groupedReports.length === 0" class="text-center text-grey pa-8">
      暂无学生进度数据
    </div>
  </div>

  <!-- Review Dialog -->
  <v-dialog v-model="reviewDialogVisible" max-width="600">
    <v-card v-if="selectedReport">
      <v-card-title class="d-flex justify-space-between align-center">
        <span class="text-h5"
          >审核{{ selectedReport.prog_report_type === 0 ? '开题报告' : '中期检查' }}</span
        >
        <v-btn icon="mdi-close" variant="text" @click="reviewDialogVisible = false" />
      </v-card-title>

      <v-card-text>
        <v-form ref="reviewFormRef">
          <v-select
            v-model="reviewForm.outcome"
            item-title="name"
            item-value="value"
            :items="outcomeOptions"
            label="审核结果"
            :rules="[(v) => (v !== null && v !== undefined) || '请选择审核结果']"
            variant="outlined"
          />

          <v-text-field
            v-model.number="reviewForm.grade"
            label="成绩"
            :rules="[(v) => (v >= 0 && v <= 100) || '成绩范围0-100']"
            type="number"
            variant="outlined"
          />

          <v-textarea v-model="reviewForm.comment" label="审核意见" rows="4" variant="outlined" />
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="reviewDialogVisible = false"> 取消 </v-btn>
        <v-btn color="primary" @click="submitReview"> 提交 </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-snackbar v-model="snackbar.show" :color="snackbar.color" :timeout="3000">
    {{ snackbar.message }}
  </v-snackbar>
</template>

<script lang="ts" setup>
import type { FinalDefenseDetails, ProgressReportDetailResponse, UserGetResponse } from '@/api'
import { onMounted, ref } from 'vue'
import { createApiClient, getErrorMessage, PROGRESS_OUTCOME_MAP } from '@/api'
import { API_BASE_URL } from '@/config'

const currentPart = 2 // Progress review is part 2 in TeacherDrawer
const userInfo = ref<UserGetResponse | null>(null)
const progressReports = ref<ProgressReportDetailResponse[]>([])
const finalDefenses = ref<FinalDefenseDetails[]>([])
const expandedPanel = ref<number | null>(null)
const reviewDialogVisible = ref(false)
const reviewFormRef = ref<any>(null)
const selectedReport = ref<ProgressReportDetailResponse | null>(null)

const reviewForm = ref({
  outcome: null as number | null,
  grade: null as number | null,
  comment: '',
})

const snackbar = ref({
  show: false,
  message: '',
  color: 'success',
})

function getStepItems(report: GroupedReport) {
  const currentStep = getCurrentStep(report)
  return [
    { title: '选题', value: 1, props: { editable: true } },
    { title: '开题', value: 2, props: { editable: currentStep >= 2 } },
    { title: '中期', value: 3, props: { editable: currentStep >= 3 } },
    { title: '答辩', value: 4, props: { editable: currentStep >= 4 } },
  ]
}

const outcomeOptions = [
  { value: 1, name: '通过' },
  { value: 2, name: '打回' },
]

const apiClient = createApiClient(API_BASE_URL)

interface GroupedReport {
  student_user_name: string
  student_name: string
  topic_id: number
  topic_name: string
  initial?: ProgressReportDetailResponse
  midterm?: ProgressReportDetailResponse
  defense?: FinalDefenseDetails
}

const groupedReports = ref<GroupedReport[]>([])

async function fetchUserInfo() {
  try {
    userInfo.value = await apiClient.auth.getCurrentUser()
  } catch (error) {
    console.error('Failed to fetch user info:', error)
  }
}

async function loadProgressReports() {
  try {
    const response = await apiClient.progressReports.getProgressReports()
    progressReports.value = response.reports
  } catch (error: any) {
    console.error('Failed to load progress reports:', error)
  }
}

async function loadFinalDefenses() {
  try {
    const response = await apiClient.finalDefenses.getFinalDefenses()
    console.log(response.defenses)
    finalDefenses.value = response.defenses
  } catch (error: any) {
    console.error('Failed to load final defenses:', error)
  }
}

async function groupReports() {
  const grouped = new Map<string, GroupedReport>()

  // Group progress reports - keep only the most recent report of each type per student
  const reportsByStudent = new Map<
    string,
    { initial: ProgressReportDetailResponse[]; midterm: ProgressReportDetailResponse[] }
  >()

  for (const report of progressReports.value) {
    if (!reportsByStudent.has(report.student_user_name)) {
      reportsByStudent.set(report.student_user_name, { initial: [], midterm: [] })
    }
    const studentReports = reportsByStudent.get(report.student_user_name)!
    if (report.prog_report_type === 0) {
      studentReports.initial.push(report)
    } else {
      studentReports.midterm.push(report)
    }
  }

  // Create grouped reports with most recent reports
  for (const [studentUserName, reports] of reportsByStudent.entries()) {
    const latestInitial =
      reports.initial.length > 0
        ? reports.initial.sort(
            (a, b) =>
              new Date(b.prog_report_time).getTime() - new Date(a.prog_report_time).getTime(),
          )[0]
        : undefined
    const latestMidterm =
      reports.midterm.length > 0
        ? reports.midterm.sort(
            (a, b) =>
              new Date(b.prog_report_time).getTime() - new Date(a.prog_report_time).getTime(),
          )[0]
        : undefined

    if (latestInitial || latestMidterm) {
      const report = latestInitial || latestMidterm!
      grouped.set(studentUserName, {
        student_user_name: studentUserName,
        student_name: report.student_name,
        topic_id: report.topic_id,
        topic_name: '',
        initial: latestInitial,
        midterm: latestMidterm,
      })
    }
  }

  // Add final defenses and update topic names
  // Group defenses by student to handle multiple defense records
  const defensesByStudent = new Map<string, FinalDefenseDetails[]>()
  for (const defense of finalDefenses.value) {
    if (!defensesByStudent.has(defense.student_user_name)) {
      defensesByStudent.set(defense.student_user_name, [])
    }
    defensesByStudent.get(defense.student_user_name)!.push(defense)
  }

  // For each student, select the appropriate defense record
  for (const [studentUserName, defenses] of defensesByStudent.entries()) {
    // If there are multiple defense records, prioritize incomplete ones (final_def_outcome is null/undefined)
    const incompleteDefenses = defenses.filter(
      (d) => d.final_def_outcome === null || d.final_def_outcome === undefined,
    )

    let selectedDefense: FinalDefenseDetails
    if (incompleteDefenses.length > 0) {
      // If there are incomplete defenses, select the most recent one
      const sorted = incompleteDefenses.sort(
        (a, b) => new Date(b.final_def_time).getTime() - new Date(a.final_def_time).getTime(),
      )
      selectedDefense = sorted[0]!
    } else {
      // If all defenses are complete, select the most recent one
      const sorted = defenses.sort(
        (a, b) => new Date(b.final_def_time).getTime() - new Date(a.final_def_time).getTime(),
      )
      selectedDefense = sorted[0]!
    }

    if (!grouped.has(studentUserName)) {
      grouped.set(studentUserName, {
        student_user_name: studentUserName,
        student_name: selectedDefense.student_name,
        topic_id: selectedDefense.topic_id,
        topic_name: selectedDefense.topic_name,
      })
    }
    const group = grouped.get(studentUserName)!
    group.topic_name = selectedDefense.topic_name
    group.defense = selectedDefense
  }

  // Fetch topic names for students without final defense
  const topicIds = new Set<number>()
  for (const report of grouped.values()) {
    if (!report.topic_name && report.topic_id) {
      topicIds.add(report.topic_id)
    }
  }

  // Fetch topic details for missing topic names
  const topicMap = new Map<number, string>()
  for (const topicId of topicIds) {
    try {
      const topic = await apiClient.topics.getTopicById(topicId)
      topicMap.set(topicId, topic.topic_name)
    } catch (error) {
      console.error(`Failed to fetch topic ${topicId}:`, error)
    }
  }

  // Update topic names
  for (const report of grouped.values()) {
    if (!report.topic_name && topicMap.has(report.topic_id)) {
      report.topic_name = topicMap.get(report.topic_id)!
    }
  }

  groupedReports.value = Array.from(grouped.values())
}

function getCurrentStep(report: GroupedReport): number {
  if (!report.initial || report.initial.prog_report_outcome !== 1) return 2
  if (!report.midterm || report.midterm.prog_report_outcome !== 1) return 3
  return 4
}

function getOverallStatus(report: GroupedReport): string {
  if (report.defense) {
    if (
      report.defense.final_def_outcome !== null &&
      report.defense.final_def_outcome !== undefined
    ) {
      return report.defense.final_def_outcome ? '已完成' : '答辩未通过'
    }
    return '待答辩'
  }
  if (report.midterm?.prog_report_outcome === 1) {
    return '中期已通过'
  }
  if (report.midterm?.prog_report_outcome === 0) {
    return '中期待审核'
  }
  if (report.initial?.prog_report_outcome === 1) {
    return '开题已通过'
  }
  if (report.initial?.prog_report_outcome === 0) {
    return '开题待审核'
  }
  return '未开始'
}

function getOverallStatusColor(report: GroupedReport): string {
  const status = getOverallStatus(report)
  if (status.includes('已完成')) return 'success'
  if (status.includes('待审核')) return 'warning'
  if (status.includes('未通过')) return 'error'
  if (status.includes('已通过')) return 'info'
  return 'default'
}

function formatDateTime(dateTime: string | null): string {
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

function getProgressOutcomeName(outcome: number): string {
  return PROGRESS_OUTCOME_MAP.get(outcome as 0 | 1 | 2) || '未知'
}

function getProgressOutcomeColor(outcome: number): string {
  const colors = { 0: 'warning', 1: 'success', 2: 'error' }
  return colors[outcome as 0 | 1 | 2] || 'default'
}

function openReviewDialog(report: ProgressReportDetailResponse) {
  selectedReport.value = report
  reviewForm.value = {
    outcome: null,
    grade: null,
    comment: '',
  }
  reviewDialogVisible.value = true
}

async function submitReview() {
  const { valid } = await reviewFormRef.value.validate()
  if (!valid || !selectedReport.value) return

  try {
    await apiClient.progressReports.updateProgressReport(selectedReport.value.prog_report_id, {
      outcome: reviewForm.value.outcome as 0 | 1 | 2,
      comment: reviewForm.value.comment || undefined,
      grade: reviewForm.value.grade || undefined,
    })
    snackbar.value = {
      show: true,
      message: '审核提交成功',
      color: 'success',
    }
    reviewDialogVisible.value = false
    // Reload data
    await loadProgressReports()
    await groupReports()
  } catch (error: any) {
    console.error('Failed to submit review:', error)
    snackbar.value = {
      show: true,
      message: getErrorMessage('progress', error.statusCode),
      color: 'error',
    }
  }
}

function downloadAttachment(attachment: string, fileName: string) {
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
    snackbar.value = {
      show: true,
      message: '下载失败',
      color: 'error',
    }
  }
}

async function approveDefense(defense: FinalDefenseDetails) {
  try {
    await apiClient.finalDefenses.updateFinalDefenseAsTeacher(defense.final_def_id, {
      approved: true,
    })
    snackbar.value = {
      show: true,
      message: '答辩申请已批准',
      color: 'success',
    }
    // Reload data
    await loadFinalDefenses()
    await groupReports()
  } catch (error: any) {
    console.error('Failed to approve defense:', error)
    snackbar.value = {
      show: true,
      message: getErrorMessage('defense', error.statusCode),
      color: 'error',
    }
  }
}

async function rejectDefense(defense: FinalDefenseDetails) {
  try {
    await apiClient.finalDefenses.updateFinalDefenseAsTeacher(defense.final_def_id, {
      approved: false,
    })
    snackbar.value = {
      show: true,
      message: '答辩申请已拒绝',
      color: 'success',
    }
    // Reload data
    await loadFinalDefenses()
    await groupReports()
  } catch (error: any) {
    console.error('Failed to reject defense:', error)
    snackbar.value = {
      show: true,
      message: getErrorMessage('defense', error.statusCode),
      color: 'error',
    }
  }
}

onMounted(async () => {
  fetchUserInfo()
  await loadProgressReports()
  await loadFinalDefenses()
  await groupReports()
})
</script>

<style scoped>
:deep(.v-stepper-header) {
  box-shadow: none !important;
}
</style>
