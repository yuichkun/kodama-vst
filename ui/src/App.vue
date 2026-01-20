<script setup lang="ts">
import KnobControl from './components/KnobControl.vue'
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-950 to-slate-900 flex flex-col">
    <header class="p-6 text-center">
      <h1 class="text-3xl font-bold bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
        Kodama
      </h1>
      <p class="text-gray-500 text-sm mt-1">Digital Delay</p>
    </header>

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
  </div>
</template>
