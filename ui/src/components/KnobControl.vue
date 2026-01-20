<script setup lang="ts">
import { ref, computed } from 'vue'
import { useParameter } from '@/composables/useRuntime'

const props = defineProps<{
  parameterId: string
  label: string
  unit?: string
}>()

const { normalizedValue, displayValue, setNormalizedValue } =
  useParameter(props.parameterId)

const isDragging = ref(false)
const startY = ref(0)
const startValue = ref(0)

const rotation = computed(() => {
  const minAngle = -135
  const maxAngle = 135
  return minAngle + normalizedValue.value * (maxAngle - minAngle)
})

// SVG arc calculations for 270° sweep (r=40)
const circumference = 2 * Math.PI * 40 // ≈ 251.33
const arcLength = circumference * (270 / 360) // ≈ 188.5

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
  <div class="knob-control w-[70px]">
    <div class="knob" @mousedown="onMouseDown">
      <svg viewBox="0 0 100 100">
        <!-- Background track arc (270°) -->
        <circle
          class="knob-track"
          cx="50"
          cy="50"
          r="40"
          fill="none"
          stroke-width="3"
          stroke-linecap="round"
          :stroke-dasharray="`${arcLength} ${circumference}`"
          stroke-dashoffset="0"
          transform="rotate(135 50 50)"
        />
        <!-- Value arc -->
        <circle
          class="knob-value-arc"
          cx="50"
          cy="50"
          r="40"
          fill="none"
          stroke-width="3"
          stroke-linecap="round"
          :stroke-dasharray="`${normalizedValue * arcLength} ${circumference}`"
          stroke-dashoffset="0"
          transform="rotate(135 50 50)"
        />
        <!-- Center circle -->
        <circle class="knob-center" cx="50" cy="50" r="28" />
        <!-- Indicator line -->
        <line
          class="knob-indicator"
          x1="50"
          y1="50"
          x2="50"
          y2="26"
          stroke-width="3"
          stroke-linecap="round"
          :transform="`rotate(${rotation} 50 50)`"
        />
      </svg>
    </div>
    <div class="knob-label text-center text-gray-700">
      <div class="knob-value text-xs">
        {{ displayValue }}<span v-if="unit" class="knob-unit">{{ unit }}</span>
      </div>
      <div class="knob-name font-bold mt-1 text-xs">{{ label }}</div>
    </div>
  </div>
</template>

<style scoped>
.knob-control {
  --knob-track-color: #3a3a4a;
  --knob-value-color: #628494;
  --knob-center-color: #2a2a3a;
  --knob-indicator-color: #628494;
}

.knob-track {
  stroke: var(--knob-track-color);
}

.knob-value-arc {
  stroke: var(--knob-value-color);
}

.knob-center {
  fill: var(--knob-center-color);
}

.knob-indicator {
  stroke: var(--knob-indicator-color);
}
</style>
