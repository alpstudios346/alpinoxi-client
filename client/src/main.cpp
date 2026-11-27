// The entry point. Everything a window needs is in ui.cpp; this file only
// starts it and reports what went wrong if it did not.

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <commctrl.h>

#include "ui.h"

#pragma comment(lib, "comctl32.lib")

int WINAPI wWinMain(HINSTANCE instance, HINSTANCE, PWSTR, int show) {
  // Without this, the window's controls draw as Windows 95 boxes and the
  // rounded cards come out squared off.
  INITCOMMONCONTROLSEX controls = {sizeof(controls), ICC_STANDARD_CLASSES};
  InitCommonControlsEx(&controls);
  SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

  alpinoxi::Window window;
  if (!window.Create(instance, show)) {
    MessageBoxW(nullptr, L"The window could not be created.", L"Alpinoxi Client",
                MB_ICONERROR | MB_OK);
    return 1;
  }
  return window.Run();
}
