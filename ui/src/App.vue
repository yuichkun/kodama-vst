<script setup lang="ts">
import { ref, computed } from 'vue'
import KnobControl from './components/KnobControl.vue'
import WebAudioControls from './components/WebAudioControls.vue'
import WaveformDisplay from './components/WaveformDisplay.vue'
import { useRuntime } from './composables/useRuntime'
import { useWaveform } from './composables/useWaveform'

const { isWeb, isInitialized, initError } = useRuntime()

const waveformRef = ref<InstanceType<typeof WaveformDisplay> | null>(null)
const waveformSpeed = ref(61)  // 1=slow, 64=fast

const maxZoom = computed(() => waveformRef.value?.MAX_ZOOM ?? 64)
const waveformZoom = computed(() => maxZoom.value - waveformSpeed.value + 1)

useWaveform((data) => {
  waveformRef.value?.handleWaveformData(data)
})
</script>

<template>
  <div class="app text-black">
    <header class="mt-8 text-center grid place-content-center">
      <h1 class="mb-2 text-3xl font-bold text-[#202c32] font-['Limelight']">kodama</h1>
      <div class="flex items-center gap-2 text-[8px] text-gray-500">
        <span>Zoom</span>
        <input
          type="range"
          v-model.number="waveformSpeed"
          min="1"
          :max="maxZoom"
          step="1"
          class="speed-slider w-24 accent-gray-400"
        />
      </div>
    </header>

    <div v-if="isWeb && !isInitialized && !initError">
      <p>Initializing audio engine...</p>
    </div>

    <div v-else-if="initError">
      <p>Error: {{ initError }}</p>
    </div>

    <template v-else>
      <WebAudioControls v-if="isWeb" />

      <div class="-z-10 fixed top-0 left-0 w-screen h-screen">
        <WaveformDisplay ref="waveformRef" :zoom="waveformZoom" />
      </div>

      <main class="flex justify-center gap-12 fixed bottom-0 left-0 w-full mb-10">
        <KnobControl parameter-id="voices" label="Voices" />
        <KnobControl parameter-id="delayTime" label="Time" unit="ms" />
        <KnobControl parameter-id="feedback" label="Feedback" unit="%" />
        <KnobControl parameter-id="mix" label="Mix" unit="%" />
      </main>
    </template>
  </div>
</template>

<style scoped>
.speed-slider {
  -webkit-appearance: none;
  appearance: none;
  height: 4px;
  background: transparent;
}

.speed-slider::-webkit-slider-runnable-track {
  height: 2px;
  background: #9ca3af;
  border-radius: 1px;
}

.speed-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 10px;
  height: 10px;
  background: #6b7280;
  border-radius: 50%;
  margin-top: -4px;
  cursor: pointer;
}

.speed-slider::-moz-range-track {
  height: 2px;
  background: #9ca3af;
  border-radius: 1px;
}

.speed-slider::-moz-range-thumb {
  width: 10px;
  height: 10px;
  background: #6b7280;
  border-radius: 50%;
  border: none;
  cursor: pointer;
}
</style>
