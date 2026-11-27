// The window: rail, top bar, content and status, drawn by Win32 itself.
//
// No browser engine, so the process starts in the time it takes Windows to
// create a window, and the memory it uses is the memory it draws with.

#ifndef UNICODE
#define UNICODE
#endif
#ifndef _UNICODE
#define _UNICODE
#endif
#define WIN32_LEAN_AND_MEAN

#include "main.h"

#include <commctrl.h>
#include <windowsx.h>

#include <string>

namespace alpinoxi {
namespace {

// ---------- layout ----------
constexpr int kRailWidth = 244;
constexpr int kTopBarHeight = 64;
constexpr int kPad = 24;
constexpr int kRowHeight = 44;
constexpr int kStatusHeight = 30;

// ---------- palette: Lunar's proportions, our blue ----------
const COLORREF kBg = RGB(0x06, 0x0b, 0x16);
const COLORREF kPanel = RGB(0x0a, 0x11, 0x22);
const COLORREF kCard = RGB(0x10, 0x1a, 0x2f);
const COLORREF kBorder = RGB(0x1d, 0x2c, 0x48);
const COLORREF kAccent = RGB(0x40, 0xa9, 0xff);
const COLORREF kAccentBright = RGB(0x62, 0xbc, 0xff);
const COLORREF kAccentDeep = RGB(0x1d, 0x7e, 0xd6);
const COLORREF kText = RGB(0xe8, 0xf1, 0xff);
const COLORREF kMuted = RGB(0x8b, 0xa3, 0xc7);
const COLORREF kDim = RGB(0x5f, 0x7a, 0x9e);
const COLORREF kInk = RGB(0x04, 0x10, 0x1f);

const wchar_t* const kClassName = L"AlpinoxiClientWindow";

// ---------- state ----------
HWND gWindow = nullptr;
HFONT gFontUi = nullptr;
HFONT gFontTitle = nullptr;
HFONT gFontSmall = nullptr;
int gScreen = 0;
int gHoverRow = -1;
int gHoverAction = -1;
std::wstring gStatus = L"Ready";

struct Screen {
  const wchar_t* title;
  const wchar_t* subtitle;
};

const Screen kScreens[] = {
    {L"Home", L"Your default instance, ready to launch."},
    {L"Instances", L"Every game version you have installed."},
    {L"Skin", L"Your skin, from your account or a file."},
    {L"Friends", L"Who is online right now."},
    {L"Mods", L"The catalogue, and what is installed."},
    {L"Import", L"Bring in an existing setup."},
    {L"Settings", L"Memory, keybinds and the HUD."},
};
constexpr int kScreenCount = static_cast<int>(sizeof(kScreens) / sizeof(kScreens[0]));

// ---------- drawing helpers ----------

HFONT MakeFont(int height, int weight) {
  return CreateFontW(height, 0, 0, 0, weight, FALSE, FALSE, FALSE, DEFAULT_CHARSET,
                     OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY,
                     DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI");
}

void Fill(HDC dc, const RECT& rect, COLORREF colour) {
  HBRUSH brush = CreateSolidBrush(colour);
  FillRect(dc, &rect, brush);
  DeleteObject(brush);
}

void Label(HDC dc, const std::wstring& text, RECT rect, COLORREF colour, HFONT font, UINT format) {
  SetBkMode(dc, TRANSPARENT);
  SetTextColor(dc, colour);
  HGDIOBJ old = SelectObject(dc, font);
  DrawTextW(dc, text.c_str(), -1, &rect, format);
  SelectObject(dc, old);
}

// A rounded panel with a hairline border, the shape the site uses.
void Panel(HDC dc, const RECT& rect, COLORREF fill, COLORREF edge) {
  HBRUSH brush = CreateSolidBrush(fill);
  HPEN pen = CreatePen(PS_SOLID, 1, edge);
  HGDIOBJ oldBrush = SelectObject(dc, brush);
  HGDIOBJ oldPen = SelectObject(dc, pen);
  RoundRect(dc, rect.left, rect.top, rect.right, rect.bottom, 14, 14);
  SelectObject(dc, oldBrush);
  SelectObject(dc, oldPen);
  DeleteObject(brush);
  DeleteObject(pen);
}

void PrimaryButton(HDC dc, const RECT& rect, const wchar_t* label, bool hovered) {
  HBRUSH brush = CreateSolidBrush(hovered ? kAccentBright : kAccent);
  HPEN pen = CreatePen(PS_SOLID, 1, kAccentDeep);
  HGDIOBJ oldBrush = SelectObject(dc, brush);
  HGDIOBJ oldPen = SelectObject(dc, pen);
  RoundRect(dc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);
  SelectObject(dc, oldBrush);
  SelectObject(dc, oldPen);
  DeleteObject(brush);
  DeleteObject(pen);
  RECT text = rect;
  Label(dc, label, text, kInk, gFontUi, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
}

void GhostButton(HDC dc, const RECT& rect, const wchar_t* label, bool hovered) {
  HBRUSH brush = CreateSolidBrush(kPanel);
  HPEN pen = CreatePen(PS_SOLID, 1, hovered ? kAccent : kBorder);
  HGDIOBJ oldBrush = SelectObject(dc, brush);
  HGDIOBJ oldPen = SelectObject(dc, pen);
  RoundRect(dc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);
  SelectObject(dc, oldBrush);
  SelectObject(dc, oldPen);
  DeleteObject(brush);
  DeleteObject(pen);
  RECT text = rect;
  Label(dc, label, text, hovered ? kAccentBright : kText, gFontUi,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE);
}

// The blue square with the letter, standing in for the logo inside the window.
// The window's own icon is the user's file, loaded from build/icon.ico.
void Mark(HDC dc, int x, int y, int size) {
  RECT rect{x, y, x + size, y + size};
  Panel(dc, rect, kAccent, kAccentDeep);
  Label(dc, L"A", rect, kInk, gFontTitle, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
}

RECT NavRowRect(int index) {
  const int top = kTopBarHeight + 28 + index * kRowHeight;
  return RECT{kPad, top, kRailWidth - kPad, top + kRowHeight - 6};
}

RECT LaunchRect(const RECT& client) {
  return RECT{client.right - kPad - 200, 12, client.right - kPad - 100, 52};
}

RECT SettingsRect(const RECT& client) {
  return RECT{client.right - kPad - 92, 12, client.right - kPad, 52};
}

// ---------- the four bands ----------

void PaintRail(HDC dc, const RECT& client) {
  // A vertical gradient, band by band: plain GDI has no gradient fill.
  const int bands = 48;
  const int height = client.bottom;
  for (int i = 0; i < bands; ++i) {
    const int top = height * i / bands;
    const int bottom = height * (i + 1) / bands;
    const int t = 255 * i / bands;
    const int r = 0x0c + (0x08 - 0x0c) * t / 255;
    const int g = 0x15 + (0x0f - 0x15) * t / 255;
    const int b = 0x26 + (0x1d - 0x26) * t / 255;
    RECT band{0, top, kRailWidth, bottom};
    Fill(dc, band, RGB(r, g, b));
  }
  RECT edge{kRailWidth - 1, 0, kRailWidth, client.bottom};
  Fill(dc, edge, kBorder);

  Mark(dc, kPad, 20, 34);
  RECT brand{kPad + 44, 20, kRailWidth - kPad, 54};
  Label(dc, L"Alpinoxi", brand, kText, gFontTitle, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

  for (int i = 0; i < kScreenCount; ++i) {
    RECT row = NavRowRect(i);
    if (i == gScreen) {
      Fill(dc, row, RGB(0x12, 0x24, 0x3d));
      RECT stripe{row.left, row.top, row.left + 3, row.bottom};
      Fill(dc, stripe, kAccent);
    } else if (i == gHoverRow) {
      Fill(dc, row, RGB(0x0e, 0x1a, 0x2e));
    }
    RECT text{row.left + 16, row.top, row.right, row.bottom};
    Label(dc, kScreens[i].title, text, i == gScreen ? kAccentBright : kMuted, gFontUi,
          DT_LEFT | DT_VCENTER | DT_SINGLELINE);
  }

  RECT footer{kPad, client.bottom - 46, kRailWidth - kPad, client.bottom - 22};
  Label(dc, L"v0.4.0 \xC2\xB7 C++ native", footer, kDim, gFontSmall,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE);
}

void PaintTopBar(HDC dc, const RECT& client) {
  RECT bar{kRailWidth, 0, client.right, kTopBarHeight};
  Fill(dc, bar, kPanel);
  RECT edge{kRailWidth, kTopBarHeight - 1, client.right, kTopBarHeight};
  Fill(dc, edge, kBorder);

  RECT name{kRailWidth + kPad, 0, client.right - 340, kTopBarHeight};
  Label(dc, L"Not signed in", name, kMuted, gFontUi, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

  PrimaryButton(dc, LaunchRect(client), L"Launch", gHoverAction == 0);
  GhostButton(dc, SettingsRect(client), L"Settings", gHoverAction == 1);
}

void PaintContent(HDC dc, const RECT& client) {
  const Screen& screen = kScreens[gScreen];
  const int left = kRailWidth + kPad;
  const int right = client.right - kPad;
  int top = kTopBarHeight + kPad;

  RECT title{left, top, right, top + 34};
  Label(dc, screen.title, title, kText, gFontTitle, DT_LEFT | DT_TOP | DT_SINGLELINE);
  top += 40;
  RECT subtitle{left, top, right, top + 24};
  Label(dc, screen.subtitle, subtitle, kMuted, gFontUi, DT_LEFT | DT_TOP | DT_SINGLELINE);
  top += 44;

  if (top + 100 > client.bottom - kStatusHeight - kPad) return;

  RECT card{left, top, right, top + 96};
  Panel(dc, card, kCard, kBorder);
  Mark(dc, left + 20, top + 22, 52);
  RECT kicker{left + 88, top + 18, right - 20, top + 38};
  Label(dc, L"DEFAULT INSTANCE", kicker, kDim, gFontSmall, DT_LEFT | DT_TOP | DT_SINGLELINE);
  RECT instance{left + 88, top + 38, right - 20, top + 66};
  Label(dc, L"Fabric 1.21.1", instance, kText, gFontUi, DT_LEFT | DT_TOP | DT_SINGLELINE);
  RECT state{left + 88, top + 66, right - 20, top + 88};
  Label(dc, L"Ready to launch", state, kMuted, gFontSmall, DT_LEFT | DT_TOP | DT_SINGLELINE);
}

void PaintStatus(HDC dc, const RECT& client) {
  RECT bar{kRailWidth, client.bottom - kStatusHeight, client.right, client.bottom};
  Fill(dc, bar, kPanel);
  RECT edge{bar.left, bar.top, bar.right, bar.top + 1};
  Fill(dc, edge, kBorder);
  RECT text{bar.left + kPad, bar.top, bar.right - kPad, bar.bottom};
  Label(dc, gStatus, text, kMuted, gFontSmall, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
}

void Paint(HWND window) {
  PAINTSTRUCT paint;
  HDC dc = BeginPaint(window, &paint);
  RECT client;
  GetClientRect(window, &client);

  // Double-buffered: a gradient drawn straight onto the window flickers every
  // time the mouse crosses a row.
  HDC buffer = CreateCompatibleDC(dc);
  HBITMAP bitmap = CreateCompatibleBitmap(dc, client.right > 0 ? client.right : 1,
                                          client.bottom > 0 ? client.bottom : 1);
  HGDIOBJ oldBitmap = SelectObject(buffer, bitmap);

  Fill(buffer, client, kBg);
  PaintRail(buffer, client);
  PaintTopBar(buffer, client);
  PaintContent(buffer, client);
  PaintStatus(buffer, client);

  BitBlt(dc, 0, 0, client.right, client.bottom, buffer, 0, 0, SRCCOPY);
  SelectObject(buffer, oldBitmap);
  DeleteObject(bitmap);
  DeleteDC(buffer);
  EndPaint(window, &paint);
}

// 0..kScreenCount-1: a nav row. 100: Launch. 101: Settings. -1: nothing.
int HitTest(const RECT& client, POINT point) {
  if (point.x < kRailWidth) {
    for (int i = 0; i < kScreenCount; ++i) {
      RECT row = NavRowRect(i);
      if (PtInRect(&row, point)) return i;
    }
    return -1;
  }
  RECT launch = LaunchRect(client);
  if (PtInRect(&launch, point)) return 100;
  RECT settings = SettingsRect(client);
  if (PtInRect(&settings, point)) return 101;
  return -1;
}

void OnClick(int hit) {
  if (hit >= 0 && hit < kScreenCount) {
    gScreen = hit;
    gStatus = std::wstring(L"Showing ") + kScreens[hit].title;
  } else if (hit == 100) {
    // Launching is the installer's job, and it is the next piece of work.
    gStatus = L"Launch needs the installer: downloading the game comes next.";
  } else if (hit == 101) {
    gScreen = kScreenCount - 1;
    gStatus = L"Showing Settings";
  }
  InvalidateRect(gWindow, nullptr, FALSE);
}

LRESULT CALLBACK WindowProc(HWND window, UINT message, WPARAM wParam, LPARAM lParam) {
  switch (message) {
    case WM_PAINT:
      Paint(window);
      return 0;

    case WM_ERASEBKGND:
      return 1;  // the buffer covers every pixel

    case WM_SIZE:
      InvalidateRect(window, nullptr, FALSE);
      return 0;

    case WM_MOUSEMOVE: {
      RECT client;
      GetClientRect(window, &client);
      const POINT point{GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam)};
      const int hit = HitTest(client, point);
      const int row = (hit >= 0 && hit < kScreenCount) ? hit : -1;
      const int action = (hit == 100) ? 0 : (hit == 101) ? 1 : -1;
      if (row != gHoverRow || action != gHoverAction) {
        gHoverRow = row;
        gHoverAction = action;
        InvalidateRect(window, nullptr, FALSE);
      }
      TRACKMOUSEEVENT track{};
      track.cbSize = sizeof(track);
      track.dwFlags = TME_LEAVE;
      track.hwndTrack = window;
      TrackMouseEvent(&track);
      return 0;
    }

    case WM_MOUSELEAVE:
      gHoverRow = -1;
      gHoverAction = -1;
      InvalidateRect(window, nullptr, FALSE);
      return 0;

    case WM_LBUTTONDOWN: {
      RECT client;
      GetClientRect(window, &client);
      const POINT point{GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam)};
      OnClick(HitTest(client, point));
      return 0;
    }

    case WM_KEYDOWN:
      if (wParam == VK_ESCAPE) {
        PostMessageW(window, WM_CLOSE, 0, 0);
        return 0;
      }
      if (wParam == VK_DOWN || wParam == VK_UP) {
        const int step = (wParam == VK_DOWN) ? 1 : -1;
        gScreen = (gScreen + step + kScreenCount) % kScreenCount;
        InvalidateRect(window, nullptr, FALSE);
        return 0;
      }
      return 0;

    case WM_DESTROY:
      PostQuitMessage(0);
      return 0;

    default:
      return DefWindowProcW(window, message, wParam, lParam);
  }
}

}  // namespace

int Run(HINSTANCE instance, int show) {
  INITCOMMONCONTROLSEX controls{};
  controls.dwSize = sizeof(controls);
  controls.dwICC = ICC_STANDARD_CLASSES;
  InitCommonControlsEx(&controls);

  gFontUi = MakeFont(-15, FW_NORMAL);
  gFontTitle = MakeFont(-20, FW_BOLD);
  gFontSmall = MakeFont(-12, FW_NORMAL);

  // The window icon is the user's own file, loaded from beside the executable.
  // It is never redrawn or replaced.
  wchar_t exePath[MAX_PATH] = {};
  GetModuleFileNameW(nullptr, exePath, MAX_PATH);
  std::wstring folder(exePath);
  const size_t slash = folder.find_last_of(L'\\');
  if (slash != std::wstring::npos) folder = folder.substr(0, slash);
  const std::wstring iconPath = folder + L"\\assets\\icon.ico";
  HICON iconLarge = static_cast<HICON>(
      LoadImageW(nullptr, iconPath.c_str(), IMAGE_ICON, 0, 0, LR_LOADFROMFILE | LR_DEFAULTSIZE));
  HICON iconSmall = static_cast<HICON>(
      LoadImageW(nullptr, iconPath.c_str(), IMAGE_ICON, GetSystemMetrics(SM_CXSMICON),
                 GetSystemMetrics(SM_CYSMICON), LR_LOADFROMFILE));

  WNDCLASSEXW windowClass{};
  windowClass.cbSize = sizeof(windowClass);
  windowClass.style = CS_HREDRAW | CS_VREDRAW;
  windowClass.lpfnWndProc = WindowProc;
  windowClass.hInstance = instance;
  windowClass.hIcon = iconLarge;
  windowClass.hIconSm = iconSmall;
  windowClass.hCursor = LoadCursorW(nullptr, IDC_ARROW);
  windowClass.hbrBackground = nullptr;  // every pixel is painted
  windowClass.lpszClassName = kClassName;
  if (!RegisterClassExW(&windowClass)) return 1;

  gWindow = CreateWindowExW(0, kClassName, L"Alpinoxi Client", WS_OVERLAPPEDWINDOW, CW_USEDEFAULT,
                            CW_USEDEFAULT, 1180, 760, nullptr, nullptr, instance, nullptr);
  if (gWindow == nullptr) return 1;

  ShowWindow(gWindow, show);
  UpdateWindow(gWindow);

  MSG message{};
  while (GetMessageW(&message, nullptr, 0, 0) > 0) {
    TranslateMessage(&message);
    DispatchMessageW(&message);
  }

  DeleteObject(gFontUi);
  DeleteObject(gFontTitle);
  DeleteObject(gFontSmall);
  if (iconLarge) DestroyIcon(iconLarge);
  if (iconSmall) DestroyIcon(iconSmall);
  return static_cast<int>(message.wParam);
}

}  // namespace alpinoxi
