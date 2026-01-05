<template>
	<v-navigation-drawer permanent>
		<v-list nav>
			<v-list-subheader>学生服务</v-list-subheader>
			<v-list-item link :variant="part === 0 ? 'tonal' : undefined" prepend-icon="mdi-archive-arrow-up"
				title="选题"></v-list-item>
			<v-list-item link :variant="part === 1 ? 'tonal' : undefined" prepend-icon="mdi-swap-horizontal"
				title="进展管理"></v-list-item>
			<InfoDrawer />
		</v-list>
	</v-navigation-drawer>
</template>

<script lang="ts" setup>
const {part} = defineProps<{
	part: number
}>();
</script>
