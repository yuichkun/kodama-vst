<script setup lang="ts">
import { ref, computed } from 'vue'
import { useParameter } from '@/composables/useRuntime'

const props = defineProps<{
  parameterId: string
  label: string
  color?: string
}>()

const { normalizedValue, displayValue, properties, setNormalizedValue } =
  useParameter(props.parameterId)

const isDragging = ref(false)
const startY = ref(0)
const startValue = ref(0)

const rotation = computed(() => {
  const minAngle = -135
  const maxAngle = 135
  return minAngle + normalizedValue.value * (maxAngle - minAngle)
})

const accentColor = computed(() => props.color ?? '#8b5cf6')

const onMouseDown = (e: MouseEvent) => {
  isDragging.value = true
  startY.value = e.clientY
  startValue.value = normalizedValue.value
  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('mouseup', onMouseUp)
}

const onMouseMove = (e: MouseEvent) => {
  if (!isDragging.value) return
  const delta = (startY.value - e.clientY) / 150
  const newValue = Math.max(0, Math.min(1, startValue.value + delta))
  setNormalizedValue(newValue)
}

const onMouseUp = () => {
  isDragging.value = false
  window.removeEventListener('mousemove', onMouseMove)
  window.removeEventListener('mouseup', onMouseUp)
}
</script>

<template>
  <div class="flex flex-col items-center gap-2 select-none">
    <div class="relative w-20 h-20 cursor-pointer" @mousedown="onMouseDown">
      <svg viewBox="0 0 100 100" class="w-full h-full">
        <circle
          cx="50"
          cy="50"
          r="40"
          fill="none"
          stroke="#374151"
          stroke-width="8"
          stroke-linecap="round"
          stroke-dasharray="226.2"
          stroke-dashoffset="56.55"
          transform="rotate(135 50 50)"
        />
        <circle
          cx="50"
          cy="50"
          r="40"
          fill="none"
          :stroke="accentColor"
          stroke-width="8"
          stroke-linecap="round"
          :stroke-dasharray="`${normalizedValue * 169.65} 226.2`"
          stroke-dashoffset="0"
          transform="rotate(135 50 50)"
          class="transition-all duration-75"
        />
        <circle cx="50" cy="50" r="28" fill="#1f2937" />
        <line
          x1="50"
          y1="50"
          x2="50"
          y2="28"
          :stroke="accentColor"
          stroke-width="3"
          stroke-linecap="round"
          :transform="`rotate(${rotation} 50 50)`"
          class="transition-transform duration-75"
        />
      </svg>
    </div>
    <div class="text-center">
      <div class="text-lg font-medium text-white">
        {{ displayValue
        }}<span class="text-xs text-gray-400 ml-1">{{ properties?.label ?? '' }}</span>
      </div>
      <div class="text-sm text-gray-400">{{ label }}</div>
    </div>
  </div>
</template>
