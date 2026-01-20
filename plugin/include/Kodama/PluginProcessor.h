#pragma once

#include <juce_audio_processors/juce_audio_processors.h>
#include "kodama_dsp.h"
#include <array>

namespace kodama {

constexpr size_t WAVEFORM_BUFFER_SIZE = 512;

class KodamaProcessor final : public juce::AudioProcessor
{
public:
    KodamaProcessor();
    ~KodamaProcessor() override;

    void prepareToPlay(double sampleRate, int samplesPerBlock) override;
    void releaseResources() override;

    bool isBusesLayoutSupported(const BusesLayout& layouts) const override;

    void processBlock(juce::AudioBuffer<float>&, juce::MidiBuffer&) override;
    using AudioProcessor::processBlock;

    juce::AudioProcessorEditor* createEditor() override;
    bool hasEditor() const override;

    const juce::String getName() const override;

    bool acceptsMidi() const override;
    bool producesMidi() const override;
    bool isMidiEffect() const override;
    double getTailLengthSeconds() const override;

    int getNumPrograms() override;
    int getCurrentProgram() override;
    void setCurrentProgram(int index) override;
    const juce::String getProgramName(int index) override;
    void changeProgramName(int index, const juce::String& newName) override;

    void getStateInformation(juce::MemoryBlock& destData) override;
    void setStateInformation(const void* data, int sizeInBytes) override;

    juce::AudioProcessorValueTreeState parameters;

    static constexpr const char* PARAM_DELAY_TIME = "delayTime";
    static constexpr const char* PARAM_FEEDBACK = "feedback";
    static constexpr const char* PARAM_MIX = "mix";

    juce::SpinLock waveformLock;
    std::array<float, WAVEFORM_BUFFER_SIZE> inputWaveformBuffer{};
    std::array<float, WAVEFORM_BUFFER_SIZE> outputWaveformBuffer{};
    size_t waveformWriteIndex = 0;

private:
    static juce::AudioProcessorValueTreeState::ParameterLayout createParameterLayout();

    KodamaDspHandle* dspHandle = nullptr;

    std::atomic<float>* delayTimeParam = nullptr;
    std::atomic<float>* feedbackParam = nullptr;
    std::atomic<float>* mixParam = nullptr;

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(KodamaProcessor)
};

}
