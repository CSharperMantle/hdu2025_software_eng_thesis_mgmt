<template>
  <v-divider />
  <v-list-subheader>信息维护</v-list-subheader>
  <v-list-item link prepend-icon="mdi-cog" title="账户信息维护" />
  <v-list-item link prepend-icon="mdi-account" title="修改密码" />
  <v-divider />
  <v-list-item link prepend-icon="mdi-logout" title="退出登录" @click="handleLogout" />
</template>

<script lang="ts" setup>
import { useRouter } from 'vue-router'
import { createApiClient } from '@/api'
import { API_BASE_URL } from '@/config'

const router = useRouter()
const apiClient = createApiClient(API_BASE_URL)

async function handleLogout() {
	try {
		await apiClient.auth.logout()
		router.push('/')
	}
	catch (error) {
		console.error('Logout failed:', error)
		// Even if logout API fails, redirect to login page
		router.push('/')
	}
}
</script>
