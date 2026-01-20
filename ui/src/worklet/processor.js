class KodamaDspProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.wasm = null;
    this.ready = false;
    this.leftInPtr = null;
    this.rightInPtr = null;
    this.leftOutPtr = null;
    this.rightOutPtr = null;
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

    return true;
  }
}

registerProcessor('kodama-dsp-processor', KodamaDspProcessor);
