// The window's entry point, in one declaration.

#pragma once

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

namespace alpinoxi {

// Creates the window and runs the message loop until it closes.
int Run(HINSTANCE instance, int show);

}  // namespace alpinoxi
