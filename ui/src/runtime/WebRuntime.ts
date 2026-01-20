import type { AudioRuntime, ParameterState, ParameterProperties, WaveformData, WaveformCallback } from './types'

interface ParameterData {
  value: number
  properties: ParameterProperties
  callbacks: Set<(value: number) => void>
}

export class WebRuntime implements AudioRuntime {
  readonly type = 'web' as const

  private audioContext: AudioContext | null = null
  private workletNode: AudioWorkletNode | null = null
  private sourceNode: AudioBufferSourceNode | null = null
  private audioBuffer: AudioBuffer | null = null
  private isPlaying = false
  private parameters: Map<string, ParameterData> = new Map()
  private waveformCallbacks: Set<WaveformCallback> = new Set()

  constructor() {
    this.parameters.set('delayTime', {
      value: 300,
      properties: { start: 0, end: 2000, name: 'Delay Time', label: 'ms', interval: 1 },
      callbacks: new Set(),
    })
    this.parameters.set('feedback', {
      value: 30,
      properties: { start: 0, end: 100, name: 'Feedback', label: '%', interval: 0.1 },
      callbacks: new Set(),
    })
    this.parameters.set('mix', {
      value: 50,
      properties: { start: 0, end: 100, name: 'Mix', label: '%', interval: 0.1 },
      callbacks: new Set(),
    })
  }

  async initialize(): Promise<void> {
    this.audioContext = new AudioContext()

    await this.audioContext.audioWorklet.addModule('/worklet/processor.js')

    this.workletNode = new AudioWorkletNode(this.audioContext, 'kodama-dsp-processor', {
      numberOfInputs: 1,
      numberOfOutputs: 1,
      outputChannelCount: [2],
    })

    this.workletNode.connect(this.audioContext.destination)

    const wasmResponse = await fetch('/wasm/kodama_dsp.wasm')
    const wasmBytes = await wasmResponse.arrayBuffer()

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('WASM init timeout')), 5000)

      this.workletNode!.port.onmessage = (event) => {
        if (event.data.type === 'wasm-ready') {
          clearTimeout(timeout)
          resolve()
        } else if (event.data.type === 'error') {
          clearTimeout(timeout)
          reject(new Error(event.data.message))
        } else if (event.data.type === 'waveform') {
          this.handleWaveformData(event.data)
        }
      }

      this.workletNode!.port.postMessage({ type: 'init-wasm', wasmBytes })
    })
  }

  getParameter(id: string): ParameterState | null {
    const param = this.parameters.get(id)
    if (!param) return null

    return {
      getNormalisedValue: () => {
        const { start, end } = param.properties
        return (param.value - start) / (end - start)
      },
      setNormalisedValue: (normalized: number) => {
        const { start, end } = param.properties
        const scaled = start + normalized * (end - start)
        this.setParameter(id, scaled)
      },
      getScaledValue: () => param.value,
      setScaledValue: (value: number) => this.setParameter(id, value),
      properties: param.properties,
      onValueChanged: (callback: (value: number) => void) => {
        param.callbacks.add(callback)
        return () => param.callbacks.delete(callback)
      },
    }
  }

  setParameter(id: string, value: number): void {
    const param = this.parameters.get(id)
    if (!param) return

    param.value = value
    this.workletNode?.port.postMessage({ type: 'set-param', param: id, value })

    const normalized =
      (value - param.properties.start) / (param.properties.end - param.properties.start)
    param.callbacks.forEach((cb) => cb(normalized))
  }

  async loadAudioFile(file: File): Promise<void> {
    if (!this.audioContext) throw new Error('Runtime not initialized')
    const arrayBuffer = await file.arrayBuffer()
    this.audioBuffer = await this.audioContext.decodeAudioData(arrayBuffer)
  }

  play(): void {
    if (!this.audioContext || !this.audioBuffer || !this.workletNode) return
    this.stop()

    this.sourceNode = this.audioContext.createBufferSource()
    this.sourceNode.buffer = this.audioBuffer
    this.sourceNode.loop = true
    this.sourceNode.connect(this.workletNode)
    this.sourceNode.start()
    this.isPlaying = true
  }

  stop(): void {
    if (this.sourceNode) {
      this.sourceNode.stop()
      this.sourceNode.disconnect()
      this.sourceNode = null
    }
    this.isPlaying = false
  }

  getIsPlaying(): boolean {
    return this.isPlaying
  }

  hasAudioLoaded(): boolean {
    return this.audioBuffer !== null
  }

  dispose(): void {
    this.stop()
    this.waveformCallbacks.clear()
    this.workletNode?.disconnect()
    this.audioContext?.close()
  }

  onWaveformData(callback: WaveformCallback): () => void {
    this.waveformCallbacks.add(callback)
    return () => this.waveformCallbacks.delete(callback)
  }

  private handleWaveformData(data: { input: Float32Array; output: Float32Array; length: number }): void {
    const waveformData: WaveformData = {
      input: data.input,
      output: data.output,
      length: data.length,
    }
    this.waveformCallbacks.forEach((cb) => cb(waveformData))
  }
}
