<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import type { WaveformData } from '@/runtime/types'

const canvasRef = ref<HTMLCanvasElement | null>(null)
let ctx: CanvasRenderingContext2D | null = null
let animationId: number | null = null
let dpr = 1

const BUFFER_SIZE = 512
const buffer = new Float32Array(BUFFER_SIZE)

function resizeCanvas() {
  const canvas = canvasRef.value
  if (!canvas) return

  ctx = canvas.getContext('2d')
  if (!ctx) return

  dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr
}

function render() {
  if (!ctx || !canvasRef.value) {
    animationId = requestAnimationFrame(render)
    return
  }

  const canvas = canvasRef.value
  const w = canvas.width / dpr
  const h = canvas.height / dpr

  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.fillStyle = '#fff'
  ctx.fillRect(0, 0, w, h)

  ctx.beginPath()
  ctx.strokeStyle = '#000'
  ctx.lineWidth = 0.5

  const len = buffer.length
  for (let i = 0; i < len; i++) {
    const x = (i / (len - 1)) * w
    const y = h / 2 - (buffer[i] ?? 0) * h * 0.4
    if (i === 0) ctx.moveTo(x, y)
    else ctx.lineTo(x, y)
  }
  ctx.stroke()

  animationId = requestAnimationFrame(render)
}

function handleWaveformData(data: WaveformData) {
  const len = Math.min(data.length, BUFFER_SIZE)
  for (let i = 0; i < len; i++) {
    buffer[i] = data.output[i] ?? 0
  }
}

defineExpose({ handleWaveformData })

onMounted(() => {
  resizeCanvas()
  window.addEventListener('resize', resizeCanvas)
  animationId = requestAnimationFrame(render)
})

onUnmounted(() => {
  window.removeEventListener('resize', resizeCanvas)
  if (animationId) cancelAnimationFrame(animationId)
})
</script>

<template>
  <canvas class="w-full h-full" ref="canvasRef" />
</template>
