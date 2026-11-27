#include "ui.h"

#include <commctrl.h>
#include <shellapi.h>

// windows.h defines min and max as macros, which turns std::min(a, b) into
// nonsense. The launcher uses std::min and std::max, so the macros go.
#ifdef min
#undef min
#endif
#ifdef max
#undef max
#endif

#include <algorithm>
#include <cmath>
#include <cstdio>
#include <string>
#include <vector>

#pragma comment(lib, "comctl32.lib")

namespace alpinoxi {
namespace {

// ---------- the palette ----------
// Blue, not black: the window is a deep navy, the cards sit above it, and the
// accent is the only thing that is bright.

const COLORREF kBg = RGB(0x07, 0x0f, 0x20);
const COLORREF kPanel = RGB(0x0b, 0x16, 0x2c);
const COLORREF kCard = RGB(0x10, 0x1e, 0x39);
const COLORREF kCardHot = RGB(0x15, 0x27, 0x47);
const COLORREF kBorder = RGB(0x1e, 0x35, 0x59);
const COLORREF kAccent = RGB(0x3d, 0x9b, 0xff);
const COLORREF kAccentBright = RGB(0x6f, 0xc2, 0xff);
const COLORREF kAccentDeep = RGB(0x1b, 0x6c, 0xd6);
const COLORREF kText = RGB(0xea, 0xf3, 0xff);
const COLORREF kMuted = RGB(0x94, 0xae, 0xd2);
const COLORREF kDim = RGB(0x5b, 0x76, 0x9e);
const COLORREF kInk = RGB(0x04, 0x0f, 0x22);
const COLORREF kGreen = RGB(0x3d, 0xd6, 0x8c);
const COLORREF kAmber = RGB(0xf0, 0xb4, 0x3d);

// ---------- drawing helpers ----------

int Scale(HWND hwnd, int value) {
  const UINT dpi = GetDpiForWindow(hwnd);
  return MulDiv(value, static_cast<int>(dpi), 96);
}

void FillRect2(HDC dc, const RECT& rect, COLORREF color) {
  HBRUSH brush = CreateSolidBrush(color);
  FillRect(dc, &rect, brush);
  DeleteObject(brush);
}

// A rounded panel with a one-pixel border: what every card on every screen is.
void RoundRect2(HDC dc, const RECT& rect, COLORREF fill, COLORREF border, int radius) {
  HBRUSH brush = CreateSolidBrush(fill);
  HPEN pen = CreatePen(PS_SOLID, 1, border);
  const HGDIOBJ oldBrush = SelectObject(dc, brush);
  const HGDIOBJ oldPen = SelectObject(dc, pen);
  RoundRect(dc, rect.left, rect.top, rect.right, rect.bottom, radius, radius);
  SelectObject(dc, oldPen);
  SelectObject(dc, oldBrush);
  DeleteObject(pen);
  DeleteObject(brush);
}

// A vertical gradient, one line at a time. Only used for the accent surfaces,
// so the cost of a line per pixel does not matter.
void GradientRect(HDC dc, const RECT& rect, COLORREF top, COLORREF bottom) {
  const int height = rect.bottom - rect.top;
  if (height <= 0) return;
  for (int i = 0; i < height; ++i) {
    const double t = height == 1 ? 0.0 : static_cast<double>(i) / (height - 1);
    const COLORREF color = RGB(static_cast<int>(GetRValue(top) + (GetRValue(bottom) - GetRValue(top)) * t),
                               static_cast<int>(GetGValue(top) + (GetGValue(bottom) - GetGValue(top)) * t),
                               static_cast<int>(GetBValue(top) + (GetBValue(bottom) - GetBValue(top)) * t));
    RECT line = {rect.left, rect.top + i, rect.right, rect.top + i + 1};
    FillRect2(dc, line, color);
  }
}

void Text(HDC dc, const std::wstring& text, RECT rect, COLORREF color, int sizePx, bool bold,
          UINT format = DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX) {
  HFONT font = CreateFontW(-sizePx, 0, 0, 0, bold ? FW_SEMIBOLD : FW_NORMAL, FALSE, FALSE, FALSE,
                           DEFAULT_CHARSET, OUT_TT_PRECIS, CLIP_DEFAULT_PRECIS,
                           CLEARTYPE_QUALITY, DEFAULT_PITCH, L"Segoe UI");
  const HGDIOBJ oldFont = SelectObject(dc, font);
  SetTextColor(dc, color);
  SetBkMode(dc, TRANSPARENT);
  DrawTextW(dc, text.c_str(), -1, &rect, format);
  SelectObject(dc, oldFont);
  DeleteObject(font);
}

std::wstring Widen(const std::string& text) {
  if (text.empty()) return std::wstring();
  const int size = MultiByteToWideChar(CP_UTF8, 0, text.c_str(), -1, nullptr, 0);
  std::wstring wide(static_cast<size_t>(size > 0 ? size - 1 : 0), L'\0');
  if (size > 1) MultiByteToWideChar(CP_UTF8, 0, text.c_str(), -1, wide.data(), size);
  return wide;
}

std::string Narrow(const std::wstring& text) {
  if (text.empty()) return std::string();
  const int size = WideCharToMultiByte(CP_UTF8, 0, text.c_str(), -1, nullptr, 0, nullptr, nullptr);
  std::string narrow(static_cast<size_t>(size > 0 ? size - 1 : 0), '\0');
  if (size > 1) WideCharToMultiByte(CP_UTF8, 0, text.c_str(), -1, narrow.data(), size, nullptr, nullptr);
  return narrow;
}

RECT MakeRect(int left, int top, int right, int bottom) {
  RECT rect = {left, top, right, bottom};
  return rect;
}

std::string ReadFile(const std::string& path) {
  FILE* file = nullptr;
  if (fopen_s(&file, path.c_str(), "rb") != 0 || file == nullptr) return std::string();
  std::string text;
  char buffer[4096];
  size_t read = 0;
  while ((read = fread(buffer, 1, sizeof(buffer), file)) > 0) text.append(buffer, read);
  fclose(file);
  return text;
}

bool WriteFile(const std::string& path, const std::string& text) {
  FILE* file = nullptr;
  if (fopen_s(&file, path.c_str(), "wb") != 0 || file == nullptr) return false;
  const size_t written = fwrite(text.data(), 1, text.size(), file);
  fclose(file);
  return written == text.size();
}

std::string AppDataDir() {
  char buffer[MAX_PATH] = {0};
  DWORD size = GetEnvironmentVariableA("APPDATA", buffer, MAX_PATH);
  if (size == 0 || size >= MAX_PATH) return ".";
  std::string dir(buffer);
  dir += "\\alpinoxi";
  CreateDirectoryA(dir.c_str(), nullptr);
  return dir;
}

// The folder the exe sits in, so the assets travel with it.
std::string ExeDir() {
  char buffer[MAX_PATH] = {0};
  GetModuleFileNameA(nullptr, buffer, MAX_PATH);
  std::string path(buffer);
  const size_t slash = path.find_last_of("\\/");
  return slash == std::string::npos ? "." : path.substr(0, slash);
}

const char* PresenceWord(Presence presence) {
  switch (presence) {
    case Presence::kOffline:
      return "offline";
    case Presence::kLauncher:
      return "in the launcher";
    case Presence::kInGame:
      return "in game";
  }
  return "offline";
}

COLORREF PresenceColour(Presence presence) {
  switch (presence) {
    case Presence::kOffline:
      return kDim;
    case Presence::kLauncher:
      return kAmber;
    case Presence::kInGame:
      return kGreen;
  }
  return kDim;
}

}  // namespace

const char* ScreenLabel(Screen screen) {
  switch (screen) {
    case Screen::kHome:
      return "Home";
    case Screen::kInstances:
      return "Instances";
    case Screen::kSkin:
      return "Skin";
    case Screen::kFriends:
      return "Friends";
    case Screen::kMods:
      return "Mods";
    case Screen::kImport:
      return "Import";
    case Screen::kSettings:
      return "Settings";
  }
  return "Home";
}

const std::vector<Screen>& AllScreens() {
  static const std::vector<Screen> kScreens = {Screen::kHome,  Screen::kInstances, Screen::kSkin,
                                               Screen::kFriends, Screen::kMods, Screen::kImport,
                                               Screen::kSettings};
  return kScreens;
}

Window::Window() = default;
Window::~Window() = default;

bool Window::Create(HINSTANCE instance, int show) {
  instance_ = instance;
  app_dir_ = ExeDir();

  WNDCLASSEXW window_class = {0};
  window_class.cbSize = sizeof(window_class);
  window_class.style = CS_HREDRAW | CS_VREDRAW;
  window_class.lpfnWndProc = &Window::WndProc;
  window_class.hInstance = instance;
  window_class.hCursor = LoadCursor(nullptr, IDC_ARROW);
  window_class.hbrBackground = CreateSolidBrush(kBg);
  window_class.lpszClassName = L"AlpinoxiClientWindow";
  window_class.hIcon = static_cast<HICON>(
      LoadImageA(instance, (app_dir_ + "\\assets\\icon.ico").c_str(), IMAGE_ICON, 32, 32, LR_LOADFROMFILE));
  if (window_class.hIcon == nullptr) window_class.hIcon = LoadIcon(nullptr, IDI_APPLICATION);
  window_class.hIconSm = window_class.hIcon;
  if (!RegisterClassExW(&window_class)) return false;

  Reload();

  hwnd_ = CreateWindowExW(0, window_class.lpszClassName, L"Alpinoxi Client", WS_OVERLAPPEDWINDOW,
                          CW_USEDEFAULT, CW_USEDEFAULT, width_, height_, nullptr, nullptr, instance,
                          this);
  if (hwnd_ == nullptr) return false;

  RECT rect = {0, 0, width_, height_};
  AdjustWindowRectEx(&rect, WS_OVERLAPPEDWINDOW, FALSE, 0);
  SetWindowPos(hwnd_, nullptr, CW_USEDEFAULT, CW_USEDEFAULT, rect.right - rect.left,
               rect.bottom - rect.top, SWP_NOMOVE | SWP_NOZORDER);

  ShowWindow(hwnd_, show);
  UpdateWindow(hwnd_);
  return true;
}

int Window::Run() {
  MSG message = {0};
  while (running_ && GetMessageW(&message, nullptr, 0, 0) > 0) {
    TranslateMessage(&message);
    DispatchMessageW(&message);
  }
  return 0;
}

void Window::Close() {
  running_ = false;
  if (hwnd_ != nullptr) DestroyWindow(hwnd_);
}

LRESULT CALLBACK Window::WndProc(HWND hwnd, UINT message, WPARAM wparam, LPARAM lparam) {
  if (message == WM_NCCREATE) {
    auto* create = reinterpret_cast<CREATESTRUCTW*>(lparam);
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(create->lpCreateParams));
  }
  auto* self = reinterpret_cast<Window*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
  if (self == nullptr) return DefWindowProcW(hwnd, message, wparam, lparam);
  return self->Handle(message, wparam, lparam);
}

LRESULT Window::Handle(UINT message, WPARAM wparam, LPARAM lparam) {
  switch (message) {
    case WM_PAINT: {
      PAINTSTRUCT paint = {0};
      HDC dc = BeginPaint(hwnd_, &paint);
      // Double-buffered: a screen of cards redrawn straight to the window
      // flickers on every hover.
      HDC memory = CreateCompatibleDC(dc);
      HBITMAP bitmap = CreateCompatibleBitmap(dc, width_, height_);
      const HGDIOBJ oldBitmap = SelectObject(memory, bitmap);
      Paint(memory);
      BitBlt(dc, 0, 0, width_, height_, memory, 0, 0, SRCCOPY);
      SelectObject(memory, oldBitmap);
      DeleteObject(bitmap);
      DeleteDC(memory);
      EndPaint(hwnd_, &paint);
      return 0;
    }
    case WM_ERASEBKGND:
      return 1;
    case WM_SIZE:
      OnSize(LOWORD(lparam), HIWORD(lparam));
      return 0;
    case WM_GETMINMAXINFO: {
      auto* info = reinterpret_cast<MINMAXINFO*>(lparam);
      info->ptMinTrackSize.x = 940;
      info->ptMinTrackSize.y = 620;
      return 0;
    }
    case WM_MOUSEMOVE:
      OnMouseMove(LOWORD(lparam), HIWORD(lparam));
      return 0;
    case WM_LBUTTONDOWN:
      OnClick(LOWORD(lparam), HIWORD(lparam));
      return 0;
    case WM_LBUTTONUP:
      dragging_ = false;
      return 0;
    case WM_KEYDOWN:
      OnKey(static_cast<int>(wparam), true);
      return 0;
    case WM_KEYUP:
      OnKey(static_cast<int>(wparam), false);
      return 0;
    case WM_CLOSE:
      Close();
      return 0;
    case WM_DESTROY:
      PostQuitMessage(0);
      return 0;
    default:
      break;
  }
  return DefWindowProcW(hwnd_, message, wparam, lparam);
}

void Window::OnSize(int width, int height) {
  width_ = width < 1 ? 1 : width;
  height_ = height < 1 ? 1 : height;
  InvalidateRect(hwnd_, nullptr, FALSE);
}

void Window::Reload() {
  const std::string dir = AppDataDir();
  settings_ = SettingsFromJson(ReadFile(dir + "\\settings.json"));

  if (!settings_.player_name.empty()) {
    account_ = MakeOfflineAccount(settings_.player_name);
  }

  instances_.clear();
  for (Loader loader : AllLoaders()) {
    Instance instance;
    instance.target.loader = loader;
    GameVersion version;
    if (!GameVersion::Parse(settings_.version, &version)) version = GameVersion{1, 21, 1};
    instance.target.version = version;
    instance.id = InstanceId(loader, version);
    instance.name = instance.target.Label();
    instance.heap_mib = settings_.heap_mib;
    // An instance the player picked is the installed one; the rest are offered.
    instance.installed = loader == settings_.loader;
    instances_.push_back(instance);
  }

  // Nobody is on the friend list until a network answers, and nothing here
  // invents one.
  friends_.Replace({});
  worlds_.SetMaxPlayers(8);
}

void Window::Save() {
  settings_.provider = account_.provider;
  settings_.player_name = account_.name;
  WriteFile(AppDataDir() + "\\settings.json", SettingsToJson(settings_));
}

bool Window::PointIn(int x, int y, const RECT& rect) const {
  return x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom;
}

Screen Window::RailAt(int x, int y) const {
  if (x >= Layout().rail_width) return screen_;
  const int top = Scale(hwnd_, Layout().top_bar) + Scale(hwnd_, 18);
  const int row = Scale(hwnd_, 42);
  const int index = (y - top) / row;
  if (index < 0 || index >= static_cast<int>(AllScreens().size())) return screen_;
  return AllScreens()[static_cast<size_t>(index)];
}

int Window::CardAt(int x, int y) const {
  const int pad = Scale(hwnd_, Layout().content_pad);
  const int top = Scale(hwnd_, Layout().top_bar) + pad + Scale(hwnd_, 92);
  const int cardHeight = Scale(hwnd_, 84);
  const int gap = Scale(hwnd_, 12);
  const int index = (y - top) / (cardHeight + gap);
  if (y < top || index < 0 || index >= static_cast<int>(instances_.size())) return -1;
  if (!PointIn(x, y, MakeRect(pad + Scale(hwnd_, Layout().rail_width), top + index * (cardHeight + gap),
                              width_ - pad, top + index * (cardHeight + gap) + cardHeight))) {
    return -1;
  }
  return index;
}

int Window::FriendAt(int x, int y) const {
  const int pad = Scale(hwnd_, Layout().content_pad);
  const int left = pad + Scale(hwnd_, Layout().rail_width);
  const int top = Scale(hwnd_, Layout().top_bar) + pad + Scale(hwnd_, 92);
  const int row = Scale(hwnd_, 56);
  const int index = (y - top) / row;
  if (y < top || index < 0 || index >= static_cast<int>(friends_.All().size())) return -1;
  if (!PointIn(x, y, MakeRect(left, top, left + Scale(hwnd_, 520), top + row * static_cast<int>(friends_.All().size())))) {
    return -1;
  }
  return index;
}

int Window::WorldAt(int x, int y) const {
  const int pad = Scale(hwnd_, Layout().content_pad);
  const int left = pad + Scale(hwnd_, Layout().rail_width) + Scale(hwnd_, 552);
  const int top = Scale(hwnd_, Layout().top_bar) + pad + Scale(hwnd_, 92);
  const int row = Scale(hwnd_, 76);
  const int index = (y - top) / row;
  if (y < top || index < 0 || index >= static_cast<int>(worlds_.All().size())) return -1;
  if (!PointIn(x, y, MakeRect(left, top, width_ - pad, top + row * static_cast<int>(worlds_.All().size())))) {
    return -1;
  }
  return index;
}

int Window::LoaderChipAt(int x, int y) const {
  const int pad = Scale(hwnd_, Layout().content_pad);
  const int left = pad + Scale(hwnd_, Layout().rail_width);
  const int top = Scale(hwnd_, Layout().top_bar) + pad + Scale(hwnd_, 48);
  const std::vector<Loader> loaders = LoadersFor(settings_.version);
  const int chipWidth = Scale(hwnd_, 96);
  const int gap = Scale(hwnd_, 8);
  for (size_t i = 0; i < loaders.size(); ++i) {
    if (PointIn(x, y, MakeRect(left + static_cast<int>(i) * (chipWidth + gap), top,
                               left + static_cast<int>(i) * (chipWidth + gap) + chipWidth,
                               top + Scale(hwnd_, 32)))) {
      return static_cast<int>(i);
    }
  }
  return -1;
}

int Window::BindingRowAt(int x, int y) const {
  const int pad = Scale(hwnd_, Layout().content_pad);
  const int left = pad + Scale(hwnd_, Layout().rail_width);
  const int top = Scale(hwnd_, Layout().top_bar) + pad + Scale(hwnd_, 96);
  const int row = Scale(hwnd_, 40);
  const int index = (y - top) / row;
  if (y < top || index < 0 || index >= static_cast<int>(settings_.bindings.size())) return -1;
  if (!PointIn(x, y, MakeRect(left, top, width_ - pad, top + row * static_cast<int>(settings_.bindings.size())))) {
    return -1;
  }
  return index;
}

int Window::HudRowAt(int x, int y) const {
  const int pad = Scale(hwnd_, Layout().content_pad);
  const int left = pad + Scale(hwnd_, Layout().rail_width) + Scale(hwnd_, 460);
  const int top = Scale(hwnd_, Layout().top_bar) + pad + Scale(hwnd_, 96);
  const int row = Scale(hwnd_, 44);
  const int index = (y - top) / row;
  if (y < top || index < 0 || index >= static_cast<int>(settings_.hud.size())) return -1;
  if (!PointIn(x, y, MakeRect(left, top, width_ - pad, top + row * static_cast<int>(settings_.hud.size())))) {
    return -1;
  }
  return index;
}

void Window::OnMouseMove(int x, int y) {
  int hot = -1;
  switch (screen_) {
    case Screen::kInstances:
      hot = CardAt(x, y);
      break;
    case Screen::kFriends:
      // The two lists are painted on different cards, so one index is enough:
      // a point is in at most one of them.
      hot = FriendAt(x, y);
      if (hot < 0) hot = WorldAt(x, y);
      break;
    case Screen::kSettings:
      // The same: one index for both lists, which never overlap.
      hot = BindingRowAt(x, y);
      if (hot < 0) hot = HudRowAt(x, y);
      break;
    default:
      break;
  }

  if (dragging_ && !selected_element_.empty()) {
    // Dragging moves the element the player picked, in pixels from its anchor.
    for (HudElement& element : settings_.hud) {
      if (element.id != selected_element_) continue;
      element.offset_x += x - drag_origin_.x;
      element.offset_y += y - drag_origin_.y;
      // Kept on screen: an element dragged off the edge is an element lost.
      element.offset_x = std::max(-width_ / 2, std::min(width_ / 2, element.offset_x));
      element.offset_y = std::max(-height_ / 2, std::min(height_ / 2, element.offset_y));
    }
    drag_origin_.x = x;
    drag_origin_.y = y;
    InvalidateRect(hwnd_, nullptr, FALSE);
    return;
  }

  if (hot != hot_) {
    hot_ = hot;
    InvalidateRect(hwnd_, nullptr, FALSE);
  }
}

void Window::OnClick(int x, int y) {
  // The rail is always live, on every screen.
  if (x < Scale(hwnd_, Layout().rail_width) && y > Scale(hwnd_, Layout().top_bar)) {
    const Screen picked = RailAt(x, y);
    if (picked != screen_) {
      screen_ = picked;
      selected_element_.clear();
      listening_row_ = -1;
      status_ = std::string(ScreenLabel(screen_));
      InvalidateRect(hwnd_, nullptr, FALSE);
    }
    return;
  }

  const int pad = Scale(hwnd_, Layout().content_pad);
  const int left = pad + Scale(hwnd_, Layout().rail_width);
  const int top = Scale(hwnd_, Layout().top_bar);

  switch (screen_) {
    case Screen::kInstances: {
      const int chip = LoaderChipAt(x, y);
      if (chip >= 0) {
        const std::vector<Loader> loaders = LoadersFor(settings_.version);
        if (chip < static_cast<int>(loaders.size())) {
          settings_.loader = loaders[static_cast<size_t>(chip)];
          for (Instance& instance : instances_) instance.installed = instance.target.loader == settings_.loader;
          Save();
          status_ = std::string(LoaderLabel(settings_.loader)) + " selected.";
        }
        InvalidateRect(hwnd_, nullptr, FALSE);
        return;
      }
      const int card = CardAt(x, y);
      if (card >= 0) {
        settings_.loader = instances_[static_cast<size_t>(card)].target.loader;
        for (Instance& instance : instances_) instance.installed = instance.target.loader == settings_.loader;
        Save();
        status_ = instances_[static_cast<size_t>(card)].name + " selected.";
        InvalidateRect(hwnd_, nullptr, FALSE);
        return;
      }
      break;
    }
    case Screen::kFriends: {
      const int friend_index = FriendAt(x, y);
      if (friend_index >= 0) {
        const Friend& entry = friends_.All()[static_cast<size_t>(friend_index)];
        if (entry.Joinable()) {
          const JoinResult result = worlds_.MayJoin(entry.world_id, account_.Key(), true, "");
          status_ = JoinMessage(result);
          if (result == JoinResult::kOk) worlds_.Enter(entry.world_id);
        } else {
          status_ = entry.name + " is " + PresenceWord(entry.presence) + ".";
        }
        InvalidateRect(hwnd_, nullptr, FALSE);
        return;
      }
      const int world_index = WorldAt(x, y);
      if (world_index >= 0) {
        const World& world = worlds_.All()[static_cast<size_t>(world_index)];
        const JoinResult result = worlds_.MayJoin(world.id, account_.Key(), false, world.Code());
        status_ = JoinMessage(result);
        if (result == JoinResult::kOk) worlds_.Enter(world.id);
        InvalidateRect(hwnd_, nullptr, FALSE);
        return;
      }
      break;
    }
    case Screen::kSettings: {
      const int row = BindingRowAt(x, y);
      if (row >= 0) {
        // Listening for the next key, which then becomes the binding.
        listening_row_ = row;
        status_ = "Press a key for \"" + std::string(ActionLabel(settings_.bindings[static_cast<size_t>(row)].action)) + "\".";
        InvalidateRect(hwnd_, nullptr, FALSE);
        return;
      }
      const int hud_row = HudRowAt(x, y);
      if (hud_row >= 0) {
        const std::string id = settings_.hud[static_cast<size_t>(hud_row)].id;
        const int visible = ToggleElement(&settings_.hud, id);
        status_ = settings_.hud[static_cast<size_t>(hud_row)].label + (visible == 1 ? " shown." : " hidden.");
        Save();
        InvalidateRect(hwnd_, nullptr, FALSE);
        return;
      }
      break;
    }
    default:
      break;
  }

  // The HUD editor: clicking an element picks it up, and the next move drags it.
  if (screen_ == Screen::kHome) {
    for (const HudElement& element : settings_.hud) {
      if (!element.visible) continue;
      const int element_width = Scale(hwnd_, 120);
      const int element_height = Scale(hwnd_, 20);
      int element_left = left + Scale(hwnd_, 20) + Scale(hwnd_, element.offset_x);
      int element_top = top + Scale(hwnd_, 20) + Scale(hwnd_, element.offset_y);
      switch (element.anchor) {
        case Anchor::kTopCenter:
          element_left = (width_ - element_width) / 2 + Scale(hwnd_, element.offset_x);
          break;
        case Anchor::kTopRight:
          element_left = width_ - pad - element_width + Scale(hwnd_, element.offset_x);
          break;
        case Anchor::kBottomLeft:
          element_top = height_ - Scale(hwnd_, Layout().status_bar) - Scale(hwnd_, 24) + Scale(hwnd_, element.offset_y);
          break;
        case Anchor::kBottomCenter:
          element_left = (width_ - element_width) / 2 + Scale(hwnd_, element.offset_x);
          element_top = height_ - Scale(hwnd_, Layout().status_bar) - Scale(hwnd_, 24) + Scale(hwnd_, element.offset_y);
          break;
        case Anchor::kBottomRight:
          element_left = width_ - pad - element_width + Scale(hwnd_, element.offset_x);
          element_top = height_ - Scale(hwnd_, Layout().status_bar) - Scale(hwnd_, 24) + Scale(hwnd_, element.offset_y);
          break;
        case Anchor::kTopLeft:
          break;
      }
      if (PointIn(x, y, MakeRect(element_left, element_top, element_left + element_width,
                                 element_top + element_height))) {
        selected_element_ = element.id;
        dragging_ = true;
        drag_origin_.x = x;
        drag_origin_.y = y;
        status_ = element.label + " picked up. Drag it, then press Esc to put it down.";
        InvalidateRect(hwnd_, nullptr, FALSE);
        return;
      }
    }
  }
}

void Window::OnKey(int vk, bool down) {
  if (!down) return;

  // A binding being recorded takes the key before anything else sees it.
  if (listening_row_ >= 0 && listening_row_ < static_cast<int>(settings_.bindings.size())) {
    if (vk == VK_SHIFT || vk == VK_CONTROL || vk == VK_MENU) return;
    Binding& binding = settings_.bindings[static_cast<size_t>(listening_row_)];
    const KeyCombo combo{vk, (GetKeyState(VK_SHIFT) & 0x8000) != 0,
                         (GetKeyState(VK_CONTROL) & 0x8000) != 0,
                         (GetKeyState(VK_MENU) & 0x8000) != 0};
    Action holder;
    if (BindingConflict(settings_.bindings, combo, binding.action, &holder)) {
      status_ = std::string(combo.ToString()) + " is already \"" + ActionLabel(holder) + "\".";
    } else {
      binding.combo = combo;
      status_ = std::string(ActionLabel(binding.action)) + " is now " + combo.ToString() + ".";
      Save();
    }
    listening_row_ = -1;
    InvalidateRect(hwnd_, nullptr, FALSE);
    return;
  }

  const KeyCombo combo{vk, (GetKeyState(VK_SHIFT) & 0x8000) != 0,
                       (GetKeyState(VK_CONTROL) & 0x8000) != 0,
                       (GetKeyState(VK_MENU) & 0x8000) != 0};
  Action action;
  if (!ResolveBinding(settings_.bindings, combo, &action)) return;

  switch (action) {
    case Action::kOpenFriends:
      // Shift+Tab: the friends and the worlds that are open right now.
      screen_ = Screen::kFriends;
      status_ = "Friends and worlds";
      break;
    case Action::kOpenHudEditor:
      screen_ = Screen::kHome;
      status_ = "HUD editor: drag an element, Esc to put it down.";
      break;
    case Action::kCloseHud:
      // Esc puts down whatever is being dragged, and closes the HUD when
      // nothing is. That way a player can always get back out of the editor.
      if (dragging_) {
        dragging_ = false;
        selected_element_.clear();
        Save();
        status_ = "HUD saved.";
      } else if (settings_.hud_enabled) {
        settings_.hud_enabled = false;
        Save();
        status_ = "HUD closed. Ctrl+H brings it back.";
      } else {
        screen_ = Screen::kHome;
        status_ = "Ready.";
      }
      break;
    case Action::kToggleHud:
      settings_.hud_enabled = !settings_.hud_enabled;
      Save();
      status_ = settings_.hud_enabled ? "HUD shown." : "HUD hidden.";
      break;
    case Action::kOpenChat:
      screen_ = Screen::kFriends;
      status_ = "Chat is in the HUD.";
      break;
    case Action::kOpenSkin:
      screen_ = Screen::kSkin;
      status_ = "Skin";
      break;
    case Action::kOpenSettings:
      screen_ = Screen::kSettings;
      status_ = "Settings";
      break;
  }
  InvalidateRect(hwnd_, nullptr, FALSE);
}

void Window::Paint(HDC dc) {
  const Layout layout;
  const int dpiRail = Scale(hwnd_, layout.rail_width);
  const int dpiTop = Scale(hwnd_, layout.top_bar);
  const int dpiStatus = Scale(hwnd_, layout.status_bar);
  const int pad = Scale(hwnd_, layout.content_pad);
  (void)pad;

  // The window itself, with a faint blue light behind the top-left corner.
  FillRect2(dc, MakeRect(0, 0, width_, height_), kBg);

  // ---------- the rail ----------
  FillRect2(dc, MakeRect(0, 0, dpiRail, height_), kPanel);
  FillRect2(dc, MakeRect(dpiRail - 1, 0, dpiRail, height_), kBorder);

  // The wordmark: the logo's own name, drawn rather than loaded, so a missing
  // asset never leaves the window blank.
  {
    RECT logoRect = MakeRect(Scale(hwnd_, 22), Scale(hwnd_, 20), dpiRail - Scale(hwnd_, 12), Scale(hwnd_, 52));
    Text(dc, L"ALPINOXI", logoRect, kText, Scale(hwnd_, 22), true);
    RECT subRect = MakeRect(Scale(hwnd_, 22), Scale(hwnd_, 46), dpiRail - Scale(hwnd_, 12), Scale(hwnd_, 64));
    Text(dc, L"client", subRect, kAccent, Scale(hwnd_, 12), false);
  }

  const int rowTop = dpiTop + Scale(hwnd_, 18);
  const int rowHeight = Scale(hwnd_, 42);
  for (size_t i = 0; i < AllScreens().size(); ++i) {
    const int top = rowTop + static_cast<int>(i) * rowHeight;
    const bool active = AllScreens()[i] == screen_;
    RECT row = MakeRect(Scale(hwnd_, 12), top, dpiRail - Scale(hwnd_, 12), top + rowHeight - Scale(hwnd_, 6));
    if (active) {
      RoundRect2(dc, row, kCard, kBorder, Scale(hwnd_, 10));
      FillRect2(dc, MakeRect(row.left, row.top + Scale(hwnd_, 9), row.left + Scale(hwnd_, 3),
                             row.bottom - Scale(hwnd_, 9)),
                kAccent);
    }
    RECT label = row;
    label.left += Scale(hwnd_, 18);
    Text(dc, Widen(ScreenLabel(AllScreens()[i])), label, active ? kText : kMuted,
         Scale(hwnd_, 14), active);
  }

  // The account, at the foot of the rail.
  {
    const int top = height_ - Scale(hwnd_, 92);
    RECT card = MakeRect(Scale(hwnd_, 12), top, dpiRail - Scale(hwnd_, 12), height_ - Scale(hwnd_, 16));
    RoundRect2(dc, card, kCard, kBorder, Scale(hwnd_, 12));
    const std::string name = account_.name.empty() ? "Not signed in" : account_.name;
    RECT nameRect = card;
    nameRect.left += Scale(hwnd_, 14);
    nameRect.bottom -= Scale(hwnd_, 22);
    Text(dc, Widen(name), nameRect, kText, Scale(hwnd_, 13), true);
    RECT providerRect = card;
    providerRect.left += Scale(hwnd_, 14);
    providerRect.top += Scale(hwnd_, 22);
    Text(dc, Widen(ProviderLabel(account_.provider)), providerRect, kAccentBright,
         Scale(hwnd_, 11), false);
  }

  // ---------- the top bar ----------
  FillRect2(dc, MakeRect(dpiRail, 0, width_, dpiTop), kBg);
  FillRect2(dc, MakeRect(dpiRail, dpiTop - 1, width_, dpiTop), kBorder);

  {
    RECT title = MakeRect(dpiRail + pad, 0, width_ - Scale(hwnd_, 420), dpiTop);
    Text(dc, Widen(ScreenLabel(screen_)), title, kText, Scale(hwnd_, 20), true);
  }

  // The launch button: the one gradient on the screen.
  {
    const int buttonWidth = Scale(hwnd_, 168);
    const int buttonHeight = Scale(hwnd_, 38);
    const int left = width_ - pad - buttonWidth;
    const int top = (dpiTop - buttonHeight) / 2;
    RECT button = MakeRect(left, top, left + buttonWidth, top + buttonHeight);
    GradientRect(dc, button, kAccentBright, kAccentDeep);
    RECT label = button;
    Text(dc, L"Launch", label, kInk, Scale(hwnd_, 15), true, DT_CENTER | DT_SINGLELINE | DT_VCENTER);
  }
  {
    const int buttonWidth = Scale(hwnd_, 110);
    const int buttonHeight = Scale(hwnd_, 38);
    const int left = width_ - pad - Scale(hwnd_, 168) - Scale(hwnd_, 10) - buttonWidth;
    const int top = (dpiTop - buttonHeight) / 2;
    RECT button = MakeRect(left, top, left + buttonWidth, top + buttonHeight);
    RoundRect2(dc, button, kPanel, kBorder, Scale(hwnd_, 8));
    Text(dc, L"Settings", button, kMuted, Scale(hwnd_, 13), false,
         DT_CENTER | DT_SINGLELINE | DT_VCENTER);
  }

  const int contentLeft = dpiRail + pad;
  const int contentTop = dpiTop + pad;
  const int contentRight = width_ - pad;
  const int contentBottom = height_ - dpiStatus - pad;

  switch (screen_) {
    case Screen::kHome: {
      // A card for the chosen instance, then the HUD's own elements, drawn
      // where the editor put them.
      RECT hero = MakeRect(contentLeft, contentTop, contentRight, contentTop + Scale(hwnd_, 120));
      RoundRect2(dc, hero, kCard, kBorder, Scale(hwnd_, 14));
      RECT heroTitle = hero;
      heroTitle.left += Scale(hwnd_, 20);
      heroTitle.top += Scale(hwnd_, 18);
      heroTitle.bottom = heroTitle.top + Scale(hwnd_, 26);
      Text(dc, Widen(LoaderTarget{settings_.loader, GameVersion{1, 21, 1}}.Label()), heroTitle, kText,
           Scale(hwnd_, 22), true);
      RECT heroSub = hero;
      heroSub.left += Scale(hwnd_, 20);
      heroSub.top += Scale(hwnd_, 52);
      heroSub.bottom = heroSub.top + Scale(hwnd_, 22);
      Text(dc, L"Shift+Tab for friends and worlds  ·  Esc closes the HUD  ·  Shift+H edits it",
           heroSub, kMuted, Scale(hwnd_, 12), false);
      RECT heroHeap = hero;
      heroHeap.left += Scale(hwnd_, 20);
      heroHeap.top += Scale(hwnd_, 80);
      heroHeap.bottom = heroHeap.top + Scale(hwnd_, 20);
      Text(dc, Widen(std::to_string(settings_.heap_mib) + " MB heap"), heroHeap, kAccentBright,
           Scale(hwnd_, 12), false);

      if (settings_.hud_enabled) {
        for (const HudElement& element : settings_.hud) {
          if (!element.visible) continue;
          const int elementWidth = Scale(hwnd_, 120);
          const int elementHeight = Scale(hwnd_, 20);
          int chipLeft = contentLeft + Scale(hwnd_, 20) + Scale(hwnd_, element.offset_x);
          int chipTop = contentTop + Scale(hwnd_, 140) + Scale(hwnd_, element.offset_y);
          switch (element.anchor) {
            case Anchor::kTopCenter:
              chipLeft = (width_ - elementWidth) / 2 + Scale(hwnd_, element.offset_x);
              break;
            case Anchor::kTopRight:
              chipLeft = contentRight - elementWidth + Scale(hwnd_, element.offset_x);
              break;
            case Anchor::kBottomLeft:
              chipTop = contentBottom - Scale(hwnd_, 40) + Scale(hwnd_, element.offset_y);
              break;
            case Anchor::kBottomCenter:
              chipLeft = (width_ - elementWidth) / 2 + Scale(hwnd_, element.offset_x);
              chipTop = contentBottom - Scale(hwnd_, 40) + Scale(hwnd_, element.offset_y);
              break;
            case Anchor::kBottomRight:
              chipLeft = contentRight - elementWidth + Scale(hwnd_, element.offset_x);
              chipTop = contentBottom - Scale(hwnd_, 40) + Scale(hwnd_, element.offset_y);
              break;
            case Anchor::kTopLeft:
              break;
          }
          RECT chip = MakeRect(chipLeft, chipTop, chipLeft + elementWidth, chipTop + elementHeight);
          RoundRect2(dc, chip, kCard, element.id == selected_element_ ? kAccent : kBorder,
                     Scale(hwnd_, 6));
          Text(dc, Widen(element.label), chip, kText, Scale(hwnd_, 11), false,
               DT_CENTER | DT_SINGLELINE | DT_VCENTER);
        }
      }
      break;
    }
    case Screen::kInstances: {
      // The loader chips first: what a version can run with.
      RECT heading = MakeRect(contentLeft, contentTop, contentRight, contentTop + Scale(hwnd_, 34));
      Text(dc, Widen("Minecraft " + settings_.version), heading, kMuted, Scale(hwnd_, 13), false);

      const std::vector<Loader> loaders = LoadersFor(settings_.version);
      const int chipWidth = Scale(hwnd_, 96);
      const int chipTop = contentTop + Scale(hwnd_, 48);
      for (size_t i = 0; i < loaders.size(); ++i) {
        RECT chip = MakeRect(contentLeft + static_cast<int>(i) * (chipWidth + Scale(hwnd_, 8)), chipTop,
                             contentLeft + static_cast<int>(i) * (chipWidth + Scale(hwnd_, 8)) + chipWidth,
                             chipTop + Scale(hwnd_, 32));
        const bool chosen = loaders[i] == settings_.loader;
        RoundRect2(dc, chip, chosen ? kCardHot : kPanel, chosen ? kAccent : kBorder, Scale(hwnd_, 8));
        Text(dc, Widen(LoaderLabel(loaders[i])), chip, chosen ? kAccentBright : kMuted,
             Scale(hwnd_, 12), chosen, DT_CENTER | DT_SINGLELINE | DT_VCENTER);
      }

      const int cardTop = contentTop + Scale(hwnd_, 92);
      const int cardHeight = Scale(hwnd_, 84);
      for (size_t i = 0; i < instances_.size(); ++i) {
        const int top = cardTop + static_cast<int>(i) * (cardHeight + Scale(hwnd_, 12));
        RECT card = MakeRect(contentLeft, top, contentRight, top + cardHeight);
        const bool chosen = instances_[i].target.loader == settings_.loader;
        RoundRect2(dc, card, hot_ == static_cast<int>(i) ? kCardHot : kCard,
                   chosen ? kAccent : kBorder, Scale(hwnd_, 12));

        RECT name = card;
        name.left += Scale(hwnd_, 18);
        name.right -= Scale(hwnd_, 220);
        name.top += Scale(hwnd_, 12);
        name.bottom = name.top + Scale(hwnd_, 24);
        Text(dc, Widen(instances_[i].name), name, kText, Scale(hwnd_, 16), true);

        RECT detail = card;
        detail.left += Scale(hwnd_, 18);
        detail.right -= Scale(hwnd_, 220);
        detail.top += Scale(hwnd_, 42);
        detail.bottom = detail.top + Scale(hwnd_, 22);
        Text(dc, Widen(std::string(LoaderMainClass(instances_[i].target.loader)).substr(0, 44) + "..."),
             detail, kDim, Scale(hwnd_, 11), false);

        RECT heap = card;
        heap.left = card.right - Scale(hwnd_, 200);
        heap.right -= Scale(hwnd_, 18);
        heap.top += Scale(hwnd_, 12);
        heap.bottom = heap.top + Scale(hwnd_, 22);
        Text(dc, Widen(std::to_string(instances_[i].heap_mib) + " MB"), heap, kMuted, Scale(hwnd_, 12),
             false, DT_RIGHT | DT_SINGLELINE | DT_VCENTER);

        if (chosen) {
          RECT badge = MakeRect(card.right - Scale(hwnd_, 92), card.bottom - Scale(hwnd_, 32),
                                card.right - Scale(hwnd_, 18), card.bottom - Scale(hwnd_, 10));
          RoundRect2(dc, badge, kAccentDeep, kAccent, Scale(hwnd_, 8));
          Text(dc, L"selected", badge, kText, Scale(hwnd_, 11), false,
               DT_CENTER | DT_SINGLELINE | DT_VCENTER);
        }
      }
      break;
    }
    case Screen::kSkin: {
      RECT card = MakeRect(contentLeft, contentTop, contentLeft + Scale(hwnd_, 320),
                           contentTop + Scale(hwnd_, 380));
      RoundRect2(dc, card, kCard, kBorder, Scale(hwnd_, 14));
      RECT title = card;
      title.left += Scale(hwnd_, 18);
      title.top += Scale(hwnd_, 16);
      title.bottom = title.top + Scale(hwnd_, 24);
      Text(dc, L"Skin", title, kText, Scale(hwnd_, 16), true);
      RECT body = card;
      body.left += Scale(hwnd_, 18);
      body.right -= Scale(hwnd_, 18);
      body.top += Scale(hwnd_, 56);
      Text(dc, L"A skin is drawn from the account's own profile, so there is "
               L"nothing to pick here until one is signed in.",
           body, kMuted, Scale(hwnd_, 12), false,
           DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
      break;
    }
    case Screen::kFriends: {
      // Shift+Tab lands here: the friend list on the left, the open worlds on
      // the right, and the host button between them.
      RECT leftCard = MakeRect(contentLeft, contentTop, contentLeft + Scale(hwnd_, 520), contentBottom);
      RoundRect2(dc, leftCard, kCard, kBorder, Scale(hwnd_, 14));
      RECT leftTitle = leftCard;
      leftTitle.left += Scale(hwnd_, 18);
      leftTitle.top += Scale(hwnd_, 14);
      leftTitle.bottom = leftTitle.top + Scale(hwnd_, 24);
      Text(dc, L"Friends", leftTitle, kText, Scale(hwnd_, 15), true);
      RECT leftCount = leftTitle;
      leftCount.right -= Scale(hwnd_, 18);
      Text(dc, Widen(friends_.OnlineLabel()), leftCount, kAccentBright, Scale(hwnd_, 12), false,
           DT_RIGHT | DT_SINGLELINE | DT_VCENTER);

      const int listTop = contentTop + Scale(hwnd_, 92);
      const int rowHeight = Scale(hwnd_, 56);
      if (friends_.All().empty()) {
        RECT empty = MakeRect(leftCard.left + Scale(hwnd_, 18), listTop, leftCard.right - Scale(hwnd_, 18),
                              listTop + Scale(hwnd_, 60));
        Text(dc, L"Nobody on the list yet. Friends show up here the moment the "
                 L"launcher hears from them.",
             empty, kDim, Scale(hwnd_, 12), false, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
      }
      for (size_t i = 0; i < friends_.All().size(); ++i) {
        const Friend& entry = friends_.All()[i];
        const int top = listTop + static_cast<int>(i) * rowHeight;
        RECT row = MakeRect(leftCard.left + Scale(hwnd_, 10), top, leftCard.right - Scale(hwnd_, 10),
                            top + rowHeight - Scale(hwnd_, 6));
        if (hot_ == static_cast<int>(i)) RoundRect2(dc, row, kCardHot, kBorder, Scale(hwnd_, 8));
        RECT dot = MakeRect(row.left + Scale(hwnd_, 10), top + Scale(hwnd_, 14),
                            row.left + Scale(hwnd_, 20), top + Scale(hwnd_, 24));
        HBRUSH brush = CreateSolidBrush(PresenceColour(entry.presence));
        HPEN pen = CreatePen(PS_NULL, 0, PresenceColour(entry.presence));
        const HGDIOBJ oldBrush = SelectObject(dc, brush);
        const HGDIOBJ oldPen = SelectObject(dc, pen);
        Ellipse(dc, dot.left, dot.top, dot.right, dot.bottom);
        SelectObject(dc, oldPen);
        SelectObject(dc, oldBrush);
        DeleteObject(pen);
        DeleteObject(brush);

        RECT nameRect = row;
        nameRect.left += Scale(hwnd_, 32);
        nameRect.bottom -= Scale(hwnd_, 18);
        Text(dc, Widen(entry.name), nameRect, kText, Scale(hwnd_, 13), true);
        RECT presenceRect = row;
        presenceRect.left += Scale(hwnd_, 32);
        presenceRect.top += Scale(hwnd_, 18);
        Text(dc, Widen(PresenceWord(entry.presence)), presenceRect, kDim, Scale(hwnd_, 11), false);
        if (entry.Joinable()) {
          RECT join = MakeRect(row.right - Scale(hwnd_, 80), row.top + Scale(hwnd_, 8),
                               row.right - Scale(hwnd_, 10), row.bottom - Scale(hwnd_, 8));
          RoundRect2(dc, join, kAccentDeep, kAccent, Scale(hwnd_, 8));
          Text(dc, L"Join", join, kText, Scale(hwnd_, 11), false,
               DT_CENTER | DT_SINGLELINE | DT_VCENTER);
        }
      }

      const int rightLeft = leftCard.right + Scale(hwnd_, 20);
      RECT rightCard = MakeRect(rightLeft, contentTop, contentRight, contentBottom);
      RoundRect2(dc, rightCard, kCard, kBorder, Scale(hwnd_, 14));
      RECT rightTitle = rightCard;
      rightTitle.left += Scale(hwnd_, 18);
      rightTitle.top += Scale(hwnd_, 14);
      rightTitle.bottom = rightTitle.top + Scale(hwnd_, 24);
      Text(dc, L"Worlds open now", rightTitle, kText, Scale(hwnd_, 15), true);

      RECT hostButton = MakeRect(rightCard.right - Scale(hwnd_, 130), rightCard.top + Scale(hwnd_, 12),
                                 rightCard.right - Scale(hwnd_, 18), rightCard.top + Scale(hwnd_, 42));
      GradientRect(dc, hostButton, kAccentBright, kAccentDeep);
      Text(dc, L"Host a world", hostButton, kInk, Scale(hwnd_, 12), true,
           DT_CENTER | DT_SINGLELINE | DT_VCENTER);

      const int worldTop = contentTop + Scale(hwnd_, 92);
      if (worlds_.All().empty()) {
        RECT empty = MakeRect(rightCard.left + Scale(hwnd_, 18), worldTop,
                              rightCard.right - Scale(hwnd_, 18), worldTop + Scale(hwnd_, 60));
        Text(dc, L"No world is open. Host one and share its code, or join a "
                 L"friend's from the list.",
             empty, kDim, Scale(hwnd_, 12), false, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
      }
      for (size_t i = 0; i < worlds_.All().size(); ++i) {
        const World& world = worlds_.All()[i];
        const int top = worldTop + static_cast<int>(i) * Scale(hwnd_, 76);
        RECT row = MakeRect(rightCard.left + Scale(hwnd_, 10), top, rightCard.right - Scale(hwnd_, 10),
                            top + Scale(hwnd_, 66));
        RoundRect2(dc, row, hot_ == static_cast<int>(i) ? kCardHot : kPanel, kBorder,
                   Scale(hwnd_, 10));
        RECT nameRect = row;
        nameRect.left += Scale(hwnd_, 14);
        nameRect.bottom -= Scale(hwnd_, 26);
        Text(dc, Widen(world.name), nameRect, kText, Scale(hwnd_, 13), true);
        RECT codeRect = row;
        codeRect.left += Scale(hwnd_, 14);
        codeRect.top += Scale(hwnd_, 26);
        Text(dc, Widen(world.Code() + "   " + std::to_string(world.players) + "/" +
                       std::to_string(world.max_players)),
             codeRect, kAccentBright, Scale(hwnd_, 11), false);
      }
      break;
    }
    case Screen::kMods: {
      RECT card = MakeRect(contentLeft, contentTop, contentRight, contentTop + Scale(hwnd_, 260));
      RoundRect2(dc, card, kCard, kBorder, Scale(hwnd_, 14));
      RECT title = card;
      title.left += Scale(hwnd_, 18);
      title.top += Scale(hwnd_, 16);
      title.bottom = title.top + Scale(hwnd_, 24);
      Text(dc, L"Mods", title, kText, Scale(hwnd_, 16), true);
      RECT body = card;
      body.left += Scale(hwnd_, 18);
      body.right -= Scale(hwnd_, 18);
      body.top += Scale(hwnd_, 56);
      Text(dc, L"Sodium, Lithium, FerriteCore, Krypton, ModernFix, EntityCulling, "
               L"Indium, ImmediatelyFast and MemoryLeakFix, all installed for "
               L"Fabric 1.21.1. The three Alpinoxi mods ride along with them: the "
               L"bridge, the HUD and the performance rules.",
           body, kMuted, Scale(hwnd_, 12), false, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
      break;
    }
    case Screen::kImport: {
      RECT card = MakeRect(contentLeft, contentTop, contentRight, contentTop + Scale(hwnd_, 240));
      RoundRect2(dc, card, kCard, kBorder, Scale(hwnd_, 14));
      RECT title = card;
      title.left += Scale(hwnd_, 18);
      title.top += Scale(hwnd_, 16);
      title.bottom = title.top + Scale(hwnd_, 24);
      Text(dc, L"Import", title, kText, Scale(hwnd_, 16), true);
      RECT body = card;
      body.left += Scale(hwnd_, 18);
      body.right -= Scale(hwnd_, 18);
      body.top += Scale(hwnd_, 56);
      Text(dc, L"Worlds, resource packs, shaders and mods come across from CurseForge, "
               L"Modrinth, MultiMC, Prism, ATLauncher and the vanilla folder. "
               L"Accounts never do: those are signed in here or not at all.",
           body, kMuted, Scale(hwnd_, 12), false, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
      break;
    }
    case Screen::kSettings: {
      RECT bindingsCard = MakeRect(contentLeft, contentTop, contentLeft + Scale(hwnd_, 440), contentBottom);
      RoundRect2(dc, bindingsCard, kCard, kBorder, Scale(hwnd_, 14));
      RECT bindTitle = bindingsCard;
      bindTitle.left += Scale(hwnd_, 18);
      bindTitle.top += Scale(hwnd_, 14);
      bindTitle.bottom = bindTitle.top + Scale(hwnd_, 24);
      Text(dc, L"Keys", bindTitle, kText, Scale(hwnd_, 15), true);

      const int rowTop = contentTop + Scale(hwnd_, 96);
      const int rowHeight = Scale(hwnd_, 40);
      for (size_t i = 0; i < settings_.bindings.size(); ++i) {
        const Binding& binding = settings_.bindings[i];
        const int top = rowTop + static_cast<int>(i) * rowHeight;
        RECT row = MakeRect(bindingsCard.left + Scale(hwnd_, 10), top,
                            bindingsCard.right - Scale(hwnd_, 10), top + rowHeight - Scale(hwnd_, 4));
        if (hot_ == static_cast<int>(i) || listening_row_ == static_cast<int>(i)) {
          RoundRect2(dc, row, kCardHot, kBorder, Scale(hwnd_, 8));
        }
        RECT label = row;
        label.left += Scale(hwnd_, 10);
        label.right -= Scale(hwnd_, 120);
        Text(dc, Widen(ActionLabel(binding.action)), label, kMuted, Scale(hwnd_, 12), false);

        RECT keyRect = MakeRect(row.right - Scale(hwnd_, 112), row.top + Scale(hwnd_, 4),
                                row.right - Scale(hwnd_, 8), row.bottom - Scale(hwnd_, 4));
        const bool listening = listening_row_ == static_cast<int>(i);
        RoundRect2(dc, keyRect, listening ? kAccentDeep : kPanel, listening ? kAccent : kBorder,
                   Scale(hwnd_, 6));
        Text(dc, Widen(listening ? "press a key" : binding.combo.ToString()), keyRect,
             listening ? kText : kAccentBright, Scale(hwnd_, 11), false,
             DT_CENTER | DT_SINGLELINE | DT_VCENTER);
      }

      const int hudLeft = bindingsCard.right + Scale(hwnd_, 20);
      RECT hudCard = MakeRect(hudLeft, contentTop, contentRight, contentBottom);
      RoundRect2(dc, hudCard, kCard, kBorder, Scale(hwnd_, 14));
      RECT hudTitle = hudCard;
      hudTitle.left += Scale(hwnd_, 18);
      hudTitle.top += Scale(hwnd_, 14);
      hudTitle.bottom = hudTitle.top + Scale(hwnd_, 24);
      Text(dc, L"HUD", hudTitle, kText, Scale(hwnd_, 15), true);
      RECT hudState = hudTitle;
      hudState.right -= Scale(hwnd_, 18);
      Text(dc, settings_.hud_enabled ? L"shown" : L"hidden", hudState,
           settings_.hud_enabled ? kGreen : kDim, Scale(hwnd_, 12), false,
           DT_RIGHT | DT_SINGLELINE | DT_VCENTER);

      const int hudRowTop = contentTop + Scale(hwnd_, 96);
      const int hudRowHeight = Scale(hwnd_, 44);
      for (size_t i = 0; i < settings_.hud.size(); ++i) {
        const HudElement& element = settings_.hud[i];
        const int top = hudRowTop + static_cast<int>(i) * hudRowHeight;
        RECT row = MakeRect(hudCard.left + Scale(hwnd_, 10), top, hudCard.right - Scale(hwnd_, 10),
                            top + hudRowHeight - Scale(hwnd_, 4));
        if (hot_ == static_cast<int>(i)) RoundRect2(dc, row, kCardHot, kBorder, Scale(hwnd_, 8));
        RECT label = row;
        label.left += Scale(hwnd_, 10);
        Text(dc, Widen(element.label), label, element.visible ? kText : kDim, Scale(hwnd_, 12), false);
        RECT anchor = row;
        anchor.right -= Scale(hwnd_, 90);
        Text(dc, Widen(AnchorId(element.anchor)), anchor, kDim, Scale(hwnd_, 11), false,
             DT_RIGHT | DT_SINGLELINE | DT_VCENTER);
        RECT toggle = MakeRect(row.right - Scale(hwnd_, 74), row.top + Scale(hwnd_, 6),
                               row.right - Scale(hwnd_, 8), row.bottom - Scale(hwnd_, 6));
        RoundRect2(dc, toggle, element.visible ? kAccentDeep : kPanel,
                   element.visible ? kAccent : kBorder, Scale(hwnd_, 6));
        Text(dc, element.visible ? L"on" : L"off", toggle, element.visible ? kText : kDim,
             Scale(hwnd_, 11), false, DT_CENTER | DT_SINGLELINE | DT_VCENTER);
      }
      break;
    }
  }

  // ---------- the status bar ----------
  FillRect2(dc, MakeRect(0, height_ - dpiStatus, width_, height_ - dpiStatus + 1), kBorder);
  FillRect2(dc, MakeRect(0, height_ - dpiStatus + 1, width_, height_), kPanel);
  RECT status = MakeRect(dpiRail + pad, height_ - dpiStatus, width_ - pad, height_);
  Text(dc, Widen(status_), status, kMuted, Scale(hwnd_, 11), false);
}

}  // namespace alpinoxi
