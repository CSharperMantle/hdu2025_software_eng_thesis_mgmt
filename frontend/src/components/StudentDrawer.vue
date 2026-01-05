<template>
  <v-navigation-drawer permanent>
    <v-list nav>
      <v-list-subheader>学生服务</v-list-subheader>
      <v-list-item
        link
        prepend-icon="mdi-archive-arrow-up"
        title="选题"
        to="/student/select"
        :variant="part === 0 ? 'tonal' : undefined"
      />
      <v-list-item
        link
        prepend-icon="mdi-swap-horizontal"
        title="进展管理"
        to="/student/progress"
        :variant="part === 1 ? 'tonal' : undefined"
      />
      <InfoDrawer />
    </v-list>
  </v-navigation-drawer>
</template>

<script lang="ts" setup>
  const { part } = defineProps<{
    part: number
  }>()
</script>
