juce_add_plugin(Kodama
    COMPANY_NAME "KodamaAudio"
    IS_SYNTH FALSE
    NEEDS_MIDI_INPUT FALSE
    NEEDS_MIDI_OUTPUT FALSE
    PLUGIN_MANUFACTURER_CODE Koda
    PLUGIN_CODE Kdma
    FORMATS AU VST3 Standalone
    PRODUCT_NAME "Kodama"
    COPY_PLUGIN_AFTER_BUILD TRUE
)

get_target_property(PRODUCT_NAME Kodama JUCE_PRODUCT_NAME)
get_target_property(COMPANY_NAME Kodama JUCE_COMPANY_NAME)

target_compile_definitions(Kodama
    PRIVATE
        JUCE_PRODUCT_NAME="${PRODUCT_NAME}"
        JUCE_COMPANY_NAME="${COMPANY_NAME}"
        JUCE_PLUGIN_VERSION="${PROJECT_VERSION}"
        KODAMA_UI_DIST_PATH="${CMAKE_CURRENT_SOURCE_DIR}/../ui/dist"
)

target_sources(Kodama
    PRIVATE
        src/PluginProcessor.cpp
        src/PluginEditor.cpp
)

target_include_directories(Kodama
    PRIVATE
        include
)

add_dependencies(Kodama kodama_dsp_rust)

target_link_libraries(Kodama
    PRIVATE
        kodama_dsp
        juce::juce_audio_utils
        juce::juce_gui_extra
    PUBLIC
        juce::juce_recommended_config_flags
        juce::juce_recommended_lto_flags
        juce::juce_recommended_warning_flags
)

if(APPLE)
    target_link_libraries(Kodama PRIVATE "-framework Security" "-framework CoreFoundation")
endif()

target_compile_definitions(Kodama
    PUBLIC
        JUCE_WEB_BROWSER=1
        JUCE_USE_CURL=0
        JUCE_VST3_CAN_REPLACE_VST2=0
)

file(COPY "${CMAKE_CURRENT_SOURCE_DIR}/../libs/juce/modules/juce_gui_extra/native/javascript/"
     DESTINATION "${CMAKE_CURRENT_SOURCE_DIR}/../ui/public/js/juce/")
