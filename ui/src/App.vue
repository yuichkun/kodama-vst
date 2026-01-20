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
  <div class="app text-black">
    <header class="mt-8 text-center">
      <h1 class="mb-2 text-3xl font-bold text-[#202c32] font-['Limelight']">kodama</h1>
      <p class="text-xs">Digital Delay</p>
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
        <WaveformDisplay ref="waveformRef" />
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
