<template>
  <AppBar />

  <div class="login-background fill-height d-flex">
    <v-card class="ma-auto w-50">
      <template #title>
        <span class="font-weight-black">登录</span>
      </template>

      <v-card-text class="pt-4">
        <v-sheet class="mx-auto">
          <v-form @submit.prevent="submit">
            <v-text-field v-model="userName" label="用户名" />
            <v-text-field v-model="passWord" label="密码" type="password" />

            <v-btn block class="mt-2" text="Login" type="submit" />
          </v-form>
        </v-sheet>
      </v-card-text>
    </v-card>
  </div>

  <v-snackbar v-model="snackbar.show" :color="snackbar.color" :timeout="3000">
    {{ snackbar.message }}
  </v-snackbar>
</template>

<script lang="ts" setup>
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { createApiClient, getErrorMessage } from '@/api'
import { API_BASE_URL } from '@/config'

const userName = ref('')
const passWord = ref('')

const snackbar = ref({
  show: false,
  message: '',
  color: 'success',
})

const apiClient = createApiClient(API_BASE_URL)
const router = useRouter()

async function checkAndLogout() {
  try {
    // Try to get current user info to check if already logged in
    await apiClient.auth.getCurrentUser()

    // If we reach here, user is already logged in, so logout
    await apiClient.auth.logout()

    snackbar.value = {
      show: true,
      message: '检测到已登录，已自动退出',
      color: 'info',
    }
  } catch {
    // Not logged in, continue to login page
    console.log('Not logged in yet')
  }
}

onMounted(() => {
  checkAndLogout()
})

async function submit() {
  try {
    await apiClient.auth.login({
      username: userName.value,
      password: passWord.value,
    })

    // Get user info to determine role
    const userInfo = await apiClient.auth.getCurrentUser()
    console.log('Login successful, user role:', userInfo.role)

    snackbar.value = {
      show: true,
      message: '登录成功!',
      color: 'success',
    }

    // Redirect to appropriate page based on userInfo.role
    switch (userInfo.role) {
      case 'student': {
        router.push('/student/select')

        break
      }
      case 'teacher': {
        router.push('/teacher/topics')

        break
      }
      case 'office': {
        router.push('/office/topics')

        break
      }
      case 'defense_board': {
        router.push('/defense-board/scoring')

        break
      }
      // No default
    }
    // TODO: Add redirect for admin role
  } catch (error: any) {
    console.error('Login failed:', error)

    snackbar.value = {
      show: true,
      message: getErrorMessage('login', error.statusCode),
      color: 'error',
    }
  }
}
</script>

<style scoped>
.login-background {
  background-image: url(/background.jpg);
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
}
</style>
