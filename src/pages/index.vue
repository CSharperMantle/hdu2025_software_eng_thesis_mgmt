<template>
  <AppBar />

  <div class="fill-height d-flex">
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
</template>

<script lang="ts" setup>
  import { ref } from 'vue'

  const userName = ref('')
  const passWord = ref('')

  function submit () {
    console.log('hello')
  }
</script>
