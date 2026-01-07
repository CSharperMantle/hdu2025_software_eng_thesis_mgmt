<template>
  <v-navigation-drawer permanent>
    <v-list nav>
      <v-list-subheader>教务处服务</v-list-subheader>
      <v-list-item
        link
        prepend-icon="mdi-clipboard-check"
        title="课题审核"
        to="/office/topics"
        :variant="part === 0 ? 'tonal' : undefined"
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
