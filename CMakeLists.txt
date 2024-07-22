cmake_minimum_required(VERSION 3.16)
project(Oracle LANGUAGES CXX)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
option(BUILD_SHARED_LIBS "Build shared libraries" OFF)

include(FetchContent)
FetchContent_Declare(SFML
    GIT_REPOSITORY https://github.com/SFML/SFML.git
    GIT_TAG 2.6.x)
FetchContent_MakeAvailable(SFML)

# Include directories
include_directories(include)

add_executable(Oracle
    src/main.cpp
    include/display.h
    src/display.cpp
    include/bitboards.h
    src/bitboards.cpp
    include/position.h
    src/position.cpp
)

# Specify include directories for the target
target_include_directories(Oracle PRIVATE include)

target_link_libraries(Oracle PRIVATE sfml-graphics)
target_compile_features(Oracle PRIVATE cxx_std_17)

if(WIN32)
    add_custom_command(
        TARGET Oracle
        COMMENT "Copy OpenAL DLL"
        PRE_BUILD COMMAND ${CMAKE_COMMAND} -E copy ${SFML_SOURCE_DIR}/extlibs/bin/$<IF:$<EQUAL:${CMAKE_SIZEOF_VOID_P},8>,x64,x86>/openal32.dll $<TARGET_FILE_DIR:Oracle>
        VERBATIM)
endif()

install(TARGETS Oracle)
