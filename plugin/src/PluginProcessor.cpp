#include "Kodama/PluginProcessor.h"
#include "Kodama/PluginEditor.h"

namespace kodama {

KodamaProcessor::KodamaProcessor()
    : AudioProcessor(BusesProperties()
                         .withInput("Input", juce::AudioChannelSet::stereo(), true)
                         .withOutput("Output", juce::AudioChannelSet::stereo(), true)),
      parameters(*this, nullptr, juce::Identifier("KodamaParameters"), createParameterLayout())
{
    delayTimeParam = parameters.getRawParameterValue(PARAM_DELAY_TIME);
    feedbackParam = parameters.getRawParameterValue(PARAM_FEEDBACK);
    mixParam = parameters.getRawParameterValue(PARAM_MIX);
    voicesParam = parameters.getRawParameterValue(PARAM_VOICES);

    dspHandle = kodama_dsp_create(44100.0f);
}

KodamaProcessor::~KodamaProcessor()
{
    if (dspHandle != nullptr)
    {
        kodama_dsp_destroy(dspHandle);
        dspHandle = nullptr;
    }
}

juce::AudioProcessorValueTreeState::ParameterLayout KodamaProcessor::createParameterLayout()
{
    std::vector<std::unique_ptr<juce::RangedAudioParameter>> params;

    params.push_back(std::make_unique<juce::AudioParameterFloat>(
        juce::ParameterID{PARAM_DELAY_TIME, 1},
        "Delay Time",
        juce::NormalisableRange<float>(0.0f, 2000.0f, 1.0f),
        300.0f,
        juce::AudioParameterFloatAttributes().withLabel("ms")));

    params.push_back(std::make_unique<juce::AudioParameterFloat>(
        juce::ParameterID{PARAM_FEEDBACK, 1},
        "Feedback",
        juce::NormalisableRange<float>(0.0f, 100.0f, 0.1f),
        30.0f,
        juce::AudioParameterFloatAttributes().withLabel("%")));

    params.push_back(std::make_unique<juce::AudioParameterFloat>(
        juce::ParameterID{PARAM_MIX, 1},
        "Mix",
        juce::NormalisableRange<float>(0.0f, 100.0f, 0.1f),
        50.0f,
        juce::AudioParameterFloatAttributes().withLabel("%")));

    params.push_back(std::make_unique<juce::AudioParameterInt>(
        juce::ParameterID{PARAM_VOICES, 1},
        "Voices",
        1, 16, 1));

    return {params.begin(), params.end()};
}

const juce::String KodamaProcessor::getName() const { return JucePlugin_Name; }
bool KodamaProcessor::acceptsMidi() const { return false; }
bool KodamaProcessor::producesMidi() const { return false; }
bool KodamaProcessor::isMidiEffect() const { return false; }
double KodamaProcessor::getTailLengthSeconds() const { return 2.0; }
int KodamaProcessor::getNumPrograms() { return 1; }
int KodamaProcessor::getCurrentProgram() { return 0; }
void KodamaProcessor::setCurrentProgram(int) {}
const juce::String KodamaProcessor::getProgramName(int) { return {}; }
void KodamaProcessor::changeProgramName(int, const juce::String&) {}

void KodamaProcessor::prepareToPlay(double sampleRate, int)
{
    if (dspHandle != nullptr)
    {
        kodama_dsp_set_sample_rate(dspHandle, static_cast<float>(sampleRate));
        kodama_dsp_reset(dspHandle);
    }
}

void KodamaProcessor::releaseResources()
{
    if (dspHandle != nullptr)
    {
        kodama_dsp_reset(dspHandle);
    }
}

bool KodamaProcessor::isBusesLayoutSupported(const BusesLayout& layouts) const
{
    if (layouts.getMainOutputChannelSet() != juce::AudioChannelSet::mono() &&
        layouts.getMainOutputChannelSet() != juce::AudioChannelSet::stereo())
        return false;

    if (layouts.getMainOutputChannelSet() != layouts.getMainInputChannelSet())
        return false;

    return true;
}

void KodamaProcessor::processBlock(juce::AudioBuffer<float>& buffer, juce::MidiBuffer&)
{
    juce::ScopedNoDenormals noDenormals;

    if (dspHandle == nullptr)
        return;

    kodama_dsp_set_delay_time(dspHandle, delayTimeParam->load());
    kodama_dsp_set_feedback(dspHandle, feedbackParam->load() / 100.0f);
    kodama_dsp_set_mix(dspHandle, mixParam->load() / 100.0f);
    kodama_dsp_set_voices(dspHandle, static_cast<uint32_t>(voicesParam->load()));

    const auto numChannels = buffer.getNumChannels();
    const auto numSamples = static_cast<size_t>(buffer.getNumSamples());

    static constexpr size_t MAX_BLOCK_SIZE = 2048;
    std::array<float, MAX_BLOCK_SIZE> inputCopy{};
    const size_t copySize = std::min(numSamples, MAX_BLOCK_SIZE);
    std::copy_n(buffer.getReadPointer(0), copySize, inputCopy.begin());

    if (numChannels >= 2)
    {
        kodama_dsp_process(
            dspHandle,
            buffer.getReadPointer(0),
            buffer.getReadPointer(1),
            buffer.getWritePointer(0),
            buffer.getWritePointer(1),
            numSamples);
    }
    else if (numChannels == 1)
    {
        kodama_dsp_process(
            dspHandle,
            buffer.getReadPointer(0),
            buffer.getReadPointer(0),
            buffer.getWritePointer(0),
            buffer.getWritePointer(0),
            numSamples);
    }

    const juce::SpinLock::ScopedTryLockType lock(waveformLock);
    if (lock.isLocked())
    {
        currentVoiceCount = kodama_dsp_get_voice_count(dspHandle);

        for (uint32_t v = 0; v < currentVoiceCount && v < MAX_VOICES; ++v)
        {
            kodama_dsp_get_voice_waveform(dspHandle, v, voiceWaveformBuffers[v].data());
        }
    }
}

bool KodamaProcessor::hasEditor() const { return true; }

juce::AudioProcessorEditor* KodamaProcessor::createEditor()
{
    return new KodamaEditor(*this);
}

void KodamaProcessor::getStateInformation(juce::MemoryBlock& destData)
{
    auto state = parameters.copyState();
    std::unique_ptr<juce::XmlElement> xml(state.createXml());
    copyXmlToBinary(*xml, destData);
}

void KodamaProcessor::setStateInformation(const void* data, int sizeInBytes)
{
    std::unique_ptr<juce::XmlElement> xmlState(getXmlFromBinary(data, sizeInBytes));
    if (xmlState && xmlState->hasTagName(parameters.state.getType()))
        parameters.replaceState(juce::ValueTree::fromXml(*xmlState));
}

}

juce::AudioProcessor* JUCE_CALLTYPE createPluginFilter()
{
    return new kodama::KodamaProcessor();
}
