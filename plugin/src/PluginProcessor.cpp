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
}

KodamaProcessor::~KodamaProcessor() = default;

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
    currentSampleRate = sampleRate;
    writePosition = 0;

    for (auto& buffer : delayBuffer)
    {
        buffer.resize(MAX_DELAY_SAMPLES, 0.0f);
        std::fill(buffer.begin(), buffer.end(), 0.0f);
    }
}

void KodamaProcessor::releaseResources()
{
    for (auto& buffer : delayBuffer)
        buffer.clear();
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

    const auto delayTimeMs = delayTimeParam->load();
    const auto feedback = feedbackParam->load() / 100.0f;
    const auto mix = mixParam->load() / 100.0f;

    const int delaySamples = static_cast<int>((delayTimeMs / 1000.0) * currentSampleRate);
    const int clampedDelay = std::clamp(delaySamples, 1, MAX_DELAY_SAMPLES - 1);

    const auto numChannels = std::min(buffer.getNumChannels(), 2);
    const auto numSamples = buffer.getNumSamples();

    for (int channel = 0; channel < numChannels; ++channel)
    {
        auto* channelData = buffer.getWritePointer(channel);
        auto& delay = delayBuffer[static_cast<size_t>(channel)];

        for (int sample = 0; sample < numSamples; ++sample)
        {
            const float drySignal = channelData[sample];

            int readPosition = writePosition - clampedDelay;
            if (readPosition < 0)
                readPosition += MAX_DELAY_SAMPLES;

            const float delayedSignal = delay[static_cast<size_t>(readPosition)];

            delay[static_cast<size_t>(writePosition)] = drySignal + (delayedSignal * feedback);

            channelData[sample] = (drySignal * (1.0f - mix)) + (delayedSignal * mix);

            if (channel == numChannels - 1)
            {
                writePosition++;
                if (writePosition >= MAX_DELAY_SAMPLES)
                    writePosition = 0;
            }
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
