<script setup lang="ts">
import { ref } from 'vue'
import { useRuntime } from '@/composables/useRuntime'

const { runtime } = useRuntime()
const fileInput = ref<HTMLInputElement | null>(null)
const fileName = ref<string | null>(null)
const isPlaying = ref(false)
const hasAudio = ref(false)
const isMenuOpen = ref(false)

const handleFileSelect = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file || !runtime.value) return

  try {
    await runtime.value.loadAudioFile?.(file)
    fileName.value = file.name
    hasAudio.value = true
  } catch (e) {
    console.error('Failed to load audio file:', e)
  }
}

const togglePlayback = () => {
  if (!runtime.value) return

  if (isPlaying.value) {
    runtime.value.stop?.()
    isPlaying.value = false
  } else {
    runtime.value.play?.()
    isPlaying.value = true
  }
}

const openFilePicker = () => {
  fileInput.value?.click()
}

const toggleMenu = () => {
  isMenuOpen.value = !isMenuOpen.value
}

const closeMenu = () => {
  isMenuOpen.value = false
}
</script>

<template>
  <div class="web-audio-controls">
    <button
      @click="toggleMenu"
      class="hamburger-btn"
      :class="{ 'is-open': isMenuOpen }"
    >
      <span class="hamburger-line"></span>
      <span class="hamburger-line"></span>
      <span class="hamburger-line"></span>
    </button>

    <Transition name="fade">
      <div v-if="isMenuOpen" @click="closeMenu" class="overlay" />
    </Transition>

    <Transition name="slide">
      <div v-if="isMenuOpen" class="panel">
        <div class="panel-content">
          <h3>Audio Source</h3>

          <input
            ref="fileInput"
            type="file"
            accept="audio/*"
            class="file-input"
            @change="handleFileSelect"
          />

          <button @click="openFilePicker" class="btn btn-primary">
            Choose File
          </button>

          <div class="file-name">
            <span v-if="fileName">{{ fileName }}</span>
            <span v-else class="no-file">No file selected</span>
          </div>

          <button
            @click="togglePlayback"
            :disabled="!hasAudio"
            class="btn"
            :class="hasAudio ? (isPlaying ? 'btn-stop' : 'btn-play') : 'btn-disabled'"
          >
            {{ isPlaying ? 'Stop' : 'Play' }}
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.hamburger-btn {
  position: fixed;
  top: 12px;
  right: 12px;
  z-index: 50;
  width: 40px;
  height: 40px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 5px;
  background: #222;
  border: 1px solid #333;
  border-radius: 6px;
  cursor: pointer;
}

.hamburger-line {
  width: 18px;
  height: 2px;
  background: #888;
  border-radius: 1px;
  transition: all 0.3s;
}

.hamburger-btn.is-open .hamburger-line:nth-child(1) {
  transform: translateY(7px) rotate(45deg);
}

.hamburger-btn.is-open .hamburger-line:nth-child(2) {
  opacity: 0;
}

.hamburger-btn.is-open .hamburger-line:nth-child(3) {
  transform: translateY(-7px) rotate(-45deg);
}

.overlay {
  position: fixed;
  inset: 0;
  z-index: 40;
  background: rgba(0, 0, 0, 0.5);
}

.panel {
  position: fixed;
  top: 0;
  right: 0;
  z-index: 45;
  height: 100%;
  width: 260px;
  max-width: 85vw;
  background: #1a1a1a;
  border-left: 1px solid #333;
}

.panel-content {
  padding: 60px 20px 20px;
}

.panel-content h3 {
  font-size: 12px;
  color: #666;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 16px;
}

.file-input {
  display: none;
}

.btn {
  width: 100%;
  padding: 10px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  margin-bottom: 12px;
}

.btn-primary {
  background: #333;
  color: #fff;
}

.btn-primary:hover {
  background: #444;
}

.btn-play {
  background: #2a4a2a;
  color: #8f8;
}

.btn-play:hover {
  background: #3a5a3a;
}

.btn-stop {
  background: #4a2a2a;
  color: #f88;
}

.btn-stop:hover {
  background: #5a3a3a;
}

.btn-disabled {
  background: #222;
  color: #555;
  cursor: not-allowed;
}

.file-name {
  padding: 8px 0;
  margin-bottom: 12px;
  font-size: 13px;
  color: #aaa;
  border-bottom: 1px solid #333;
  word-break: break-all;
}

.no-file {
  color: #555;
  font-style: italic;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-enter-active,
.slide-leave-active {
  transition: transform 0.25s ease-out;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}
</style>
