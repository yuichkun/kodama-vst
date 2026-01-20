#include "Kodama/PluginProcessor.h"
#include "Kodama/PluginEditor.h"
#include <unordered_map>

#if !JUCE_DEBUG
#include "KodamaUIBinaryData.h"
#endif

namespace kodama {

namespace {

auto streamToVector(juce::InputStream& stream)
{
    std::vector<std::byte> result(static_cast<size_t>(stream.getTotalLength()));
    stream.setPosition(0);
    [[maybe_unused]] const auto bytesRead = stream.read(result.data(), result.size());
    jassert(bytesRead == static_cast<ssize_t>(result.size()));
    return result;
}

}

const char* KodamaEditor::getMimeForExtension(const juce::String& extension)
{
    static const std::unordered_map<juce::String, const char*> mimeMap = {
        {"htm", "text/html"},
        {"html", "text/html"},
        {"txt", "text/plain"},
        {"jpg", "image/jpeg"},
        {"jpeg", "image/jpeg"},
        {"svg", "image/svg+xml"},
        {"ico", "image/vnd.microsoft.icon"},
        {"json", "application/json"},
        {"png", "image/png"},
        {"css", "text/css"},
        {"map", "application/json"},
        {"js", "text/javascript"},
        {"mjs", "text/javascript"},
        {"woff2", "font/woff2"}};

    if (const auto it = mimeMap.find(extension.toLowerCase()); it != mimeMap.end())
        return it->second;

    jassertfalse;
    return "application/octet-stream";
}

KodamaEditor::KodamaEditor(KodamaProcessor& p)
    : AudioProcessorEditor(&p), processorRef(p)
{
    delayTimeRelay = std::make_unique<juce::WebSliderRelay>(KodamaProcessor::PARAM_DELAY_TIME);
    feedbackRelay = std::make_unique<juce::WebSliderRelay>(KodamaProcessor::PARAM_FEEDBACK);
    mixRelay = std::make_unique<juce::WebSliderRelay>(KodamaProcessor::PARAM_MIX);

    webView = std::make_unique<juce::WebBrowserComponent>(
        juce::WebBrowserComponent::Options{}
            .withNativeIntegrationEnabled()
            .withResourceProvider([this](const auto& url) { return getResource(url); })
            .withKeepPageLoadedWhenBrowserIsHidden()
            .withOptionsFrom(*delayTimeRelay)
            .withOptionsFrom(*feedbackRelay)
            .withOptionsFrom(*mixRelay));

    delayTimeAttachment = std::make_unique<juce::WebSliderParameterAttachment>(
        *processorRef.parameters.getParameter(KodamaProcessor::PARAM_DELAY_TIME),
        *delayTimeRelay,
        nullptr);

    feedbackAttachment = std::make_unique<juce::WebSliderParameterAttachment>(
        *processorRef.parameters.getParameter(KodamaProcessor::PARAM_FEEDBACK),
        *feedbackRelay,
        nullptr);

    mixAttachment = std::make_unique<juce::WebSliderParameterAttachment>(
        *processorRef.parameters.getParameter(KodamaProcessor::PARAM_MIX),
        *mixRelay,
        nullptr);

    addAndMakeVisible(*webView);
#if JUCE_DEBUG
    webView->goToURL("http://localhost:5173");
#else
    webView->goToURL(juce::WebBrowserComponent::getResourceProviderRoot());
#endif

    setSize(500, 400);
    setResizable(true, true);

    startTimerHz(60);
}

KodamaEditor::~KodamaEditor()
{
    stopTimer();
}

void KodamaEditor::resized()
{
    webView->setBounds(getLocalBounds());
}

void KodamaEditor::timerCallback()
{
    juce::Array<juce::var> inputArray;
    juce::Array<juce::var> outputArray;

    {
        juce::SpinLock::ScopedLockType lock(processorRef.waveformLock);
        const size_t writeIdx = processorRef.waveformWriteIndex;

        for (size_t i = 0; i < WAVEFORM_BUFFER_SIZE; ++i)
        {
            const size_t readIdx = (writeIdx + i) % WAVEFORM_BUFFER_SIZE;
            inputArray.add(processorRef.inputWaveformBuffer[readIdx]);
            outputArray.add(processorRef.outputWaveformBuffer[readIdx]);
        }
    }

    auto waveformData = std::make_unique<juce::DynamicObject>();
    waveformData->setProperty("input", inputArray);
    waveformData->setProperty("output", outputArray);
    waveformData->setProperty("length", static_cast<int>(WAVEFORM_BUFFER_SIZE));

    webView->emitEventIfBrowserIsVisible("waveformData", juce::var(waveformData.release()));
}

std::optional<KodamaEditor::Resource> KodamaEditor::getResource(const juce::String& url)
{
#if JUCE_DEBUG
    const juce::File resourceRoot{KODAMA_UI_DIST_PATH};
    const auto resourceToRetrieve = url == "/" ? "index.html" : url.fromFirstOccurrenceOf("/", false, false);
    const auto resourceFile = resourceRoot.getChildFile(resourceToRetrieve);
    
    if (auto stream = resourceFile.createInputStream())
    {
        const auto extension = resourceToRetrieve.fromLastOccurrenceOf(".", false, false);
        return Resource{streamToVector(*stream), getMimeForExtension(extension)};
    }

    return std::nullopt;
#else
    if (url == "/" || url == "/index.html")
    {
        return Resource{
            std::vector<std::byte>(
                reinterpret_cast<const std::byte*>(KodamaUI::index_html),
                reinterpret_cast<const std::byte*>(KodamaUI::index_html) + KodamaUI::index_htmlSize
            ),
            "text/html"
        };
    }
    return std::nullopt;
#endif
}

}
