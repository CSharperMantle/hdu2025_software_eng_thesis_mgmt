<template>
  <AppBar />
  <DefenseBoardDrawer :part="currentPart" />

  <UserInfoBar :user-info="userInfo" role="defense_board" />

  <div class="d-flex flex-column pa-4">
    <div class="text-h5 font-weight-black mb-4">答辩评分管理</div>

    <v-expansion-panels v-model="expandedPanel">
      <v-expansion-panel
        v-for="defense in finalDefenses"
        :key="defense.final_def_id"
        :value="defense.final_def_id"
      >
        <v-expansion-panel-title>
          <div class="d-flex align-center justify-space-between w-100 pr-4">
            <div>
              <span class="font-weight-bold">{{ defense.student_name }}</span>
              <span class="text-grey ml-2">课题: {{ defense.topic_name }}</span>
            </div>
            <v-chip size="small" :color="getDefenseStatusColor(defense)">
              {{ getDefenseStatus(defense) }}
            </v-chip>
          </div>
        </v-expansion-panel-title>

        <v-expansion-panel-text>
          <div class="pa-4">
            <div class="mb-3">
              <div class="text-subtitle-2 text-grey mb-1">提交时间:</div>
              <div class="text-body-2">{{ formatDateTime(defense.final_def_time) }}</div>
            </div>

            <div v-if="defense.final_def_outcome !== null && defense.final_def_outcome !== undefined" class="mb-3">
              <v-chip :color="defense.final_def_outcome ? 'success' : 'error'" class="mb-2">
                {{ defense.final_def_outcome ? '通过' : '未通过' }}
              </v-chip>
              <div v-if="defense.final_def_grade" class="mb-2">
                <span class="text-subtitle-2 text-grey">成绩: </span>
                <span class="text-h6">{{ defense.final_def_grade }}</span>
              </div>
              <div v-if="defense.final_def_comment" class="mb-2">
                <div class="text-subtitle-2 text-grey mb-1">评审意见:</div>
                <div class="text-body-2">{{ defense.final_def_comment }}</div>
              </div>
            </div>

            <div class="d-flex justify-space-between mt-3">
              <v-btn
                v-if="defense.final_def_attachment"
                color="info"
                @click="downloadAttachment(defense.final_def_attachment, `${defense.student_name}_答辩材料`)"
              >
                <v-icon start>mdi-download</v-icon>
                下载答辩材料
              </v-btn>
              <v-spacer v-else />
              <v-btn
                v-if="defense.final_def_outcome === null || defense.final_def_outcome === undefined"
                color="primary"
                @click="openScoringDialog(defense)"
              >
                评分
              </v-btn>
            </div>
          </div>
        </v-expansion-panel-text>
      </v-expansion-panel>
    </v-expansion-panels>

    <div v-if="finalDefenses.length === 0" class="text-center text-grey pa-8">
      暂无待评分的答辩
    </div>
  </div>

  <!-- Scoring Dialog -->
  <v-dialog v-model="scoringDialogVisible" max-width="600">
    <v-card v-if="selectedDefense">
      <v-card-title class="d-flex justify-space-between align-center">
        <span class="text-h5">答辩评分 - {{ selectedDefense.student_name }}</span>
        <v-btn icon="mdi-close" variant="text" @click="scoringDialogVisible = false" />
      </v-card-title>

      <v-card-text>
        <div class="mb-4">
          <div class="text-subtitle-2 text-grey mb-1">课题名称:</div>
          <div class="text-body-1">{{ selectedDefense.topic_name }}</div>
        </div>

        <v-form ref="scoringFormRef">
          <v-radio-group
            v-model="scoringForm.outcome"
            label="答辩结果"
            :rules="[v => v !== null && v !== undefined || '请选择答辩结果']"
          >
            <v-radio label="通过" :value="true" />
            <v-radio label="未通过" :value="false" />
          </v-radio-group>

          <v-text-field
            v-model.number="scoringForm.grade"
            label="答辩成绩"
            type="number"
            variant="outlined"
            :rules="[
              v => (v !== null && v !== undefined && v !== '') || '请输入成绩',
              v => (v >= 0 && v <= 100) || '成绩范围0-100'
            ]"
          />

          <v-textarea
            v-model="scoringForm.comment"
            label="评审意见"
            variant="outlined"
            rows="4"
            :rules="[v => !!v || '请输入评审意见']"
          />
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="scoringDialogVisible = false">
          取消
        </v-btn>
        <v-btn color="primary" @click="submitScoring">
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
import { onMounted, ref } from 'vue'
import {
	createApiClient,
	getErrorMessage,
} from '@/api'
import type { FinalDefenseDetails, UserGetResponse } from '@/api'
import { API_BASE_URL } from '@/config'

const currentPart = 0 // Scoring is part 0 in DefenseBoardDrawer
const userInfo = ref<UserGetResponse | null>(null)
const finalDefenses = ref<FinalDefenseDetails[]>([])
const expandedPanel = ref<number | null>(null)
const scoringDialogVisible = ref(false)
const scoringFormRef = ref<any>(null)
const selectedDefense = ref<FinalDefenseDetails | null>(null)

const scoringForm = ref({
	outcome: null as boolean | null,
	grade: null as number | null,
	comment: '',
})

const snackbar = ref({
	show: false,
	message: '',
	color: 'success',
})

const apiClient = createApiClient(API_BASE_URL)

async function fetchUserInfo() {
	try {
		userInfo.value = await apiClient.auth.getCurrentUser()
	}
	catch (error) {
		console.error('Failed to fetch user info:', error)
	}
}

async function loadFinalDefenses() {
	try {
		const response = await apiClient.finalDefenses.getFinalDefenses()
		finalDefenses.value = response.defenses
	}
	catch (error: any) {
		console.error('Failed to load final defenses:', error)
	}
}

function getDefenseStatus(defense: FinalDefenseDetails): string {
	if (defense.final_def_outcome !== null && defense.final_def_outcome !== undefined) {
		return defense.final_def_outcome ? '已通过' : '未通过'
	}
	return '待评分'
}

function getDefenseStatusColor(defense: FinalDefenseDetails): string {
	if (defense.final_def_outcome !== null && defense.final_def_outcome !== undefined) {
		return defense.final_def_outcome ? 'success' : 'error'
	}
	return 'warning'
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

function openScoringDialog(defense: FinalDefenseDetails) {
	selectedDefense.value = defense
	scoringForm.value = {
		outcome: null,
		grade: null,
		comment: '',
	}
	scoringDialogVisible.value = true
}

async function submitScoring() {
	const { valid } = await scoringFormRef.value.validate()
	if (!valid || !selectedDefense.value)
		return

	try {
		await apiClient.finalDefenses.updateFinalDefenseAsDefenseBoard(
			selectedDefense.value.final_def_id,
			{
				outcome: scoringForm.value.outcome as boolean,
				comment: scoringForm.value.comment,
				grade: scoringForm.value.grade as number,
			},
		)
		snackbar.value = {
			show: true,
			message: '评分提交成功',
			color: 'success',
		}
		scoringDialogVisible.value = false
		// Reload data
		await loadFinalDefenses()
	}
	catch (error: any) {
		console.error('Failed to submit scoring:', error)
		snackbar.value = {
			show: true,
			message: getErrorMessage('defense', error.statusCode),
			color: 'error',
		}
	}
}

function downloadAttachment(attachment: string, fileName: string) {
	try {
		const link = document.createElement('a')
		link.href = attachment
		link.download = fileName
		document.body.appendChild(link)
		link.click()
		document.body.removeChild(link)
	}
	catch (error) {
		console.error('Failed to download attachment:', error)
		snackbar.value = {
			show: true,
			message: '下载失败',
			color: 'error',
		}
	}
}

onMounted(() => {
	fetchUserInfo()
	loadFinalDefenses()
})
</script>
