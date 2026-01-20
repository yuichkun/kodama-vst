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
    <!-- Hamburger Button -->
    <button
      @click="toggleMenu"
      class="hamburger-btn fixed top-4 right-4 z-50 w-11 h-11 flex flex-col items-center justify-center gap-1.5 bg-slate-800/90 backdrop-blur-sm rounded-lg border border-slate-700/50 hover:bg-slate-700/90 transition-all duration-200 shadow-lg"
      :class="{ 'is-open': isMenuOpen }"
      aria-label="Toggle audio menu"
    >
      <span class="hamburger-line w-5 h-0.5 bg-gray-300 rounded-full transition-all duration-300 origin-center"></span>
      <span class="hamburger-line w-5 h-0.5 bg-gray-300 rounded-full transition-all duration-300 origin-center"></span>
      <span class="hamburger-line w-5 h-0.5 bg-gray-300 rounded-full transition-all duration-300 origin-center"></span>
    </button>

    <!-- Overlay -->
    <Transition name="fade">
      <div
        v-if="isMenuOpen"
        @click="closeMenu"
        class="fixed inset-0 z-40 bg-black/60 backdrop-blur-sm"
      />
    </Transition>

    <!-- Slide-in Menu Panel -->
    <Transition name="slide">
      <div
        v-if="isMenuOpen"
        class="fixed top-0 right-0 z-45 h-full w-72 max-w-[85vw] bg-slate-900/95 backdrop-blur-md border-l border-slate-700/50 shadow-2xl"
      >
        <div class="p-6 pt-20">
          <h3 class="text-sm font-medium text-gray-400 mb-4 uppercase tracking-wider">Audio Source</h3>

          <input
            ref="fileInput"
            type="file"
            accept="audio/*"
            class="hidden"
            @change="handleFileSelect"
          />

          <div class="space-y-4">
            <button
              @click="openFilePicker"
              class="w-full px-4 py-3 bg-purple-600 hover:bg-purple-500 text-white text-sm font-medium rounded-lg transition-colors duration-200 shadow-md hover:shadow-purple-500/20"
            >
              Choose File
            </button>

            <div class="px-1 py-2 text-sm text-gray-400 border-b border-slate-700/50">
              <span v-if="fileName" class="text-gray-300 break-all">
                {{ fileName }}
              </span>
              <span v-else class="text-gray-500 italic">No file selected</span>
            </div>

            <button
              @click="togglePlayback"
              :disabled="!hasAudio"
              :class="[
                'w-full py-3 rounded-lg text-sm font-medium transition-all duration-200 shadow-md',
                hasAudio
                  ? isPlaying
                    ? 'bg-red-600 hover:bg-red-500 text-white hover:shadow-red-500/20'
                    : 'bg-green-600 hover:bg-green-500 text-white hover:shadow-green-500/20'
                  : 'bg-slate-700 text-gray-500 cursor-not-allowed shadow-none',
              ]"
            >
              {{ isPlaying ? 'Stop' : 'Play' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* Hamburger animation */
.hamburger-btn.is-open .hamburger-line:nth-child(1) {
  transform: translateY(8px) rotate(45deg);
}

.hamburger-btn.is-open .hamburger-line:nth-child(2) {
  opacity: 0;
  transform: scaleX(0);
}

.hamburger-btn.is-open .hamburger-line:nth-child(3) {
  transform: translateY(-8px) rotate(-45deg);
}

/* Overlay fade transition */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* Panel slide transition */
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}

/* z-index for panel between overlay and button */
.z-45 {
  z-index: 45;
}
</style>
