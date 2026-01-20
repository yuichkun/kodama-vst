cmake_minimum_required(VERSION 3.22 FATAL_ERROR)

project(Kodama VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(KODAMA_DSP_DIR "${CMAKE_SOURCE_DIR}/dsp")

if(APPLE)
    if(CMAKE_OSX_ARCHITECTURES STREQUAL "arm64" OR CMAKE_SYSTEM_PROCESSOR STREQUAL "arm64")
        set(RUST_TARGET "aarch64-apple-darwin")
    else()
        set(RUST_TARGET "x86_64-apple-darwin")
    endif()
elseif(WIN32)
    set(RUST_TARGET "x86_64-pc-windows-msvc")
else()
    if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
        set(RUST_TARGET "aarch64-unknown-linux-gnu")
    else()
        set(RUST_TARGET "x86_64-unknown-linux-gnu")
    endif()
endif()

find_program(CARGO_EXECUTABLE cargo HINTS "$ENV{HOME}/.cargo/bin")
if(NOT CARGO_EXECUTABLE)
    message(FATAL_ERROR "Cargo not found. Please install Rust: https://rustup.rs")
endif()

add_custom_target(kodama_dsp_rust
    COMMAND ${CARGO_EXECUTABLE} build --release
    WORKING_DIRECTORY ${KODAMA_DSP_DIR}
    COMMENT "Building Rust DSP library"
)

if(WIN32)
    set(KODAMA_DSP_LIB "${KODAMA_DSP_DIR}/target/release/kodama_dsp.lib")
else()
    set(KODAMA_DSP_LIB "${KODAMA_DSP_DIR}/target/release/libkodama_dsp.a")
endif()

add_library(kodama_dsp STATIC IMPORTED GLOBAL)
set_target_properties(kodama_dsp PROPERTIES
    IMPORTED_LOCATION "${KODAMA_DSP_LIB}"
)
add_dependencies(kodama_dsp kodama_dsp_rust)

add_subdirectory(libs/juce)
add_subdirectory(plugin)
