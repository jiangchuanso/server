cmake_minimum_required(VERSION 3.15...4.0)

project(translation CXX C)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Newer versions of Cmake require this to compile
set(CMAKE_POLICY_VERSION_MINIMUM "3.5" CACHE STRING "Minimum CMake policy version" FORCE)

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

if(CMAKE_BUILD_TYPE STREQUAL "Release")
    # Some versions of GCC will cause uninitialized errors during optimization
    add_compile_options(-Wno-error=maybe-uninitialized)
    add_compile_options(-Wno-error=uninitialized)
endif()

set(BERGAMOT_SUBMODULE_PATH "$ENV{BERGAMOT_SUBMODULE_PATH}")

if(EXISTS "${BERGAMOT_SUBMODULE_PATH}/CMakeLists.txt")
    add_subdirectory(${BERGAMOT_SUBMODULE_PATH} bergamot-translator)
else()
    include(FetchContent)
    FetchContent_Declare(
        bergamot-translator
        GIT_REPOSITORY https://github.com/browsermt/bergamot-translator.git
        GIT_TAG main)
    FetchContent_MakeAvailable(bergamot-translator)
endif()

add_library(translation STATIC translation.cpp)
target_include_directories(translation PUBLIC
    $<TARGET_PROPERTY:bergamot-translator,INTERFACE_INCLUDE_DIRECTORIES>
)
target_link_libraries(translation bergamot-translator pthread)
