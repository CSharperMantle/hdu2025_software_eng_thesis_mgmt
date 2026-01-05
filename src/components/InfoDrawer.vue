<template>
  <v-divider />
  <v-list-subheader>信息维护</v-list-subheader>
  <v-list-item link prepend-icon="mdi-cog" title="账户信息维护" @click="openUserInfoDialog" />
  <v-list-item link prepend-icon="mdi-account" title="修改密码" @click="openPasswordDialog" />
  <v-divider />
  <v-list-item link prepend-icon="mdi-logout" title="退出登录" @click="handleLogout" />

  <!-- User Info Dialog -->
  <v-dialog v-model="userInfoDialogVisible" max-width="600">
    <v-card>
      <v-card-title class="d-flex justify-space-between align-center">
        <span class="text-h5">账户信息维护</span>
        <v-btn icon="mdi-close" variant="text" @click="userInfoDialogVisible = false" />
      </v-card-title>

      <v-card-text>
        <v-form ref="userInfoFormRef">
          <v-text-field
            v-model="userInfoForm.name"
            label="姓名"
            :rules="[v => !!v || '请输入姓名']"
            variant="outlined"
          />

          <v-file-input
            v-model="avatarFile"
            accept="image/*"
            label="头像"
            prepend-icon="mdi-camera"
            variant="outlined"
            @change="handleAvatarChange"
          />

          <v-avatar v-if="avatarPreview" class="mb-4" size="100">
            <v-img :src="avatarPreview" />
          </v-avatar>
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="userInfoDialogVisible = false">
          取消
        </v-btn>
        <v-btn color="primary" @click="saveUserInfo">
          保存
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- Password Dialog -->
  <v-dialog v-model="passwordDialogVisible" max-width="600">
    <v-card>
      <v-card-title class="d-flex justify-space-between align-center">
        <span class="text-h5">修改密码</span>
        <v-btn icon="mdi-close" variant="text" @click="passwordDialogVisible = false" />
      </v-card-title>

      <v-card-text>
        <v-form ref="passwordFormRef">
          <v-text-field
            v-model="passwordForm.password"
            label="新密码"
            :rules="[v => !!v || '请输入新密码', v => v.length >= 5 || '密码至少 5 位']"
            type="password"
            variant="outlined"
          />

          <v-text-field
            v-model="passwordForm.confirmPassword"
            label="确认密码"
            :rules="[
              v => !!v || '请确认密码',
              v => v === passwordForm.password || '两次密码不一致',
            ]"
            type="password"
            variant="outlined"
          />
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="passwordDialogVisible = false">
          取消
        </v-btn>
        <v-btn color="primary" @click="savePassword">
          保存
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
  import { useRouter } from 'vue-router'
  import { createApiClient, getErrorMessage } from '@/api'
  import { API_BASE_URL } from '@/config'

  const router = useRouter()
  const apiClient = createApiClient(API_BASE_URL)

  const userInfoDialogVisible = ref(false)
  const passwordDialogVisible = ref(false)
  const userInfoFormRef = ref<any>(null)
  const passwordFormRef = ref<any>(null)

  const userInfoForm = ref({
    name: '',
  })

  const passwordForm = ref({
    password: '',
    confirmPassword: '',
  })

  const avatarFile = ref<File | null>(null)
  const avatarPreview = ref<string | null>(null)

  const snackbar = ref({
    show: false,
    message: '',
    color: 'success',
  })

  async function loadUserInfo () {
    try {
      const userInfo = await apiClient.auth.getCurrentUser()
      userInfoForm.value.name = userInfo.name || ''
      avatarPreview.value = userInfo.avatar || null
    } catch (error) {
      console.error('Failed to load user info:', error)
    }
  }

  function openUserInfoDialog () {
    loadUserInfo()
    userInfoDialogVisible.value = true
  }

  function openPasswordDialog () {
    passwordForm.value = {
      password: '',
      confirmPassword: '',
    }
    passwordDialogVisible.value = true
  }

  function handleAvatarChange () {
    console.log('avatar change')
    console.log(avatarFile)
    if (avatarFile.value !== null) {
      console.log('avatar change valid')
      const file = avatarFile.value
      const reader = new FileReader()
      reader.addEventListener('load', e => {
        console.log(e)
        avatarPreview.value = e.target?.result as string
      })
      reader.readAsDataURL(file)
    }
  }

  async function saveUserInfo () {
    const { valid } = await userInfoFormRef.value.validate()
    if (!valid)
      return

    try {
      await apiClient.auth.updateCurrentUser({
        name: userInfoForm.value.name,
        avatar: avatarPreview.value || undefined,
      })
      snackbar.value = {
        show: true,
        message: '账户信息更新成功',
        color: 'success',
      }
      userInfoDialogVisible.value = false
      // Reload page to refresh avatar in UserInfoBar
      window.location.reload()
    } catch (error: any) {
      console.error('Failed to update user info:', error)
      snackbar.value = {
        show: true,
        message: getErrorMessage('user', error.statusCode),
        color: 'error',
      }
    }
  }

  async function savePassword () {
    const { valid } = await passwordFormRef.value.validate()
    if (!valid)
      return

    try {
      await apiClient.auth.updateCurrentUser({
        password: passwordForm.value.password,
      })
      snackbar.value = {
        show: true,
        message: '密码修改成功',
        color: 'success',
      }
      passwordDialogVisible.value = false
    } catch (error: any) {
      console.error('Failed to update password:', error)
      snackbar.value = {
        show: true,
        message: getErrorMessage('user', error.statusCode),
        color: 'error',
      }
    }
  }

  async function handleLogout () {
    try {
      await apiClient.auth.logout()
      router.push('/')
    } catch (error) {
      console.error('Logout failed:', error)
      // Even if logout API fails, redirect to login page
      router.push('/')
    }
  }

  onMounted(() => {
    loadUserInfo()
  })
</script>
