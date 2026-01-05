<template>
  <v-navigation-drawer permanent>
    <v-list nav>
      <v-list-subheader>教师服务</v-list-subheader>
      <v-list-item
        link
        prepend-icon="mdi-book-plus"
        title="新建课题"
        :variant="part === 0 ? 'tonal' : undefined"
      />
      <v-list-item
        link
        prepend-icon="mdi-account-multiple-check"
        title="选题审核"
        :variant="part === 1 ? 'tonal' : undefined"
      />
      <v-list-item
        link
        prepend-icon="mdi-file-document-edit"
        title="进展审核"
        :variant="part === 2 ? 'tonal' : undefined"
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
