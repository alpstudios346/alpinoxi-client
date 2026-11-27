// The window, drawn by Win32 and GDI themselves.
//
// The palette is the project's blue: a deep navy that leans blue rather than
// black, glass panels with a hairline edge, and one bright accent that is only
// used where something can be pressed.

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
constexpr int kRailWidth = 236;
constexpr int kTopBarHeight = 68;
constexpr int kPad = 26;
constexpr int kRowHeight = 42;
constexpr int kStatusHeight = 30;

// ---------- palette: blue, all the way down ----------
const COLORREF kBg = RGB(0x07, 0x0f, 0x20);
const COLORREF kPanel = RGB(0x0b, 0x16, 0x2c);
const COLORREF kCard = RGB(0x10, 0x1e, 0x39);
const COLORREF kCardTop = RGB(0x14, 0x25, 0x45);
const COLORREF kBorder = RGB(0x1e, 0x35, 0x59);
const COLORREF kBorderLit = RGB(0x2c, 0x52, 0x86);
const COLORREF kAccent = RGB(0x3d, 0x9b, 0xff);
const COLORREF kAccentBright = RGB(0x6f, 0xc2, 0xff);
const COLORREF kAccentDeep = RGB(0x1b, 0x6c, 0xd6);
const COLORREF kText = RGB(0xea, 0xf3, 0xff);
const COLORREF kMuted = RGB(0x94, 0xae, 0xd2);
const COLORREF kDim = RGB(0x5b, 0x76, 0x9e);
const COLORREF kInk = RGB(0x04, 0x0f, 0x22);

const wchar_t* const kClassName = L"AlpinoxiClientWindow";

// ---------- state ----------
HWND gWindow = nullptr;
HFONT gFontUi = nullptr;
HFONT gFontTitle = nullptr;
HFONT gFontSmall = nullptr;
HFONT gFontBrand = nullptr;
HFONT gFontHero = nullptr;
int gScreen = 0;
int gHoverRow = -1;
int gHoverAction = -1;
std::wstring gStatus = L"Ready";
std::wstring gInstance = L"Fabric 1.21.1";
std::wstring gAccount = L"Not signed in";
bool gSignedIn = false;

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

HFONT MakeFont(int height, int weight, const wchar_t* face = L"Segoe UI") {
  return CreateFontW(height, 0, 0, 0, weight, FALSE, FALSE, FALSE, DEFAULT_CHARSET,
                     OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY,
                     DEFAULT_PITCH | FF_DONTCARE, face);
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

// A vertical gradient in bands: plain GDI has no gradient fill, and a band of
// one or two pixels is indistinguishable from a real one at this size.
void Gradient(HDC dc, const RECT& rect, COLORREF from, COLORREF to, int bands = 64) {
  const int height = rect.bottom - rect.top;
  if (height <= 0) return;
  const int fromR = GetRValue(from), fromG = GetGValue(from), fromB = GetBValue(from);
  const int toR = GetRValue(to), toG = GetGValue(to), toB = GetBValue(to);
  for (int i = 0; i < bands; ++i) {
    const int top = rect.top + height * i / bands;
    const int bottom = rect.top + height * (i + 1) / bands;
    const int t = 255 * i / (bands > 1 ? bands - 1 : 1);
    RECT band{rect.left, top, rect.right, bottom};
    Fill(dc, band,
         RGB(fromR + (toR - fromR) * t / 255, fromG + (toG - fromG) * t / 255,
             fromB + (toB - fromB) * t / 255));
  }
}

// A horizontal gradient, for the accent line under the top bar.
void GradientAcross(HDC dc, const RECT& rect, COLORREF from, COLORREF to, int bands = 96) {
  const int width = rect.right - rect.left;
  if (width <= 0) return;
  const int fromR = GetRValue(from), fromG = GetGValue(from), fromB = GetBValue(from);
  const int toR = GetRValue(to), toG = GetGValue(to), toB = GetBValue(to);
  for (int i = 0; i < bands; ++i) {
    const int left = rect.left + width * i / bands;
    const int right = rect.left + width * (i + 1) / bands;
    const int t = 255 * i / (bands > 1 ? bands - 1 : 1);
    RECT band{left, rect.top, right, rect.bottom};
    Fill(dc, band,
         RGB(fromR + (toR - fromR) * t / 255, fromG + (toG - fromG) * t / 255,
             fromB + (toB - fromB) * t / 255));
  }
}

void Panel(HDC dc, const RECT& rect, COLORREF fill, COLORREF edge, int radius = 14) {
  HBRUSH brush = CreateSolidBrush(fill);
  HPEN pen = CreatePen(PS_SOLID, 1, edge);
  HGDIOBJ oldBrush = SelectObject(dc, brush);
  HGDIOBJ oldPen = SelectObject(dc, pen);
  RoundRect(dc, rect.left, rect.top, rect.right, rect.bottom, radius, radius);
  SelectObject(dc, oldBrush);
  SelectObject(dc, oldPen);
  DeleteObject(brush);
  DeleteObject(pen);
}

// A card: glass fill, a hairline edge, and a brighter top edge so it reads as
// lifted off the background rather than cut into it.
void Card(HDC dc, const RECT& rect) {
  Panel(dc, rect, kCard, kBorder);
  RECT lit{rect.left + 14, rect.top, rect.right - 14, rect.top + 1};
  Fill(dc, lit, kBorderLit);
}

void PrimaryButton(HDC dc, const RECT& rect, const wchar_t* label, bool hovered) {
  // Fill first, then the edge: the pen draws over the gradient's rim, which is
  // what makes the button look pressed into the bar.
  Gradient(dc, rect, hovered ? kAccentBright : kAccent, kAccentDeep, 28);
  Panel(dc, rect, RGB(0, 0, 0), kAccentDeep, 10);
  RECT text = rect;
  Label(dc, label, text, kInk, gFontUi, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
}

void GhostButton(HDC dc, const RECT& rect, const wchar_t* label, bool hovered) {
  Panel(dc, rect, hovered ? kCardTop : kPanel, hovered ? kAccent : kBorder, 10);
  RECT text = rect;
  Label(dc, label, text, hovered ? kAccentBright : kText, gFontUi,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE);
}

// The blue tile with the letter. The window's own icon is the user's own file,
// loaded from assets/icon.ico and never redrawn.
void Mark(HDC dc, int x, int y, int size) {
  RECT rect{x, y, x + size, y + size};
  Gradient(dc, rect, kAccentBright, kAccentDeep, 24);
  Panel(dc, rect, RGB(0, 0, 0), kAccentDeep, size / 4);
  Label(dc, L"A", rect, kInk, gFontBrand, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
}

RECT NavRowRect(int index) {
  const int top = kTopBarHeight + 30 + index * kRowHeight;
  return RECT{kPad - 8, top, kRailWidth - kPad + 8, top + kRowHeight - 6};
}

RECT LaunchRect(const RECT& client) {
  return RECT{client.right - kPad - 196, 14, client.right - kPad - 88, 54};
}

RECT SettingsRect(const RECT& client) {
  return RECT{client.right - kPad - 80, 14, client.right - kPad, 54};
}

RECT AccountChipRect() {
  return RECT{kRailWidth + kPad, 15, kRailWidth + kPad + 230, 53};
}

// ---------- the bands ----------

void PaintRail(HDC dc, const RECT& client) {
  RECT rail{0, 0, kRailWidth, client.bottom};
  Gradient(dc, rail, RGB(0x0a, 0x15, 0x2c), RGB(0x0c, 0x1b, 0x38));

  RECT edge{kRailWidth - 1, 0, kRailWidth, client.bottom};
  Fill(dc, edge, kBorder);

  Mark(dc, kPad, 22, 36);
  RECT brand{kPad + 46, 22, kRailWidth - kPad, 58};
  Label(dc, L"Alpinoxi", brand, kText, gFontTitle, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

  for (int i = 0; i < kScreenCount; ++i) {
    RECT row = NavRowRect(i);
    if (i == gScreen) {
      Panel(dc, row, RGB(0x15, 0x2a, 0x4c), kBorderLit, 9);
      RECT stripe{row.left + 3, row.top + 7, row.left + 6, row.bottom - 7};
      Fill(dc, stripe, kAccent);
    } else if (i == gHoverRow) {
      Panel(dc, row, RGB(0x11, 0x22, 0x3f), kBorder, 9);
    }
    RECT text{row.left + 18, row.top, row.right, row.bottom};
    Label(dc, kScreens[i].title, text, i == gScreen ? kAccentBright : kMuted, gFontUi,
          DT_LEFT | DT_VCENTER | DT_SINGLELINE);
  }

  // The account sits at the foot of the rail, where a launcher's account
  // belongs: one row, one name, one provider.
  RECT account{kPad - 8, client.bottom - 92, kRailWidth - kPad + 8, client.bottom - 44};
  Panel(dc, account, gSignedIn ? RGB(0x12, 0x26, 0x45) : RGB(0x0f, 0x1d, 0x36), kBorder, 10);
  RECT name{account.left + 14, account.top + 8, account.right - 14, account.top + 26};
  Label(dc, gAccount, name, kText, gFontSmall, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
  RECT provider{account.left + 14, account.top + 24, account.right - 14, account.bottom - 4};
  Label(dc, gSignedIn ? L"Microsoft" : L"Offline \xC2\xB7 Cracked", provider, kDim, gFontSmall,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE);

  RECT footer{kPad, client.bottom - 34, kRailWidth - kPad, client.bottom - 14};
  Label(dc, L"v0.4.0 \xC2\xB7 C++ native", footer, kDim, gFontSmall,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE);
}

void PaintTopBar(HDC dc, const RECT& client) {
  RECT bar{kRailWidth, 0, client.right, kTopBarHeight};
  Fill(dc, bar, kPanel);
  RECT edge{kRailWidth, kTopBarHeight - 1, client.right, kTopBarHeight};
  Fill(dc, edge, kBorder);
  RECT glow{kRailWidth, kTopBarHeight - 3, client.right, kTopBarHeight - 2};
  GradientAcross(dc, glow, RGB(0x0b, 0x16, 0x2c), RGB(0x0b, 0x16, 0x2c));

  RECT chip = AccountChipRect();
  Panel(dc, chip, kCard, kBorder, 10);
  RECT dot{chip.left + 12, chip.top + 15, chip.left + 21, chip.top + 24};
  HBRUSH dotBrush = CreateSolidBrush(gSignedIn ? kAccent : kDim);
  HGDIOBJ old = SelectObject(dc, dotBrush);
  HGDIOBJ oldPen = SelectObject(dc, GetStockObject(NULL_PEN));
  Ellipse(dc, dot.left, dot.top, dot.right, dot.bottom);
  SelectObject(dc, oldPen);
  SelectObject(dc, old);
  DeleteObject(dotBrush);
  RECT name{chip.left + 30, chip.top, chip.right - 10, chip.bottom};
  Label(dc, gAccount, name, kMuted, gFontUi, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

  PrimaryButton(dc, LaunchRect(client), L"Launch", gHoverAction == 0);
  GhostButton(dc, SettingsRect(client), L"Settings", gHoverAction == 1);
}

void PaintContent(HDC dc, const RECT& client) {
  const Screen& screen = kScreens[gScreen];
  const int left = kRailWidth + kPad;
  const int right = client.right - kPad;
  int top = kTopBarHeight + kPad;

  RECT title{left, top, right, top + 38};
  Label(dc, screen.title, title, kText, gFontHero, DT_LEFT | DT_TOP | DT_SINGLELINE);
  top += 44;
  RECT subtitle{left, top, right, top + 24};
  Label(dc, screen.subtitle, subtitle, kMuted, gFontUi, DT_LEFT | DT_TOP | DT_SINGLELINE);
  top += 42;

  if (top + 120 > client.bottom - kStatusHeight - kPad) return;

  // The instance card: the one thing on this screen you can act on.
  RECT card{left, top, right, top + 116};
  Card(dc, card);
  Mark(dc, left + 22, top + 30, 56);
  RECT kicker{left + 96, top + 24, right - 24, top + 44};
  Label(dc, L"DEFAULT INSTANCE", kicker, kDim, gFontSmall, DT_LEFT | DT_TOP | DT_SINGLELINE);
  RECT instance{left + 96, top + 44, right - 24, top + 76};
  Label(dc, gInstance, instance, kText, gFontTitle, DT_LEFT | DT_TOP | DT_SINGLELINE);
  RECT state{left + 96, top + 78, right - 24, top + 100};
  Label(dc, L"Ready to launch", state, kMuted, gFontSmall, DT_LEFT | DT_TOP | DT_SINGLELINE);

  // A row of three figures: heap, loader and what is installed.
  top += 136;
  if (top + 96 > client.bottom - kStatusHeight - kPad) return;
  const int gap = 16;
  const int width = (right - left - gap * 2) / 3;
  const wchar_t* labels[3] = {L"HEAP", L"LOADER", L"INSTALLED"};
  const wchar_t* values[3] = {L"4096 MiB", gInstance.c_str(), L"0 instances"};
  for (int i = 0; i < 3; ++i) {
    RECT tile{left + i * (width + gap), top, left + i * (width + gap) + width, top + 92};
    Card(dc, tile);
    RECT tileLabel{tile.left + 18, tile.top + 18, tile.right - 18, tile.top + 38};
    Label(dc, labels[i], tileLabel, kDim, gFontSmall, DT_LEFT | DT_TOP | DT_SINGLELINE);
    RECT tileValue{tile.left + 18, tile.top + 42, tile.right - 18, tile.top + 72};
    Label(dc, values[i], tileValue, kText, gFontTitle, DT_LEFT | DT_TOP | DT_SINGLELINE);
  }
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
  gFontTitle = MakeFont(-19, FW_SEMIBOLD);
  gFontSmall = MakeFont(-12, FW_NORMAL);
  gFontBrand = MakeFont(-18, FW_BOLD);
  gFontHero = MakeFont(-26, FW_SEMIBOLD);

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
                            CW_USEDEFAULT, 1200, 780, nullptr, nullptr, instance, nullptr);
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
  DeleteObject(gFontBrand);
  DeleteObject(gFontHero);
  if (iconLarge) DestroyIcon(iconLarge);
  if (iconSmall) DestroyIcon(iconSmall);
  return static_cast<int>(message.wParam);
}

}  // namespace alpinoxi
