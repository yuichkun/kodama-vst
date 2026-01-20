export interface ParameterProperties {
  start: number
  end: number
  name: string
  label: string
  interval: number
  skew?: number
  numSteps?: number
  parameterIndex?: number
}

export interface ParameterState {
  getNormalisedValue(): number
  setNormalisedValue(value: number): void
  getScaledValue(): number
  setScaledValue(value: number): void
  properties: ParameterProperties
  onValueChanged(callback: (value: number) => void): () => void
}

export interface AudioRuntime {
  readonly type: 'juce' | 'web'
  getParameter(id: string): ParameterState | null
  setParameter(id: string, value: number): void
  loadAudioFile?(file: File): Promise<void>
  play?(): void
  stop?(): void
  getIsPlaying?(): boolean
  hasAudioLoaded?(): boolean
  dispose?(): void
}
