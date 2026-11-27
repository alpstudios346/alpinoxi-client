// The launcher's entry point. Windows, keep it short.

#ifndef UNICODE
#define UNICODE
#endif
#ifndef _UNICODE
#define _UNICODE
#endif
#define WIN32_LEAN_AND_MEAN

#include <windows.h>
#include <shellapi.h>

#include "main.h"

int APIENTRY wWinMain(HINSTANCE instance, HINSTANCE, LPWSTR, int show) {
  // One launcher at a time: a second copy would fight over the same settings
  // file and the same game folder.
  HANDLE once = CreateMutexW(nullptr, TRUE, L"AlpinoxiClientSingleInstance");
  if (once != nullptr && GetLastError() == ERROR_ALREADY_EXISTS) {
    HWND existing = FindWindowW(L"AlpinoxiClientWindow", nullptr);
    if (existing) {
      ShowWindow(existing, SW_RESTORE);
      SetForegroundWindow(existing);
    }
    CloseHandle(once);
    return 0;
  }

  const int result = alpinoxi::Run(instance, show);
  if (once != nullptr) CloseHandle(once);
  return result;
}
