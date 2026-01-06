<template>
  <v-navigation-drawer permanent>
    <v-list nav>
      <v-list-subheader>答辩委员会</v-list-subheader>
      <v-list-item
        link
        prepend-icon="mdi-account-voice"
        title="答辩评分"
        to="/defense-board/scoring"
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
