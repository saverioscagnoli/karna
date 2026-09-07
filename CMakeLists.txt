set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

cmake_minimum_required(VERSION 3.16)
project(myapp C)

set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED ON)

# --- SDL3 -------------------------------------------------
set(SDL_SHARED       ON  CACHE BOOL "" FORCE)
set(SDL_STATIC       OFF CACHE BOOL "" FORCE)
set(SDL_TEST_LIBRARY OFF CACHE BOOL "" FORCE)
set(SDL_INSTALL      OFF CACHE BOOL "" FORCE)
add_subdirectory(vendor/SDL EXCLUDE_FROM_ALL)

# --- app --------------------------------------------------
file(GLOB_RECURSE MYAPP_SOURCES CONFIGURE_DEPENDS src/*.c logging/*.c)

add_executable(myapp ${MYAPP_SOURCES})
target_include_directories(myapp PRIVATE src logging math ${CMAKE_SOURCE_DIR})
target_link_libraries(myapp PRIVATE SDL3::SDL3)

# copy SDL3.dll next to the exe on Windows
if(WIN32)
  add_custom_command(TARGET myapp POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<TARGET_RUNTIME_DLLS:myapp> $<TARGET_FILE_DIR:myapp>
    COMMAND_EXPAND_LISTS)
endif()
