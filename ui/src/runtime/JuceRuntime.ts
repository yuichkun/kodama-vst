import type { AudioRuntime, ParameterState, WaveformData, WaveformCallback } from './types'
import { getSliderState } from '@/juce/index.js'

export class JuceRuntime implements AudioRuntime {
  readonly type = 'juce' as const
  private waveformCallbacks: Set<WaveformCallback> = new Set()
  private waveformUnsubscribe: (() => void) | null = null

  getParameter(id: string): ParameterState | null {
    if (typeof window === 'undefined' || !window.__JUCE__) return null

    const sliderState = getSliderState(id)
    if (!sliderState) return null

    return {
      getNormalisedValue: () => sliderState.getNormalisedValue(),
      setNormalisedValue: (value: number) => sliderState.setNormalisedValue(value),
      getScaledValue: () => sliderState.getScaledValue(),
      setScaledValue: (value: number) => sliderState.setScaledValue(value),
      properties: sliderState.properties,
      onValueChanged: (callback: (value: number) => void) => {
        const listenerId = sliderState.valueChangedEvent.addListener(() => {
          callback(sliderState.getNormalisedValue())
        })
        return () => {
          sliderState.valueChangedEvent.removeListener(listenerId)
        }
      },
    }
  }

  setParameter(id: string, value: number): void {
    const param = this.getParameter(id)
    param?.setScaledValue(value)
  }

  async loadAudioFile(): Promise<void> {}
  play(): void {}
  stop(): void {}
  getIsPlaying(): boolean {
    return false
  }
  hasAudioLoaded(): boolean {
    return false
  }
  dispose(): void {
    this.waveformCallbacks.clear()
    this.waveformUnsubscribe?.()
    this.waveformUnsubscribe = null
  }

  onWaveformData(callback: WaveformCallback): () => void {
    this.waveformCallbacks.add(callback)

    if (!this.waveformUnsubscribe && window.__JUCE__?.backend) {
      this.waveformUnsubscribe = window.__JUCE__.backend.addEventListener(
        'waveformData',
        (data) => {
          const typedData = data as {
            input: number[]
            output: number[]
            voiceWaveforms?: number[][]
            voiceCount?: number
            length: number
          }

          const voiceWaveforms = typedData.voiceWaveforms?.map((arr) => new Float32Array(arr))

          const waveformData: WaveformData = {
            input: new Float32Array(typedData.input),
            output: new Float32Array(typedData.output),
            voiceWaveforms,
            voiceCount: typedData.voiceCount,
            length: typedData.length,
          }
          this.waveformCallbacks.forEach((cb) => cb(waveformData))
        }
      )
    }

    return () => {
      this.waveformCallbacks.delete(callback)
      if (this.waveformCallbacks.size === 0) {
        this.waveformUnsubscribe?.()
        this.waveformUnsubscribe = null
      }
    }
  }
}
