<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import type { WaveformData } from '@/runtime/types'

const canvasRef = ref<HTMLCanvasElement | null>(null)
let ctx: CanvasRenderingContext2D | null = null
let animationId: number | null = null
let dpr = 1

const MAX_VOICES = 16
const BUFFER_SIZE = 512

const voiceBuffers: Float32Array[] = Array.from({ length: MAX_VOICES }, () => new Float32Array(BUFFER_SIZE))
let currentVoiceCount = 1

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

function drawWaveform(buffer: Float32Array, w: number, h: number, alpha: number) {
  if (!ctx) return

  ctx.beginPath()
  ctx.strokeStyle = `rgba(0, 0, 0, ${alpha})`
  ctx.lineWidth = 0.5

  const len = buffer.length
  for (let i = 0; i < len; i++) {
    const x = (i / (len - 1)) * w
    const y = h / 2 - (buffer[i] ?? 0) * h * 0.4
    if (i === 0) ctx.moveTo(x, y)
    else ctx.lineTo(x, y)
  }
  ctx.stroke()
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

  const alpha = 1 / currentVoiceCount + 0.3
  for (let v = 0; v < currentVoiceCount; v++) {
    const buf = voiceBuffers[v]
    if (buf) drawWaveform(buf, w, h, alpha)
  }

  animationId = requestAnimationFrame(render)
}

function handleWaveformData(data: WaveformData) {
  const voices = data.voiceWaveforms
  const count = data.voiceCount
  if (voices && count && count > 0) {
    currentVoiceCount = count
    for (let v = 0; v < count; v++) {
      const voiceData = voices[v]
      const targetBuf = voiceBuffers[v]
      if (voiceData && targetBuf) {
        targetBuf.set(voiceData)
      }
    }
  } else if (data.output) {
    currentVoiceCount = 1
    const buf = voiceBuffers[0]
    if (buf) buf.set(data.output)
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
