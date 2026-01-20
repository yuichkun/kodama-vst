cmake_minimum_required(VERSION 3.22 FATAL_ERROR)

project(Kodama VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Add JUCE
add_subdirectory(libs/juce)

# Add plugin
add_subdirectory(plugin)
