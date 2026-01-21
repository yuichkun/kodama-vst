const WAVEFORM_BUFFER_SIZE = 512;
const WAVEFORM_SEND_INTERVAL = 4;
const MAX_VOICES = 16;

class KodamaDspProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.wasm = null;
    this.ready = false;
    this.leftInPtr = null;
    this.rightInPtr = null;
    this.leftOutPtr = null;
    this.rightOutPtr = null;
    this.voiceWaveformPtr = null;

    this.inputRingBuffer = new Float32Array(WAVEFORM_BUFFER_SIZE);
    this.outputRingBuffer = new Float32Array(WAVEFORM_BUFFER_SIZE);
    this.ringWriteIndex = 0;
    this.processCounter = 0;

    this.port.onmessage = (event) => this.handleMessage(event.data);
  }

  async handleMessage(data) {
    switch (data.type) {
      case 'init-wasm':
        await this.initWasm(data.wasmBytes);
        break;
      case 'set-param':
        this.setParameter(data.param, data.value);
        break;
    }
  }

  async initWasm(wasmBytes) {
    try {
      const wasmModule = await WebAssembly.compile(wasmBytes);
      const instance = await WebAssembly.instantiate(wasmModule, {});
      this.wasm = instance.exports;
      this.wasm.wasm_init(sampleRate);

      const BLOCK_SIZE = 128;
      this.leftInPtr = this.wasm.wasm_alloc(BLOCK_SIZE);
      this.rightInPtr = this.wasm.wasm_alloc(BLOCK_SIZE);
      this.leftOutPtr = this.wasm.wasm_alloc(BLOCK_SIZE);
      this.rightOutPtr = this.wasm.wasm_alloc(BLOCK_SIZE);
      this.voiceWaveformPtr = this.wasm.wasm_alloc(WAVEFORM_BUFFER_SIZE);

      this.ready = true;
      this.port.postMessage({ type: 'wasm-ready' });
    } catch (error) {
      this.port.postMessage({ type: 'error', message: error.message });
    }
  }

  setParameter(param, value) {
    if (!this.wasm) return;
    switch (param) {
      case 'delayTime':
        this.wasm.wasm_set_delay_time(value);
        break;
      case 'feedback':
        this.wasm.wasm_set_feedback(value / 100.0);
        break;
      case 'mix':
        this.wasm.wasm_set_mix(value / 100.0);
        break;
      case 'voices':
        this.wasm.wasm_set_voices(value);
        break;
    }
  }

  process(inputs, outputs) {
    if (!this.ready || !this.wasm) return true;

    const input = inputs[0];
    const output = outputs[0];
    if (!input || input.length === 0 || !input[0]) return true;

    const leftIn = input[0];
    const rightIn = input[1] || input[0];
    const leftOut = output[0];
    const rightOut = output[1] || output[0];
    const numSamples = leftIn.length;

    const memory = this.wasm.memory;
    const leftInView = new Float32Array(memory.buffer, this.leftInPtr, numSamples);
    const rightInView = new Float32Array(memory.buffer, this.rightInPtr, numSamples);
    const leftOutView = new Float32Array(memory.buffer, this.leftOutPtr, numSamples);
    const rightOutView = new Float32Array(memory.buffer, this.rightOutPtr, numSamples);

    leftInView.set(leftIn);
    rightInView.set(rightIn);

    this.wasm.wasm_process(
      this.leftInPtr,
      this.rightInPtr,
      this.leftOutPtr,
      this.rightOutPtr,
      numSamples
    );

    leftOut.set(leftOutView);
    rightOut.set(rightOutView);

    this.updateWaveformBuffer(leftIn, leftOut, numSamples);

    return true;
  }

  updateWaveformBuffer(inputData, outputData, numSamples) {
    for (let i = 0; i < numSamples; i++) {
      if (this.ringWriteIndex < WAVEFORM_BUFFER_SIZE) {
        this.inputRingBuffer[this.ringWriteIndex] = inputData[i];
        this.outputRingBuffer[this.ringWriteIndex] = outputData[i];
        this.ringWriteIndex++;
      }
    }

    this.processCounter++;
    if (this.processCounter >= WAVEFORM_SEND_INTERVAL) {
      this.processCounter = 0;
      this.sendWaveformData();
      this.ringWriteIndex = 0;
    }
  }

  sendWaveformData() {
    const voiceCount = this.wasm.wasm_get_voice_count();
    const memory = this.wasm.memory;

    const voiceWaveforms = [];
    for (let v = 0; v < voiceCount; v++) {
      this.wasm.wasm_get_voice_waveform(v, this.voiceWaveformPtr);
      const waveformView = new Float32Array(memory.buffer, this.voiceWaveformPtr, WAVEFORM_BUFFER_SIZE);
      voiceWaveforms.push(new Float32Array(waveformView));
    }

    const transferList = [this.inputRingBuffer.buffer, this.outputRingBuffer.buffer];
    voiceWaveforms.forEach(arr => transferList.push(arr.buffer));

    this.port.postMessage(
      {
        type: 'waveform',
        input: this.inputRingBuffer,
        output: this.outputRingBuffer,
        voiceWaveforms: voiceWaveforms,
        voiceCount: voiceCount,
        length: WAVEFORM_BUFFER_SIZE
      },
      transferList
    );

    this.inputRingBuffer = new Float32Array(WAVEFORM_BUFFER_SIZE);
    this.outputRingBuffer = new Float32Array(WAVEFORM_BUFFER_SIZE);
  }
}

registerProcessor('kodama-dsp-processor', KodamaDspProcessor);
