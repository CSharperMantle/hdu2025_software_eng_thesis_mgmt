<template>
	<AppBar />
	<StudentDrawer :part="currentPart" />

	<v-sheet class="w-100 d-flex justify-space-between pa-3 elevation-2 rounded-0">
		<span>
			hello_left
		</span>
		<span>
			hello_right
		</span>
	</v-sheet>
	<div class="fill-height d-flex">
		<v-card class="ma-auto w-75">
			<template v-slot:title>
				<span class="font-weight-black">登录</span>
			</template>

			<v-card-text class="pt-4">
				<v-sheet class="mx-auto">
					<v-form @submit.prevent="submit">
						<v-text-field v-model="userName" label="用户名"></v-text-field>
						<v-text-field v-model="passWord" type="password" label="密码"></v-text-field>

						<v-btn class="mt-2" text="Login" type="submit" block></v-btn>
					</v-form>
				</v-sheet>
			</v-card-text>
		</v-card>
	</div>
</template>

<script lang="ts" setup>
import {ref} from 'vue'

const userName = ref('')
const passWord = ref('')
const currentPart = 0

const submit = () => {
	console.log("hello");
}
</script>
