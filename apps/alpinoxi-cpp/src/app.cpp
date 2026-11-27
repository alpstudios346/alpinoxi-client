#include "app.h"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <wincrypt.h>

#include <algorithm>
#include <array>
#include <cctype>
#include <cstdio>
#include <cstring>

namespace alpinoxi {
namespace {

bool AllDigits(const std::string& text) {
  if (text.empty()) return false;
  for (char c : text) {
    if (c < '0' || c > '9') return false;
  }
  return true;
}

bool ReadNumber(const std::string& text, uint16_t* out) {
  if (!AllDigits(text) || text.size() > 5) return false;
  *out = static_cast<uint16_t>(std::stoul(text));
  return true;
}

// ---------- MD5, for the offline UUID ----------
// Straight from RFC 1321: the launcher needs exactly one hash, and a dependency
// for it would be a megabyte of code for sixteen bytes of output.

struct Md5State {
  uint32_t a = 0x67452301u, b = 0xefcdab89u, c = 0x98badcfeu, d = 0x10325476u;
};

uint32_t RotateLeft(uint32_t value, uint32_t count) {
  return (value << count) | (value >> (32 - count));
}

void Md5Block(Md5State* state, const uint8_t* block) {
  static const uint32_t kK[64] = {
      0xd76aa478u, 0xe8c7b756u, 0x242070dbu, 0xc1bdceeeu, 0xf57c0fafu, 0x4787c62au,
      0xa8304613u, 0xfd469501u, 0x698098d8u, 0x8b44f7afu, 0xffff5bb1u, 0x895cd7beu,
      0x6b901122u, 0xfd987193u, 0xa679438eu, 0x49b40821u, 0xf61e2562u, 0xc040b340u,
      0x265e5a51u, 0xe9b6c7aau, 0xd62f105du, 0x02441453u, 0xd8a1e681u, 0xe7d3fbc8u,
      0x21e1cde6u, 0xc33707d6u, 0xf4d50d87u, 0x455a14edu, 0xa9e3e905u, 0xfcefa3f8u,
      0x676f02d9u, 0x8d2a4c8au, 0xfffa3942u, 0x8771f681u, 0x6d9d6122u, 0xfde5380cu,
      0xa4beea44u, 0x4bdecfa9u, 0xf6bb4b60u, 0xbebfbc70u, 0x289b7ec6u, 0xeaa127fau,
      0xd4ef3085u, 0x04881d05u, 0xd9d4d039u, 0xe6db99e5u, 0x1fa27cf8u, 0xc4ac5665u,
      0xf4292244u, 0x432aff97u, 0xab9423a7u, 0xfc93a039u, 0x655b59c3u, 0x8f0ccc92u,
      0xffeff47du, 0x85845dd1u, 0x6fa87e4fu, 0xfe2ce6e0u, 0xa3014314u, 0x4e0811a1u,
      0xf7537e82u, 0xbd3af235u, 0x2ad7d2bbu, 0xeb86d391u};
  static const uint32_t kS[64] = {7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
                                  5, 9,  14, 20, 5, 9,  14, 20, 5, 9,  14, 20, 5, 9,  14, 20,
                                  4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
                                  6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21};

  uint32_t m[16];
  for (int i = 0; i < 16; ++i) {
    m[i] = static_cast<uint32_t>(block[i * 4]) | (static_cast<uint32_t>(block[i * 4 + 1]) << 8) |
           (static_cast<uint32_t>(block[i * 4 + 2]) << 16) |
           (static_cast<uint32_t>(block[i * 4 + 3]) << 24);
  }

  uint32_t a = state->a, b = state->b, c = state->c, d = state->d;
  for (uint32_t i = 0; i < 64; ++i) {
    uint32_t f = 0;
    uint32_t g = 0;
    if (i < 16) {
      f = (b & c) | (~b & d);
      g = i;
    } else if (i < 32) {
      f = (d & b) | (~d & c);
      g = (5 * i + 1) % 16;
    } else if (i < 48) {
      f = b ^ c ^ d;
      g = (3 * i + 5) % 16;
    } else {
      f = c ^ (b | ~d);
      g = (7 * i) % 16;
    }
    uint32_t temp = d;
    d = c;
    c = b;
    b = b + RotateLeft(a + f + kK[i] + m[g], kS[i]);
    a = temp;
  }
  state->a += a;
  state->b += b;
  state->c += c;
  state->d += d;
}

std::array<uint8_t, 16> Md5(const std::string& text) {
  Md5State state;
  std::vector<uint8_t> data(text.begin(), text.end());
  const uint64_t bitLength = static_cast<uint64_t>(data.size()) * 8;

  data.push_back(0x80);
  while (data.size() % 64 != 56) data.push_back(0x00);
  for (int i = 0; i < 8; ++i) data.push_back(static_cast<uint8_t>((bitLength >> (8 * i)) & 0xff));

  for (size_t offset = 0; offset < data.size(); offset += 64) Md5Block(&state, data.data() + offset);

  std::array<uint8_t, 16> digest{};
  const uint32_t words[4] = {state.a, state.b, state.c, state.d};
  for (int i = 0; i < 4; ++i) {
    for (int byte = 0; byte < 4; ++byte) {
      digest[i * 4 + byte] = static_cast<uint8_t>((words[i] >> (8 * byte)) & 0xff);
    }
  }
  return digest;
}

std::string Hex(const std::array<uint8_t, 16>& digest) {
  static const char* kDigits = "0123456789abcdef";
  std::string out;
  out.reserve(32);
  for (uint8_t byte : digest) {
    out.push_back(kDigits[byte >> 4]);
    out.push_back(kDigits[byte & 0x0f]);
  }
  return out;
}

}  // namespace

// ---------- GameVersion ----------

bool GameVersion::Parse(const std::string& text, GameVersion* out) {
  std::string trimmed;
  for (char c : text) {
    if (!std::isspace(static_cast<unsigned char>(c))) trimmed.push_back(c);
  }
  if (trimmed.empty()) return false;

  std::vector<std::string> parts;
  std::string current;
  for (char c : trimmed) {
    if (c == '.') {
      parts.push_back(current);
      current.clear();
    } else {
      current.push_back(c);
    }
  }
  parts.push_back(current);

  if (parts.size() < 2 || parts.size() > 3) return false;

  GameVersion parsed;
  if (!ReadNumber(parts[0], &parsed.major)) return false;
  if (!ReadNumber(parts[1], &parsed.minor)) return false;
  if (parts.size() == 3 && !ReadNumber(parts[2], &parsed.patch)) return false;

  *out = parsed;
  return true;
}

std::string GameVersion::ToString() const {
  char buffer[32];
  if (patch == 0) {
    std::snprintf(buffer, sizeof(buffer), "%u.%u", major, minor);
  } else {
    std::snprintf(buffer, sizeof(buffer), "%u.%u.%u", major, minor, patch);
  }
  return buffer;
}

bool GameVersion::operator<(const GameVersion& other) const {
  if (major != other.major) return major < other.major;
  if (minor != other.minor) return minor < other.minor;
  return patch < other.patch;
}

bool GameVersion::operator==(const GameVersion& other) const {
  return major == other.major && minor == other.minor && patch == other.patch;
}

bool GameVersion::AtLeast(uint16_t wantMajor, uint16_t wantMinor) const {
  if (major != wantMajor) return major > wantMajor;
  return minor >= wantMinor;
}

// ---------- Loader ----------

const std::vector<Loader>& AllLoaders() {
  static const std::vector<Loader> kLoaders = {Loader::kForge, Loader::kFabric, Loader::kQuilt};
  return kLoaders;
}

const char* LoaderId(Loader loader) {
  switch (loader) {
    case Loader::kForge:
      return "forge";
    case Loader::kFabric:
      return "fabric";
    case Loader::kQuilt:
      return "quilt";
  }
  return "forge";
}

const char* LoaderLabel(Loader loader) {
  switch (loader) {
    case Loader::kForge:
      return "Forge";
    case Loader::kFabric:
      return "Fabric";
    case Loader::kQuilt:
      return "Quilt";
  }
  return "Forge";
}

bool ParseLoader(const std::string& text, Loader* out) {
  std::string lower;
  for (char c : text) lower.push_back(static_cast<char>(std::tolower(static_cast<unsigned char>(c))));
  if (lower == "forge" || lower == "neoforge") {
    *out = Loader::kForge;
    return true;
  }
  if (lower == "fabric") {
    *out = Loader::kFabric;
    return true;
  }
  if (lower == "quilt") {
    *out = Loader::kQuilt;
    return true;
  }
  return false;
}

GameVersion LoaderEarliest(Loader loader) {
  switch (loader) {
    case Loader::kForge:
      return GameVersion{1, 1, 0};
    case Loader::kFabric:
      return GameVersion{1, 14, 0};
    case Loader::kQuilt:
      return GameVersion{1, 14, 0};
  }
  return GameVersion{1, 1, 0};
}

const char* LoaderMainClass(Loader loader) {
  switch (loader) {
    case Loader::kForge:
      return "net.minecraftforge.client.loading.ClientModLoader";
    case Loader::kFabric:
      return "net.fabricmc.loader.impl.launch.knot.KnotClient";
    case Loader::kQuilt:
      return "org.quiltmc.loader.impl.launch.knot.KnotClient";
  }
  return "";
}

const char* LoaderVersionsUrl(Loader loader) {
  switch (loader) {
    case Loader::kForge:
      return "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
    case Loader::kFabric:
      return "https://meta.fabricmc.net/v2/versions/game";
    case Loader::kQuilt:
      return "https://meta.quiltmc.org/v3/versions/game";
  }
  return "";
}

// ---------- LoaderTarget ----------

LoaderError LoaderTarget::Make(Loader loader, const std::string& version, LoaderTarget* out) {
  GameVersion parsed;
  if (!GameVersion::Parse(version, &parsed)) return LoaderError::kUnreadableVersion;

  const GameVersion earliest = LoaderEarliest(loader);
  if (!parsed.AtLeast(earliest.major, earliest.minor)) return LoaderError::kBeforeTheLoader;

  out->loader = loader;
  out->version = parsed;
  return LoaderError::kNone;
}

std::string LoaderTarget::Message(LoaderError error) const {
  switch (error) {
    case LoaderError::kNone:
      return "";
    case LoaderError::kUnreadableVersion:
      return "That version is not a release the launcher can install. Snapshots and "
             "pre-releases have no mod loader profile.";
    case LoaderError::kBeforeTheLoader: {
      const GameVersion earliest = LoaderEarliest(loader);
      return std::string(LoaderLabel(loader)) + " did not exist for this version. " +
             LoaderLabel(loader) + " starts at " + earliest.ToString() + ".";
    }
  }
  return "";
}

std::string LoaderTarget::ProfileUrl() const {
  const std::string tag = version.ToString();
  switch (loader) {
    case Loader::kFabric:
      return "https://meta.fabricmc.net/v2/versions/loader/" + tag + "/0.16.9";
    case Loader::kQuilt:
      return "https://meta.quiltmc.org/v3/versions/loader/" + tag + "/0.27.0";
    case Loader::kForge:
      // Forge has no per-version profile endpoint: its installer writes one.
      return "https://maven.minecraftforge.net/net/minecraftforge/forge/" + tag + "/forge-" + tag +
             "-installer.jar";
  }
  return "";
}

std::vector<Loader> LoadersFor(const std::string& version) {
  std::vector<Loader> loaders;
  GameVersion parsed;
  if (!GameVersion::Parse(version, &parsed)) return loaders;
  for (Loader loader : AllLoaders()) {
    const GameVersion earliest = LoaderEarliest(loader);
    if (parsed.AtLeast(earliest.major, earliest.minor)) loaders.push_back(loader);
  }
  return loaders;
}

// ---------- memory ----------

uint32_t PlanHeapMiB(uint32_t wanted, uint32_t systemMiB) {
  const uint32_t half = systemMiB / 2;
  uint32_t planned = wanted < half ? wanted : half;
  if (planned > 8192) planned = 8192;
  if (planned < 1024) planned = 1024;
  return planned;
}

// ---------- accounts ----------

std::string OfflineUuid(const std::string& name) {
  std::array<uint8_t, 16> digest = Md5("OfflinePlayer:" + name);
  // Version 3 (name-based MD5) and the RFC 4122 variant, exactly as Minecraft's
  // Java code sets them.
  digest[6] = static_cast<uint8_t>((digest[6] & 0x0f) | 0x30);
  digest[8] = static_cast<uint8_t>((digest[8] & 0x3f) | 0x80);

  const std::string hex = Hex(digest);
  return hex.substr(0, 8) + "-" + hex.substr(8, 4) + "-" + hex.substr(12, 4) + "-" +
         hex.substr(16, 4) + "-" + hex.substr(20, 12);
}

bool ValidPlayerName(const std::string& name) {
  if (name.size() < 3 || name.size() > 16) return false;
  for (char c : name) {
    const bool ok = std::isalnum(static_cast<unsigned char>(c)) || c == '_';
    if (!ok) return false;
  }
  return true;
}

}  // namespace alpinoxi

// ============================================================================
// Everything below was Rust until now. Same decisions, same tests, C++.
// ============================================================================

namespace alpinoxi {
namespace {

// ---------- SHA-256, for the PKCE challenge ----------

struct Sha256State {
  uint32_t h[8] = {0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
                   0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u};
};

uint32_t RotateRight(uint32_t value, uint32_t count) {
  return (value >> count) | (value << (32 - count));
}

void Sha256Block(Sha256State* state, const uint8_t* block) {
  static const uint32_t kK[64] = {
      0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u, 0x3956c25bu, 0x59f111f1u,
      0x923f82a4u, 0xab1c5ed5u, 0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u,
      0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u, 0xe49b69c1u, 0xefbe4786u,
      0x0fc19dc6u, 0x240ca1ccu, 0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
      0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u, 0xc6e00bf3u, 0xd5a79147u,
      0x06ca6351u, 0x14292967u, 0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u,
      0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u, 0xa2bfe8a1u, 0xa81a664bu,
      0xc24b8b70u, 0xc76c51a3u, 0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
      0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u, 0x391c0cb3u, 0x4ed8aa4au,
      0x5b9cca4fu, 0x682e6ff3u, 0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u,
      0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u};

  uint32_t w[64];
  for (int i = 0; i < 16; ++i) {
    w[i] = (static_cast<uint32_t>(block[i * 4]) << 24) |
           (static_cast<uint32_t>(block[i * 4 + 1]) << 16) |
           (static_cast<uint32_t>(block[i * 4 + 2]) << 8) |
           static_cast<uint32_t>(block[i * 4 + 3]);
  }
  for (int i = 16; i < 64; ++i) {
    const uint32_t s0 = RotateRight(w[i - 15], 7) ^ RotateRight(w[i - 15], 18) ^ (w[i - 15] >> 3);
    const uint32_t s1 = RotateRight(w[i - 2], 17) ^ RotateRight(w[i - 2], 19) ^ (w[i - 2] >> 10);
    w[i] = w[i - 16] + s0 + w[i - 7] + s1;
  }

  uint32_t a = state->h[0], b = state->h[1], c = state->h[2], d = state->h[3];
  uint32_t e = state->h[4], f = state->h[5], g = state->h[6], h = state->h[7];

  for (int i = 0; i < 64; ++i) {
    const uint32_t s1 = RotateRight(e, 6) ^ RotateRight(e, 11) ^ RotateRight(e, 25);
    const uint32_t ch = (e & f) ^ (~e & g);
    const uint32_t temp1 = h + s1 + ch + kK[i] + w[i];
    const uint32_t s0 = RotateRight(a, 2) ^ RotateRight(a, 13) ^ RotateRight(a, 22);
    const uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
    const uint32_t temp2 = s0 + maj;
    h = g;
    g = f;
    f = e;
    e = d + temp1;
    d = c;
    c = b;
    b = a;
    a = temp1 + temp2;
  }

  state->h[0] += a;
  state->h[1] += b;
  state->h[2] += c;
  state->h[3] += d;
  state->h[4] += e;
  state->h[5] += f;
  state->h[6] += g;
  state->h[7] += h;
}

std::array<uint8_t, 32> Sha256(const std::string& text) {
  Sha256State state;
  std::vector<uint8_t> data(text.begin(), text.end());
  const uint64_t bitLength = static_cast<uint64_t>(data.size()) * 8;

  data.push_back(0x80);
  while (data.size() % 64 != 56) data.push_back(0x00);
  for (int i = 7; i >= 0; --i) data.push_back(static_cast<uint8_t>((bitLength >> (8 * i)) & 0xff));

  for (size_t offset = 0; offset < data.size(); offset += 64) Sha256Block(&state, data.data() + offset);

  std::array<uint8_t, 32> digest{};
  for (int i = 0; i < 8; ++i) {
    for (int byte = 0; byte < 4; ++byte) {
      digest[i * 4 + byte] = static_cast<uint8_t>((state.h[i] >> (24 - 8 * byte)) & 0xff);
    }
  }
  return digest;
}

std::string Base64Url(const uint8_t* data, size_t size) {
  static const char* kAlphabet =
      "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
  std::string out;
  size_t i = 0;
  for (; i + 2 < size; i += 3) {
    const uint32_t chunk = (static_cast<uint32_t>(data[i]) << 16) |
                           (static_cast<uint32_t>(data[i + 1]) << 8) | data[i + 2];
    out.push_back(kAlphabet[(chunk >> 18) & 0x3f]);
    out.push_back(kAlphabet[(chunk >> 12) & 0x3f]);
    out.push_back(kAlphabet[(chunk >> 6) & 0x3f]);
    out.push_back(kAlphabet[chunk & 0x3f]);
  }
  if (i + 1 == size) {
    const uint32_t chunk = static_cast<uint32_t>(data[i]) << 16;
    out.push_back(kAlphabet[(chunk >> 18) & 0x3f]);
    out.push_back(kAlphabet[(chunk >> 12) & 0x3f]);
  } else if (i + 2 == size) {
    const uint32_t chunk = (static_cast<uint32_t>(data[i]) << 16) |
                           (static_cast<uint32_t>(data[i + 1]) << 8);
    out.push_back(kAlphabet[(chunk >> 18) & 0x3f]);
    out.push_back(kAlphabet[(chunk >> 12) & 0x3f]);
    out.push_back(kAlphabet[(chunk >> 6) & 0x3f]);
  }
  return out;
}

// ---------- JSON, as far as the launcher needs it ----------

// Skips a JSON string starting at `pos` (which must point at the quote) and
// returns the position after the closing quote.
size_t SkipString(const std::string& json, size_t pos) {
  if (pos >= json.size() || json[pos] != '"') return std::string::npos;
  ++pos;
  while (pos < json.size()) {
    if (json[pos] == '\\') {
      pos += 2;
      continue;
    }
    if (json[pos] == '"') return pos + 1;
    ++pos;
  }
  return std::string::npos;
}

std::string Unescape(const std::string& text) {
  std::string out;
  for (size_t i = 0; i < text.size(); ++i) {
    if (text[i] != '\\' || i + 1 >= text.size()) {
      out.push_back(text[i]);
      continue;
    }
    const char next = text[++i];
    switch (next) {
      case 'n':
        out.push_back('\n');
        break;
      case 't':
        out.push_back('\t');
        break;
      case 'r':
        out.push_back('\r');
        break;
      case '"':
        out.push_back('"');
        break;
      case '\\':
        out.push_back('\\');
        break;
      case '/':
        out.push_back('/');
        break;
      case 'u': {
        // The launcher only needs the ASCII range; anything else becomes '?'.
        out.push_back('?');
        i += 4;
        break;
      }
      default:
        out.push_back(next);
        break;
    }
  }
  return out;
}

// Finds "key" as an object key, at any depth.
size_t FindKey(const std::string& json, const std::string& key) {
  const std::string needle = "\"" + key + "\"";
  size_t pos = 0;
  while ((pos = json.find(needle, pos)) != std::string::npos) {
    size_t after = pos + needle.size();
    while (after < json.size() && std::isspace(static_cast<unsigned char>(json[after]))) ++after;
    if (after < json.size() && json[after] == ':') return after + 1;
    pos = after;
  }
  return std::string::npos;
}

}  // namespace

// ---------- JVM flags ----------

std::vector<std::string> JvmFlags(uint32_t heapMiB) {
  std::vector<std::string> flags = {
      "-XX:+UseG1GC",           "-XX:+ParallelRefProcEnabled", "-XX:MaxGCPauseMillis=200",
      "-XX:+UnlockExperimentalVMOptions", "-XX:+DisableExplicitGC", "-XX:G1NewSizePercent=30",
      "-XX:G1MaxNewSizePercent=40", "-XX:G1ReservePercent=20", "-XX:G1HeapWastePercent=5",
      "-XX:G1MixedGCCountTarget=4", "-XX:InitiatingHeapOccupancyPercent=15",
      "-XX:G1MixedGCLiveThresholdPercent=90", "-XX:G1RSetUpdatingPauseTimePercent=5",
      "-XX:SurvivorRatio=32",   "-XX:MaxTenuringThreshold=1", "-XX:+PerfDisableSharedMem",
      "-Dfile.encoding=UTF-8",
  };

  // Region size follows the heap: 8M under 12 GB, 16M above (Aikar).
  flags.push_back(heapMiB >= 12 * 1024 ? "-XX:G1HeapRegionSize=16M" : "-XX:G1HeapRegionSize=8M");

  // AlwaysPreTouch doubles start-up time on a client and touches RAM the player
  // may need for the browser. Only worth it on big heaps.
  if (heapMiB >= 6 * 1024) flags.push_back("-XX:+AlwaysPreTouch");

  flags.push_back("-Daikars.new.flags=true");
  flags.push_back("-Dusing.aikars.flags=https://mcflags.emc.gs");
  return flags;
}

// ---------- providers ----------

const std::vector<Provider>& AllProviders() {
  static const std::vector<Provider> kProviders = {Provider::kMicrosoft, Provider::kElyBy,
                                                   Provider::kOffline};
  return kProviders;
}

const char* ProviderId(Provider provider) {
  switch (provider) {
    case Provider::kMicrosoft:
      return "microsoft";
    case Provider::kElyBy:
      return "elyby";
    case Provider::kOffline:
      return "offline";
  }
  return "offline";
}

const char* ProviderLabel(Provider provider) {
  switch (provider) {
    case Provider::kMicrosoft:
      return "Microsoft";
    case Provider::kElyBy:
      return "Ely.by";
    case Provider::kOffline:
      return "Cracked";
  }
  return "Cracked";
}

bool ParseProvider(const std::string& text, Provider* out) {
  std::string lower;
  for (char c : text) lower.push_back(static_cast<char>(std::tolower(static_cast<unsigned char>(c))));
  if (lower == "microsoft" || lower == "ms") {
    *out = Provider::kMicrosoft;
    return true;
  }
  if (lower == "elyby" || lower == "ely.by") {
    *out = Provider::kElyBy;
    return true;
  }
  if (lower == "offline" || lower == "cracked") {
    *out = Provider::kOffline;
    return true;
  }
  return false;
}

Account MakeOfflineAccount(const std::string& name) {
  Account account;
  if (!ValidPlayerName(name)) return account;
  account.provider = Provider::kOffline;
  account.name = name;
  account.uuid = OfflineUuid(name);
  account.access_token = "0";  // what a cracked client sends
  return account;
}

// ---------- sign-in ----------

std::string RandomUrlSafe(size_t bytes) {
  std::vector<uint8_t> buffer(bytes);
  HCRYPTPROV provider = 0;
  if (!CryptAcquireContextW(&provider, nullptr, nullptr, PROV_RSA_FULL, CRYPT_VERIFYCONTEXT)) {
    return std::string();
  }
  const BOOL ok = CryptGenRandom(provider, static_cast<DWORD>(bytes), buffer.data());
  CryptReleaseContext(provider, 0);
  if (!ok) return std::string();
  return Base64Url(buffer.data(), buffer.size());
}

std::string PkceChallenge(const std::string& verifier) {
  const std::array<uint8_t, 32> digest = Sha256(verifier);
  return Base64Url(digest.data(), digest.size());
}

SignInRequest StartSignIn(Provider provider, const std::string& redirect_uri) {
  SignInRequest request;
  request.verifier = RandomUrlSafe(48);
  request.state = RandomUrlSafe(24);
  const std::string challenge = PkceChallenge(request.verifier);

  // The two providers share the shape and nothing else.
  if (provider == Provider::kMicrosoft) {
    request.authorize_url =
        "https://login.live.com/oauth20_authorize.srf"
        "?client_id=00000000402b5328"
        "&response_type=code&scope=XboxLive.signin%20offline_access&prompt=select_account"
        "&redirect_uri=" + redirect_uri + "&state=" + request.state + "&code_challenge=" +
        challenge + "&code_challenge_method=S256";
  } else if (provider == Provider::kElyBy) {
    request.authorize_url =
        "https://account.ely.by/oauth2/v1"
        "?client_id=alpinoxi-client&response_type=code&scope=account_info%20minecraft_server_session"
        "&redirect_uri=" + redirect_uri + "&state=" + request.state + "&code_challenge=" +
        challenge + "&code_challenge_method=S256";
  }
  return request;
}

// ---------- JSON ----------

bool JsonString(const std::string& json, const std::string& key, std::string* out) {
  size_t pos = FindKey(json, key);
  if (pos == std::string::npos) return false;
  while (pos < json.size() && std::isspace(static_cast<unsigned char>(json[pos]))) ++pos;
  if (pos >= json.size() || json[pos] != '"') return false;
  const size_t end = SkipString(json, pos);
  if (end == std::string::npos) return false;
  *out = Unescape(json.substr(pos + 1, end - pos - 2));
  return true;
}

bool JsonNumber(const std::string& json, const std::string& key, double* out) {
  size_t pos = FindKey(json, key);
  if (pos == std::string::npos) return false;
  while (pos < json.size() && std::isspace(static_cast<unsigned char>(json[pos]))) ++pos;
  const size_t start = pos;
  if (pos < json.size() && (json[pos] == '-' || json[pos] == '+')) ++pos;
  bool digits = false;
  while (pos < json.size() &&
         (std::isdigit(static_cast<unsigned char>(json[pos])) || json[pos] == '.' ||
          json[pos] == 'e' || json[pos] == 'E' || json[pos] == '-' || json[pos] == '+')) {
    if (std::isdigit(static_cast<unsigned char>(json[pos]))) digits = true;
    ++pos;
  }
  if (!digits) return false;
  *out = std::stod(json.substr(start, pos - start));
  return true;
}

int JsonArrayCount(const std::string& json, const std::string& key) {
  size_t pos = FindKey(json, key);
  if (pos == std::string::npos) return 0;
  while (pos < json.size() && std::isspace(static_cast<unsigned char>(json[pos]))) ++pos;
  if (pos >= json.size() || json[pos] != '[') return 0;

  int depth = 0;
  int count = 0;
  bool any = false;
  for (size_t i = pos; i < json.size(); ++i) {
    const char c = json[i];
    if (c == '"') {
      i = SkipString(json, i);
      if (i == std::string::npos) return count;
      any = true;
      continue;
    }
    if (c == '{' || c == '[') {
      ++depth;
      if (depth == 2 && c == '{') {
        ++count;
        any = true;
      }
      continue;
    }
    if (c == '}' || c == ']') {
      --depth;
      if (depth == 0) return count;
      continue;
    }
    if (depth == 1 && !any && !std::isspace(static_cast<unsigned char>(c))) any = true;
  }
  return count;
}

// ---------- the game's command line ----------

std::string QuoteForWindows(const std::string& argument) {
  if (argument.empty()) return "\"\"";
  bool needsQuotes = false;
  for (char c : argument) {
    if (c == ' ' || c == '\t' || c == '"') {
      needsQuotes = true;
      break;
    }
  }
  if (!needsQuotes) return argument;

  std::string quoted = "\"";
  for (char c : argument) {
    if (c == '"') quoted += "\\\"";
    else quoted.push_back(c);
  }
  quoted.push_back('"');
  return quoted;
}

LaunchCommand BuildLaunchCommand(const std::string& java, const std::string& game_dir,
                                 const std::string& assets_dir, const std::string& asset_index,
                                 const std::string& classpath, const LoaderTarget& target,
                                 const Account& account, uint32_t heapMiB) {
  LaunchCommand command;
  command.java = java;
  command.working_dir = game_dir;

  command.arguments.push_back("-Xms" + std::to_string(heapMiB) + "M");
  command.arguments.push_back("-Xmx" + std::to_string(heapMiB) + "M");
  for (const std::string& flag : JvmFlags(heapMiB)) command.arguments.push_back(flag);

  command.arguments.push_back("-cp");
  command.arguments.push_back(QuoteForWindows(classpath));
  command.arguments.push_back(LoaderMainClass(target.loader));

  command.arguments.push_back("--gameDir");
  command.arguments.push_back(QuoteForWindows(game_dir));
  command.arguments.push_back("--assetsDir");
  command.arguments.push_back(QuoteForWindows(assets_dir));
  command.arguments.push_back("--assetIndex");
  command.arguments.push_back(asset_index);
  command.arguments.push_back("--version");
  command.arguments.push_back(target.version.ToString());
  command.arguments.push_back("--username");
  command.arguments.push_back(account.name);
  command.arguments.push_back("--uuid");
  command.arguments.push_back(account.uuid);
  command.arguments.push_back("--accessToken");
  command.arguments.push_back(account.access_token.empty() ? "0" : account.access_token);
  command.arguments.push_back("--userType");
  command.arguments.push_back(account.provider == Provider::kOffline ? "legacy" : "msa");
  command.arguments.push_back("--versionType");
  command.arguments.push_back("release");
  return command;
}

}  // namespace alpinoxi
