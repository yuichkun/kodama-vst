<script setup lang="ts">
import { ref } from 'vue'
import KnobControl from './components/KnobControl.vue'
import WebAudioControls from './components/WebAudioControls.vue'
import WaveformDisplay from './components/WaveformDisplay.vue'
import { useRuntime } from './composables/useRuntime'
import { useWaveform } from './composables/useWaveform'

const { isWeb, isInitialized, initError } = useRuntime()

const waveformRef = ref<InstanceType<typeof WaveformDisplay> | null>(null)

useWaveform((data) => {
  waveformRef.value?.handleWaveformData(data)
})
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-950 to-slate-900 flex flex-col">
    <header class="p-6 text-center">
      <h1 class="text-3xl font-bold bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
        Kodama
      </h1>
      <p class="text-gray-500 text-sm mt-1">Digital Delay</p>
    </header>

    <div v-if="isWeb && !isInitialized && !initError" class="flex-1 flex items-center justify-center">
      <p class="text-gray-400">Initializing audio engine...</p>
    </div>

    <div v-else-if="initError" class="flex-1 flex items-center justify-center">
      <p class="text-red-400">Error: {{ initError }}</p>
    </div>

    <template v-else>
      <div v-if="isWeb" class="px-6 pb-4 flex justify-center">
        <WebAudioControls />
      </div>

      <div class="px-6 pb-4">
        <div class="h-32 rounded-lg overflow-hidden border border-white/10 bg-black">
          <WaveformDisplay ref="waveformRef" />
        </div>
      </div>

      <main class="flex-1 flex items-center justify-center pb-12">
        <div class="flex gap-12">
          <KnobControl
            parameter-id="delayTime"
            label="Time"
            color="#a855f7"
          />
          <KnobControl
            parameter-id="feedback"
            label="Feedback"
            color="#ec4899"
          />
          <KnobControl
            parameter-id="mix"
            label="Mix"
            color="#6366f1"
          />
        </div>
      </main>
    </template>
  </div>
</template>
