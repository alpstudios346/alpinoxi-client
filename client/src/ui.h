// The Win32 layer: a window, the seven screens, and the keys that move between
// them. It draws and it hit-tests; every decision it makes comes from app.cpp.

#pragma once

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include "app.h"

namespace alpinoxi {

enum class Screen { kHome, kInstances, kSkin, kFriends, kMods, kImport, kSettings };

const char* ScreenLabel(Screen screen);
const std::vector<Screen>& AllScreens();

// Where the rail and the cards sit, so the hit-test and the paint agree.
struct Layout {
  int rail_width = 236;
  int top_bar = 68;
  int status_bar = 30;
  int content_pad = 28;
};

class Window {
 public:
  Window();
  ~Window();

  bool Create(HINSTANCE instance, int show);
  // Returns the exit code once the message loop ends.
  int Run();
  void Close();

 private:
  static LRESULT CALLBACK WndProc(HWND hwnd, UINT message, WPARAM wparam, LPARAM lparam);
  LRESULT Handle(UINT message, WPARAM wparam, LPARAM lparam);

  void Paint(HDC dc);
  void OnKey(int vk, bool down);
  void OnMouseMove(int x, int y);
  void OnClick(int x, int y);
  void OnSize(int width, int height);

  // Hit-testing. Each returns true when the point is inside.
  Screen RailAt(int x, int y) const;
  int CardAt(int x, int y) const;  // instance card index, or -1
  int FriendAt(int x, int y) const;
  int WorldAt(int x, int y) const;
  int LoaderChipAt(int x, int y) const;
  int BindingRowAt(int x, int y) const;
  int HudRowAt(int x, int y) const;
  bool PointIn(int x, int y, const RECT& rect) const;

  void Reload();
  void Save();

  HWND hwnd_ = nullptr;
  HINSTANCE instance_ = nullptr;
  int width_ = 1180;
  int height_ = 760;
  bool running_ = true;

  Settings settings_;
  Account account_;
  FriendBook friends_;
  WorldBook worlds_;
  std::vector<Instance> instances_;
  Screen screen_ = Screen::kHome;

  // The HUD editor: which element is picked, and whether it is being dragged.
  std::string selected_element_;
  bool dragging_ = false;
  POINT drag_origin_{0, 0};

  // A key being rebound, if the settings screen is listening for one.
  int listening_row_ = -1;

  int hot_ = -1;
  std::string status_ = "Ready.";
  std::string app_dir_;
};

}  // namespace alpinoxi
