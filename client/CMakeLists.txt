cmake_minimum_required(VERSION 3.20)

project(AlpinoxiClient LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(MSVC)
  # /utf-8: the source is UTF-8, and MSVC reads it as the ANSI code page
  # otherwise, which turns the Turkish text into mojibake.
  add_compile_options(/W4 /permissive- /utf-8 /EHsc)
endif()

set(ALPINOXI_LIBS comctl32 user32 gdi32 shell32 advapi32 ole32 winhttp)

# The launcher's decisions, kept apart from the window so the tests can link
# them with no UI at all.
add_library(alpinoxi_core STATIC src/app.cpp)
target_include_directories(alpinoxi_core PUBLIC src)
target_link_libraries(alpinoxi_core PUBLIC advapi32)
if(MSVC)
  target_compile_options(alpinoxi_core PRIVATE /W4 /permissive- /utf-8)
endif()

add_executable(alpinoxi_client WIN32 src/main.cpp src/ui.cpp)
target_link_libraries(alpinoxi_client PRIVATE alpinoxi_core ${ALPINOXI_LIBS})
if(MSVC)
  target_compile_options(alpinoxi_client PRIVATE /W4 /permissive- /utf-8)
endif()

# The icon and the assets travel beside the exe.
add_custom_command(TARGET alpinoxi_client POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
          "${CMAKE_CURRENT_SOURCE_DIR}/assets/icon.ico"
          "$<TARGET_FILE_DIR:alpinoxi_client>/assets/icon.ico"
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
          "${CMAKE_CURRENT_SOURCE_DIR}/assets/icon.png"
          "$<TARGET_FILE_DIR:alpinoxi_client>/assets/icon.png"
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
          "${CMAKE_CURRENT_SOURCE_DIR}/assets/tray.png"
          "$<TARGET_FILE_DIR:alpinoxi_client>/assets/tray.png"
  COMMENT "Copying assets beside the exe")

add_executable(alpinoxi_tests src/tests.cpp)
target_link_libraries(alpinoxi_tests PRIVATE alpinoxi_core)
if(MSVC)
  target_compile_options(alpinoxi_tests PRIVATE /W4 /permissive- /utf-8)
endif()

enable_testing()
add_test(NAME alpinoxi_tests COMMAND alpinoxi_tests)
