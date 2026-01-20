#pragma once

#include "Kodama/PluginProcessor.h"
#include <juce_gui_extra/juce_gui_extra.h>

namespace kodama {

class KodamaEditor final : public juce::AudioProcessorEditor, private juce::Timer
{
public:
    explicit KodamaEditor(KodamaProcessor&);
    ~KodamaEditor() override;

    void resized() override;

private:
    void timerCallback() override;
    using Resource = juce::WebBrowserComponent::Resource;
    std::optional<Resource> getResource(const juce::String& url);
    static const char* getMimeForExtension(const juce::String& extension);

    KodamaProcessor& processorRef;

    std::unique_ptr<juce::WebSliderRelay> delayTimeRelay;
    std::unique_ptr<juce::WebSliderRelay> feedbackRelay;
    std::unique_ptr<juce::WebSliderRelay> mixRelay;
    std::unique_ptr<juce::WebSliderRelay> voicesRelay;

    std::unique_ptr<juce::WebSliderParameterAttachment> delayTimeAttachment;
    std::unique_ptr<juce::WebSliderParameterAttachment> feedbackAttachment;
    std::unique_ptr<juce::WebSliderParameterAttachment> mixAttachment;
    std::unique_ptr<juce::WebSliderParameterAttachment> voicesAttachment;

    std::unique_ptr<juce::WebBrowserComponent> webView;

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(KodamaEditor)
};

}
