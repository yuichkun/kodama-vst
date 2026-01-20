#include "Kodama/PluginProcessor.h"
#include "Kodama/PluginEditor.h"
#include <unordered_map>

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
    webView->goToURL(juce::WebBrowserComponent::getResourceProviderRoot());

    setSize(500, 400);
    setResizable(true, true);
}

KodamaEditor::~KodamaEditor() = default;

void KodamaEditor::resized()
{
    webView->setBounds(getLocalBounds());
}

std::optional<KodamaEditor::Resource> KodamaEditor::getResource(const juce::String& url)
{
    const auto resourceFileRoot = juce::File::getSpecialLocation(
                                      juce::File::currentExecutableFile)
                                      .getParentDirectory()
                                      .getParentDirectory()
                                      .getParentDirectory()
                                      .getParentDirectory()
                                      .getChildFile("ui")
                                      .getChildFile("dist");

    auto devRoot = juce::File::getSpecialLocation(juce::File::currentExecutableFile)
                       .getParentDirectory()
                       .getParentDirectory()
                       .getParentDirectory()
                       .getParentDirectory()
                       .getParentDirectory()
                       .getParentDirectory()
                       .getChildFile("ui")
                       .getChildFile("dist");

    const auto resourceToRetrieve = url == "/" ? "index.html" : url.fromFirstOccurrenceOf("/", false, false);

    auto tryLoadResource = [&](const juce::File& root) -> std::optional<Resource> {
        const auto resource = root.getChildFile(resourceToRetrieve).createInputStream();
        if (resource)
        {
            const auto extension = resourceToRetrieve.fromLastOccurrenceOf(".", false, false);
            return Resource{streamToVector(*resource), getMimeForExtension(extension)};
        }
        return std::nullopt;
    };

    if (auto result = tryLoadResource(resourceFileRoot))
        return result;

    if (auto result = tryLoadResource(devRoot))
        return result;

    return std::nullopt;
}

}
