import { ref } from 'vue'

interface SnackbarState {
  show: boolean
  message: string
  color: 'success' | 'error' | 'warning' | 'info'
}

const snackbarState = ref<SnackbarState>({
  show: false,
  message: '',
  color: 'success',
})

export function useSnackbar() {
  const showSnackbar = (
    message: string,
    color: 'success' | 'error' | 'warning' | 'info' = 'success'
  ) => {
    snackbarState.value = {
      show: true,
      message,
      color,
    }
  }

  const showSuccess = (message: string) => {
    showSnackbar(message, 'success')
  }

  const showError = (message: string) => {
    showSnackbar(message, 'error')
  }

  const showWarning = (message: string) => {
    showSnackbar(message, 'warning')
  }

  const showInfo = (message: string) => {
    showSnackbar(message, 'info')
  }

  return {
    snackbarState,
    showSnackbar,
    showSuccess,
    showError,
    showWarning,
    showInfo,
  }
}
