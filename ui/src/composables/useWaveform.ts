import { ref, onUnmounted, watch } from 'vue'
import type { WaveformData, WaveformCallback } from '@/runtime/types'
import { useRuntime } from './useRuntime'

export function useWaveform(callback?: WaveformCallback) {
  const { runtime, isInitialized } = useRuntime()
  const waveformData = ref<WaveformData | null>(null)
  let unsubscribe: (() => void) | null = null

  watch(isInitialized, (initialized) => {
    if (!initialized) return

    const rt = runtime.value
    if (!rt?.onWaveformData) return

    unsubscribe?.()
    unsubscribe = rt.onWaveformData((data) => {
      waveformData.value = data
      callback?.(data)
    })
  }, { immediate: true })

  onUnmounted(() => {
    unsubscribe?.()
  })

  return { waveformData }
}
