<template>
  <v-app>
    <v-main>
      <router-view />
    </v-main>

    <v-snackbar
      v-model="snackbarState.show"
      :color="snackbarState.color"
      location="right"
      :timeout="3000"
    >
      {{ snackbarState.message }}
    </v-snackbar>
  </v-app>
</template>

<script lang="ts" setup>
  import { useSnackbar } from '@/composables/useSnackbar'

  const { snackbarState } = useSnackbar()
</script>
