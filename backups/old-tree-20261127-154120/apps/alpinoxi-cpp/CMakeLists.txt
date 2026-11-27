cmake_minimum_required(VERSION 3.20)

# The launcher is a Windows program: MSVC or MinGW, no other platform.
project(AlpinoxiClient LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(NOT WIN32)
  message(FATAL_ERROR "Alpinoxi Client is a Windows program.")
endif()

# Warnings are errors here: a launcher that ships with warnings is a launcher
# that ships with bugs nobody looked at.
if(MSVC)
  add_compile_options(/W4 /permissive- /utf-8 /EHsc /MP)
  add_compile_definitions(_CRT_SECURE_NO_WARNINGS UNICODE _UNICODE NOMINMAX)
else()
  add_compile_options(-Wall -Wextra)
endif()

# ---------- the window ----------
add_executable(alpinoxi-client WIN32
  src/winmain.cpp
  src/ui.cpp
  src/app.cpp
)
target_include_directories(alpinoxi-client PRIVATE src)
target_link_libraries(alpinoxi-client PRIVATE
  comctl32 user32 gdi32 shell32 advapi32 ole32 winhttp
)

# The logo is the user's own file: it is copied beside the executable, never
# regenerated. The folder is "assets", not "build", because the build directory
# is called build too and the copy was reading the file it had just written.
add_custom_command(TARGET alpinoxi-client POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E make_directory
          "$<TARGET_FILE_DIR:alpinoxi-client>/assets"
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
          "${CMAKE_CURRENT_SOURCE_DIR}/assets/icon.ico"
          "${CMAKE_CURRENT_SOURCE_DIR}/assets/icon.png"
          "${CMAKE_CURRENT_SOURCE_DIR}/assets/tray.png"
          "$<TARGET_FILE_DIR:alpinoxi-client>/assets"
)

# ---------- the decisions, tested without a window ----------
add_executable(alpinoxi-tests
  src/tests.cpp
  src/app.cpp
)
target_include_directories(alpinoxi-tests PRIVATE src)
target_link_libraries(alpinoxi-tests PRIVATE advapi32)

enable_testing()
add_test(NAME decisions COMMAND alpinoxi-tests)
