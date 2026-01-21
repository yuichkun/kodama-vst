<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import type { WaveformData } from '@/runtime/types'

const props = withDefaults(defineProps<{
  zoom?: number  // 1-16, multiplier for 512 samples
}>(), {
  zoom: 4
})

const canvasRef = ref<HTMLCanvasElement | null>(null)
let ctx: CanvasRenderingContext2D | null = null
let animationId: number | null = null
let dpr = 1

const MAX_VOICES = 16
const CHUNK_SIZE = 512
const MAX_ZOOM = 64 * 2
const BUFFER_SIZE = CHUNK_SIZE * MAX_ZOOM
const DRAW_POINTS = 512      // Fixed draw complexity

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

function drawWaveform(buffer: Float32Array, w: number, h: number, alpha: number, displaySamples: number, voiceIndex: number, voiceCount: number) {
  if (!ctx) return

  const hue = (voiceIndex / voiceCount) * 360
  ctx.strokeStyle = `hsla(${hue}, 5%, 45%, ${alpha})`
  ctx.lineWidth = 1

  const startIdx = buffer.length - displaySamples
  const step = displaySamples / DRAW_POINTS

  // Min-max downsampling to avoid aliasing/jitter
  for (let i = 0; i < DRAW_POINTS; i++) {
    const bucketStart = startIdx + Math.floor(i * step)
    const bucketEnd = startIdx + Math.floor((i + 1) * step)

    let min = buffer[bucketStart] ?? 0
    let max = min
    for (let j = bucketStart; j < bucketEnd; j++) {
      const val = buffer[j] ?? 0
      if (val < min) min = val
      if (val > max) max = val
    }

    const x = (i / (DRAW_POINTS - 1)) * w
    const yMin = h / 2 - min * h * 0.4
    const yMax = h / 2 - max * h * 0.4

    ctx.beginPath()
    ctx.moveTo(x, yMin)
    ctx.lineTo(x, yMax)
    ctx.stroke()
  }
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

  const displaySamples = Math.min(CHUNK_SIZE * props.zoom, BUFFER_SIZE)
  const alpha = 1 / currentVoiceCount + 0.3
  for (let v = 0; v < currentVoiceCount; v++) {
    const buf = voiceBuffers[v]
    if (buf) drawWaveform(buf, w, h, alpha, displaySamples, v, currentVoiceCount)
  }

  animationId = requestAnimationFrame(render)
}

function appendToBuffer(target: Float32Array, chunk: Float32Array) {
  // Shift existing data left, append new chunk at end
  const shiftAmount = chunk.length
  target.copyWithin(0, shiftAmount)
  target.set(chunk, BUFFER_SIZE - shiftAmount)
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
        appendToBuffer(targetBuf, voiceData)
      }
    }
  } else if (data.output) {
    currentVoiceCount = 1
    const buf = voiceBuffers[0]
    if (buf) appendToBuffer(buf, data.output)
  }
}

defineExpose({ handleWaveformData, MAX_ZOOM })

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
