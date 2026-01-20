import { ref, shallowRef, onMounted, onUnmounted, computed, watch } from 'vue'
import type { AudioRuntime, ParameterState } from '@/runtime/types'

const runtime = shallowRef<AudioRuntime | null>(null)
const isInitialized = ref(false)
const initError = ref<string | null>(null)
let isInitializing = false

export function useRuntime() {
  const isWeb = import.meta.env.VITE_RUNTIME === 'web'

  onMounted(async () => {
    if (runtime.value || isInitializing) return
    isInitializing = true

    try {
      if (isWeb) {
        const { WebRuntime } = await import('@/runtime/WebRuntime')
        const webRuntime = new WebRuntime()
        await webRuntime.initialize()
        runtime.value = webRuntime
      } else {
        const { JuceRuntime } = await import('@/runtime/JuceRuntime')
        runtime.value = new JuceRuntime()
      }
      isInitialized.value = true
    } catch (e) {
      initError.value = e instanceof Error ? e.message : 'Unknown error'
    }
  })

  return {
    runtime: computed(() => runtime.value),
    isWeb,
    isInitialized,
    initError,
  }
}

export function useParameter(parameterId: string) {
  const normalizedValue = ref(0)
  const scaledValue = ref(0)
  const properties = ref<ParameterState['properties'] | null>(null)
  let unsubscribe: (() => void) | null = null

  const { runtime: runtimeRef, isInitialized } = useRuntime()

  const updateFromRuntime = () => {
    const rt = runtimeRef.value
    if (!rt) return

    const param = rt.getParameter(parameterId)
    if (!param) return

    normalizedValue.value = param.getNormalisedValue()
    scaledValue.value = param.getScaledValue()
    properties.value = param.properties

    unsubscribe?.()
    unsubscribe = param.onValueChanged((newNormalized) => {
      normalizedValue.value = newNormalized
      scaledValue.value = param.getScaledValue()
    })
  }

  watch(isInitialized, (newVal) => {
    if (newVal) {
      updateFromRuntime()
    }
  }, { immediate: true })

  onUnmounted(() => {
    unsubscribe?.()
  })

  const setNormalizedValue = (value: number) => {
    const rt = runtimeRef.value
    if (!rt) return

    const param = rt.getParameter(parameterId)
    param?.setNormalisedValue(value)
  }

  const displayValue = computed(() => {
    const props = properties.value
    if (!props) return '0'
    if (props.interval >= 1) return Math.round(scaledValue.value).toString()
    return scaledValue.value.toFixed(1)
  })

  return {
    normalizedValue,
    scaledValue,
    properties,
    displayValue,
    setNormalizedValue,
    isInitialized,
  }
}
