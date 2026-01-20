import { ref, onMounted, onUnmounted, computed } from 'vue'
import type { SliderState } from '@/types/juce'

export function useJuceParameter(parameterName: string) {
  const normalizedValue = ref(0)
  const scaledValue = ref(0)
  const properties = ref({
    start: 0,
    end: 1,
    skew: 1,
    name: '',
    label: '',
    numSteps: 0,
    interval: 0,
    parameterIndex: -1,
  })

  let sliderState: SliderState | null = null

  const displayValue = computed(() => {
    const val = scaledValue.value
    if (properties.value.interval >= 1) {
      return Math.round(val).toString()
    }
    return val.toFixed(1)
  })

  onMounted(() => {
    if (typeof window !== 'undefined' && window.__JUCE__) {
      sliderState = window.__JUCE__.getSliderState(parameterName)

      if (sliderState) {
        normalizedValue.value = sliderState.getNormalisedValue()
        scaledValue.value = sliderState.getScaledValue()
        properties.value = { ...sliderState.properties }

        sliderState.valueChangedCallback = (newValue: number) => {
          normalizedValue.value = newValue
          if (sliderState) {
            scaledValue.value = sliderState.getScaledValue()
          }
        }
      }
    }
  })

  onUnmounted(() => {
    if (sliderState) {
      sliderState.valueChangedCallback = null
    }
  })

  const setNormalizedValue = (value: number) => {
    const clamped = Math.max(0, Math.min(1, value))
    normalizedValue.value = clamped
    if (sliderState) {
      sliderState.setNormalisedValue(clamped)
      scaledValue.value = sliderState.getScaledValue()
    }
  }

  return {
    normalizedValue,
    scaledValue,
    properties,
    displayValue,
    setNormalizedValue,
  }
}
