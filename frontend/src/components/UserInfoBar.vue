<template>
  <v-banner class="elevation-2 rounded-0" lines="one">
    <template #prepend>
      <v-avatar color="primary">
        <span v-if="!userInfo?.avatar">{{ avatarText }}</span>
        <v-img v-else :src="userInfo?.avatar" />
      </v-avatar>
    </template>

    <v-banner-text class="w-100">
      <div class="d-flex justify-space-between align-center w-100">
        <span>你好, {{ userInfo?.name || userInfo?.username || '用户' }}</span>
        <span class="text-body-2">{{ idLabel }}: {{ userInfo?.username || '-' }}</span>
      </div>
    </v-banner-text>
  </v-banner>
</template>

<script lang="ts" setup>
import type { UserGetResponse } from '@/api'
import { computed } from 'vue'

const props = defineProps<{
  userInfo: UserGetResponse | null
  role?: 'student' | 'teacher' | 'admin' | 'office' | 'defense_board'
}>()

const idLabel = computed(() => {
  switch (props.role) {
    case 'student': {
      return '学号'
    }
    case 'teacher': {
      return '工号'
    }
    case 'admin': {
      return '管理员'
    }
    case 'office': {
      return '教务处'
    }
    case 'defense_board': {
      return '答辩组'
    }
    default: {
      return '编号'
    }
  }
})

const avatarText = computed(() => {
  const name = props.userInfo?.name || props.userInfo?.username || '用户'
  return name.charAt(0).toUpperCase()
})
</script>
