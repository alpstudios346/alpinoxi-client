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
    const uint32_t temp = d;
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
      default:
        out.push_back(next);
        break;
    }
  }
  return out;
}

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

// ---------- versions ----------

bool GameVersion::Parse(const std::string& text, GameVersion* out) {
  std::vector<std::string> parts;
  std::string current;
  for (char c : text) {
    if (c == '.') {
      parts.push_back(current);
      current.clear();
    } else {
      current.push_back(c);
    }
  }
  parts.push_back(current);
  if (parts.size() < 2 || parts.size() > 3) return false;

  uint16_t numbers[3] = {0, 0, 0};
  for (size_t i = 0; i < parts.size(); ++i) {
    if (!ReadNumber(parts[i], &numbers[i])) return false;
  }
  out->major = numbers[0];
  out->minor = numbers[1];
  out->patch = numbers[2];
  return true;
}

std::string GameVersion::ToString() const {
  std::string text = std::to_string(major) + "." + std::to_string(minor);
  if (patch != 0) text += "." + std::to_string(patch);
  return text;
}

bool GameVersion::operator<(const GameVersion& other) const {
  if (major != other.major) return major < other.major;
  if (minor != other.minor) return minor < other.minor;
  return patch < other.patch;
}

bool GameVersion::operator==(const GameVersion& other) const {
  return major == other.major && minor == other.minor && patch == other.patch;
}

bool GameVersion::AtLeast(uint16_t wantedMajor, uint16_t wantedMinor) const {
  if (major != wantedMajor) return major > wantedMajor;
  return minor >= wantedMinor;
}

// ---------- loaders ----------

const std::vector<Loader>& AllLoaders() {
  static const std::vector<Loader> kLoaders = {Loader::kVanilla, Loader::kForge, Loader::kFabric,
                                               Loader::kQuilt};
  return kLoaders;
}

const char* LoaderId(Loader loader) {
  switch (loader) {
    case Loader::kVanilla:
      return "vanilla";
    case Loader::kForge:
      return "forge";
    case Loader::kFabric:
      return "fabric";
    case Loader::kQuilt:
      return "quilt";
  }
  return "vanilla";
}

const char* LoaderLabel(Loader loader) {
  switch (loader) {
    case Loader::kVanilla:
      return "Vanilla";
    case Loader::kForge:
      return "Forge";
    case Loader::kFabric:
      return "Fabric";
    case Loader::kQuilt:
      return "Quilt";
  }
  return "Vanilla";
}

bool ParseLoader(const std::string& text, Loader* out) {
  std::string lower;
  for (char c : text) lower.push_back(static_cast<char>(std::tolower(static_cast<unsigned char>(c))));
  if (lower == "vanilla" || lower == "none" || lower == "release") {
    *out = Loader::kVanilla;
    return true;
  }
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

// The floors are what each loader's own index offers, not a guess: Fabric's
// oldest stable game version is 1.14 (asking its meta for 1.13.2 answers 400).
GameVersion LoaderEarliest(Loader loader) {
  switch (loader) {
    case Loader::kVanilla:
      return GameVersion{1, 8, 0};
    case Loader::kForge:
      return GameVersion{1, 1, 0};
    case Loader::kFabric:
      return GameVersion{1, 14, 0};
    case Loader::kQuilt:
      return GameVersion{1, 14, 0};
  }
  return GameVersion{1, 8, 0};
}

const char* LoaderMainClass(Loader loader) {
  switch (loader) {
    case Loader::kVanilla:
      return "net.minecraft.client.main.Main";
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
    case Loader::kVanilla:
      return "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    case Loader::kForge:
      return "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
    case Loader::kFabric:
      return "https://meta.fabricmc.net/v2/versions/game";
    case Loader::kQuilt:
      return "https://meta.quiltmc.org/v3/versions/game";
  }
  return "";
}

// ---------- a version and a loader together ----------

LoaderError LoaderTarget::Make(Loader loader, const std::string& versionText,
                               LoaderTarget* out) {
  GameVersion parsed;
  if (!GameVersion::Parse(versionText, &parsed)) return LoaderError::kUnreadableVersion;
  const GameVersion earliest = LoaderEarliest(loader);
  if (parsed < earliest) return LoaderError::kBeforeTheLoader;
  out->loader = loader;
  out->version = parsed;
  return LoaderError::kNone;
}

std::string LoaderTarget::Message(LoaderError error) const {
  switch (error) {
    case LoaderError::kNone:
      return std::string();
    case LoaderError::kUnreadableVersion:
      return "That is not a release version. Snapshots and pre-releases have no profile.";
    case LoaderError::kBeforeTheLoader: {
      const GameVersion earliest = LoaderEarliest(loader);
      return std::string(LoaderLabel(loader)) + " has nothing before " + earliest.ToString() +
             ". Choose a newer version, or Vanilla.";
    }
  }
  return std::string();
}

std::string LoaderTarget::ProfileUrl() const {
  const std::string tag = version.ToString();
  switch (loader) {
    case Loader::kVanilla:
      return "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    case Loader::kForge:
      return "https://maven.minecraftforge.net/net/minecraftforge/forge/" + tag + "/forge-" + tag +
             "-installer.jar";
    case Loader::kFabric:
      return "https://meta.fabricmc.net/v2/versions/loader/" + tag + "/0.16.9";
    case Loader::kQuilt:
      return "https://meta.quiltmc.org/v3/versions/loader/" + tag + "/0.27.0";
  }
  return "";
}

std::string LoaderTarget::Label() const {
  const std::string tag = version.ToString();
  if (loader == Loader::kVanilla) return tag;
  return std::string(LoaderLabel(loader)) + " " + tag;
}

std::vector<Loader> LoadersFor(const std::string& versionText) {
  std::vector<Loader> loaders;
  GameVersion parsed;
  if (!GameVersion::Parse(versionText, &parsed)) return loaders;
  for (Loader loader : AllLoaders()) {
    const GameVersion earliest = LoaderEarliest(loader);
    if (parsed.AtLeast(earliest.major, earliest.minor)) loaders.push_back(loader);
  }
  return loaders;
}

// ---------- memory ----------

uint32_t PlanHeapMiB(uint32_t wanted, uint32_t systemMiB) {
  const uint32_t ceiling = systemMiB / 2;
  const uint32_t floorMiB = 1024;
  uint32_t plan = wanted < floorMiB ? floorMiB : wanted;
  if (plan > ceiling) plan = ceiling;
  return plan < floorMiB ? floorMiB : plan;
}

std::vector<std::string> JvmFlags(uint32_t heapMiB) {
  std::vector<std::string> flags = {
      "-XX:+UseG1GC",
      "-XX:+ParallelRefProcEnabled",
      "-XX:MaxGCPauseMillis=200",
      "-XX:+UnlockExperimentalVMOptions",
      "-XX:+DisableExplicitGC",
      "-XX:G1NewSizePercent=30",
      "-XX:G1MaxNewSizePercent=40",
      "-XX:G1ReservePercent=20",
      "-XX:G1HeapWastePercent=5",
      "-XX:G1MixedGCCountTarget=4",
      "-XX:InitiatingHeapOccupancyPercent=15",
      "-XX:G1MixedGCLiveThresholdPercent=90",
      "-XX:G1RSetUpdatingPauseTimePercent=5",
      "-XX:SurvivorRatio=32",
      "-XX:MaxTenuringThreshold=1",
      "-XX:+PerfDisableSharedMem",
      "-Dfile.encoding=UTF-8",
  };
  flags.push_back(heapMiB >= 12 * 1024 ? "-XX:G1HeapRegionSize=16M" : "-XX:G1HeapRegionSize=8M");
  // AlwaysPreTouch doubles start-up time on a client and touches RAM the player
  // may need for the browser. Only worth it on big heaps.
  if (heapMiB >= 6 * 1024) flags.push_back("-XX:+AlwaysPreTouch");
  flags.push_back("-Daikars.new.flags=true");
  flags.push_back("-Dusing.aikars.flags=https://mcflags.emc.gs");
  return flags;
}

// ---------- accounts ----------

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

std::string Account::Key() const {
  return std::string(ProviderId(provider)) + ":" + uuid;
}

std::string OfflineUuid(const std::string& name) {
  if (name.empty()) return std::string();
  std::array<uint8_t, 16> digest = Md5("OfflinePlayer:" + name);
  digest[6] = static_cast<uint8_t>((digest[6] & 0x0f) | 0x30);  // version 3
  digest[8] = static_cast<uint8_t>((digest[8] & 0x3f) | 0x80);  // RFC 4122 variant
  const std::string hex = Hex(digest);
  return hex.substr(0, 8) + "-" + hex.substr(8, 4) + "-" + hex.substr(12, 4) + "-" +
         hex.substr(16, 4) + "-" + hex.substr(20, 12);
}

bool ValidPlayerName(const std::string& name) {
  if (name.size() < 3 || name.size() > 16) return false;
  for (char c : name) {
    const bool letter = (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
    const bool digit = c >= '0' && c <= '9';
    if (!letter && !digit && c != '_') return false;
  }
  return true;
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

  if (provider == Provider::kMicrosoft) {
    request.authorize_url =
        "https://login.live.com/oauth20_authorize.srf"
        "?client_id=00000000402b5328"
        "&response_type=code&scope=XboxLive.signin%20offline_access&prompt=select_account"
        "&redirect_uri=" +
        redirect_uri + "&state=" + request.state + "&code_challenge=" + challenge +
        "&code_challenge_method=S256";
  } else if (provider == Provider::kElyBy) {
    request.authorize_url =
        "https://account.ely.by/oauth2/v1"
        "?client_id=alpinoxi-client&response_type=code&scope=account_info%20minecraft_server_session"
        "&redirect_uri=" +
        redirect_uri + "&state=" + request.state + "&code_challenge=" + challenge +
        "&code_challenge_method=S256";
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

  // Count the objects directly inside the array. SkipString returns the
  // position after the closing quote, so the loop must not step past it.
  int depth = 0;
  int count = 0;
  for (size_t i = pos; i < json.size(); ++i) {
    const char c = json[i];
    if (c == '"') {
      const size_t after = SkipString(json, i);
      if (after == std::string::npos) return count;
      i = after - 1;
      continue;
    }
    if (c == '{' || c == '[') {
      ++depth;
      if (c == '{' && depth == 2) ++count;
      continue;
    }
    if (c == '}' || c == ']') {
      --depth;
      if (depth == 0) return count;
      continue;
    }
  }
  return count;
}

std::string EscapeForJson(const std::string& text) {
  std::string out;
  out.reserve(text.size() + 8);
  for (char c : text) {
    if (c == '\\' || c == '"') out.push_back('\\');
    out.push_back(c);
  }
  return out;
}

// ---------- friends ----------

const char* PresenceId(Presence presence) {
  switch (presence) {
    case Presence::kOffline:
      return "offline";
    case Presence::kLauncher:
      return "launcher";
    case Presence::kInGame:
      return "in_game";
  }
  return "offline";
}

bool ParsePresence(const std::string& text, Presence* out) {
  if (text == "offline") {
    *out = Presence::kOffline;
    return true;
  }
  if (text == "launcher") {
    *out = Presence::kLauncher;
    return true;
  }
  if (text == "in_game") {
    *out = Presence::kInGame;
    return true;
  }
  return false;
}

std::string Friend::Key() const {
  return std::string(ProviderId(provider)) + ":" + uuid;
}

bool Friend::Joinable() const {
  return presence == Presence::kInGame && !world_id.empty();
}

void FriendBook::Replace(std::vector<Friend> friends) {
  friends_ = std::move(friends);
}

void FriendBook::SetPresence(const std::string& key, Presence presence,
                             const std::string& world_id) {
  for (Friend& entry : friends_) {
    if (entry.Key() == key) {
      entry.presence = presence;
      entry.world_id = world_id;
      return;
    }
  }
  // A presence for someone not in the book still adds them: the book may be a
  // refresh behind, and dropping the news loses a friend who is right there.
  const size_t colon = key.find(':');
  if (colon == std::string::npos) return;
  Friend fresh;
  ParseProvider(key.substr(0, colon), &fresh.provider);
  fresh.uuid = key.substr(colon + 1);
  fresh.name = fresh.uuid;
  fresh.presence = presence;
  fresh.world_id = world_id;
  friends_.push_back(fresh);
}

const std::vector<Friend>& FriendBook::All() const {
  return friends_;
}

std::vector<Friend> FriendBook::Joinable() const {
  std::vector<Friend> list;
  for (const Friend& entry : friends_) {
    if (entry.Joinable()) list.push_back(entry);
  }
  return list;
}

int FriendBook::OnlineCount() const {
  int online = 0;
  for (const Friend& entry : friends_) {
    if (entry.presence != Presence::kOffline) ++online;
  }
  return online;
}

std::string FriendBook::OnlineLabel() const {
  const int online = OnlineCount();
  return online == 0 ? std::string() : "friends " + std::to_string(online);
}

void FriendBook::Remove(const std::string& key) {
  for (auto it = friends_.begin(); it != friends_.end(); ++it) {
    if (it->Key() == key) {
      friends_.erase(it);
      return;
    }
  }
}

const Friend* FriendBook::Find(const std::string& key) const {
  for (const Friend& entry : friends_) {
    if (entry.Key() == key) return &entry;
  }
  return nullptr;
}

// ---------- hosted worlds ----------

const char* AccessId(Access access) {
  switch (access) {
    case Access::kClosed:
      return "closed";
    case Access::kFriends:
      return "friends";
    case Access::kCode:
      return "code";
  }
  return "closed";
}

bool ParseAccess(const std::string& text, Access* out) {
  if (text == "closed") {
    *out = Access::kClosed;
    return true;
  }
  if (text == "friends") {
    *out = Access::kFriends;
    return true;
  }
  if (text == "code") {
    *out = Access::kCode;
    return true;
  }
  return false;
}

bool World::Full() const {
  return players >= max_players;
}

std::string World::Code() const {
  return WorldCode(id);
}

const char* JoinMessage(JoinResult result) {
  switch (result) {
    case JoinResult::kOk:
      return "Joining...";
    case JoinResult::kClosed:
      return "That world is not open.";
    case JoinResult::kFull:
      return "That world is full.";
    case JoinResult::kNotInvited:
      return "You need an invite for that world.";
    case JoinResult::kNoSuchWorld:
      return "That world is no longer open.";
    case JoinResult::kAlreadyInside:
      return "You are already in that world.";
  }
  return "";
}

std::string WorldCode(const std::string& world_id) {
  if (world_id.empty()) return std::string();
  // Letters from a set with no look-alikes: no O/0, no I/1.
  static const char* kLetters = "ABCDEFGHJKLMNPQRSTUVWXYZ";
  const std::array<uint8_t, 16> digest = Md5(world_id);
  uint64_t bits = 0;
  for (int i = 0; i < 8; ++i) bits = (bits << 8) | digest[i];

  std::string code;
  code.reserve(8);
  for (int i = 0; i < 4; ++i) code.push_back(kLetters[(bits >> (i * 5)) % 24]);
  for (int i = 4; i < 8; ++i) code.push_back(static_cast<char>('0' + (bits >> (i * 5)) % 10));
  return code;
}

void WorldBook::SetMaxPlayers(int players) {
  max_players_ = players < 2 ? 2 : (players > 16 ? 16 : players);
}

void WorldBook::Open(const std::string& id, const std::string& name,
                     const std::string& owner_key, Access access) {
  for (World& world : worlds_) {
    if (world.id == id) {
      world.name = name.empty() ? "World" : name;
      world.owner_key = owner_key;
      world.access = access;
      world.max_players = max_players_;
      return;
    }
  }
  World world;
  world.id = id;
  world.name = name.empty() ? "World" : name;
  world.owner_key = owner_key;
  world.access = access;
  world.players = 1;
  world.max_players = max_players_;
  worlds_.push_back(world);
}

void WorldBook::Close(const std::string& id) {
  for (auto it = worlds_.begin(); it != worlds_.end(); ++it) {
    if (it->id == id) {
      worlds_.erase(it);
      break;
    }
  }
  invites_.erase(std::remove_if(invites_.begin(), invites_.end(),
                                [&id](const std::pair<std::string, std::string>& invite) {
                                  return invite.first == id;
                                }),
                 invites_.end());
  if (current_ == id) current_.clear();
}

bool WorldBook::Invite(const std::string& id, const std::string& owner_key,
                       const std::string& guest_key) {
  const World* world = Find(id);
  if (world == nullptr || world->owner_key != owner_key || guest_key.empty()) return false;
  invites_.emplace_back(id, guest_key);
  return true;
}

const World* WorldBook::Find(const std::string& id_or_code) const {
  for (const World& world : worlds_) {
    if (world.id == id_or_code) return &world;
  }
  // A code is matched without its case: a player typing it is not precise.
  std::string upper = id_or_code;
  for (char& c : upper) c = static_cast<char>(std::toupper(static_cast<unsigned char>(c)));
  for (const World& world : worlds_) {
    if (world.Code() == upper) return &world;
  }
  return nullptr;
}

JoinResult WorldBook::MayJoin(const std::string& id_or_code, const std::string& guest_key,
                              bool is_friend, const std::string& code) const {
  const World* world = Find(id_or_code);
  if (world == nullptr) return JoinResult::kNoSuchWorld;
  if (world->id == current_) return JoinResult::kAlreadyInside;
  if (world->access == Access::kClosed) return JoinResult::kClosed;
  if (world->Full()) return JoinResult::kFull;
  if (world->access == Access::kFriends) {
    return is_friend ? JoinResult::kOk : JoinResult::kNotInvited;
  }

  std::string typed = code;
  for (char& c : typed) c = static_cast<char>(std::toupper(static_cast<unsigned char>(c)));
  if (!typed.empty() && typed == world->Code()) return JoinResult::kOk;
  for (const std::pair<std::string, std::string>& invite : invites_) {
    if (invite.first == world->id && invite.second == guest_key) return JoinResult::kOk;
  }
  return JoinResult::kNotInvited;
}

void WorldBook::Enter(const std::string& id) {
  current_ = id;
  for (World& world : worlds_) {
    if (world.id == id) {
      ++world.players;
      return;
    }
  }
}

void WorldBook::Leave() {
  for (World& world : worlds_) {
    if (world.id == current_ && world.players > 0) {
      --world.players;
      break;
    }
  }
  current_.clear();
}

const std::string& WorldBook::Current() const {
  return current_;
}

const std::vector<World>& WorldBook::All() const {
  return worlds_;
}

// ---------- keys ----------

const std::vector<Action>& AllActions() {
  static const std::vector<Action> kActions = {
      Action::kOpenFriends,   Action::kOpenHudEditor, Action::kCloseHud, Action::kToggleHud,
      Action::kOpenChat,      Action::kOpenSkin,      Action::kOpenSettings,
  };
  return kActions;
}

const char* ActionId(Action action) {
  switch (action) {
    case Action::kOpenFriends:
      return "open_friends";
    case Action::kOpenHudEditor:
      return "open_hud_editor";
    case Action::kCloseHud:
      return "close_hud";
    case Action::kToggleHud:
      return "toggle_hud";
    case Action::kOpenChat:
      return "open_chat";
    case Action::kOpenSkin:
      return "open_skin";
    case Action::kOpenSettings:
      return "open_settings";
  }
  return "";
}

const char* ActionLabel(Action action) {
  switch (action) {
    case Action::kOpenFriends:
      return "Friends and worlds";
    case Action::kOpenHudEditor:
      return "HUD editor";
    case Action::kCloseHud:
      return "Close the HUD";
    case Action::kToggleHud:
      return "Show or hide the HUD";
    case Action::kOpenChat:
      return "Chat";
    case Action::kOpenSkin:
      return "Skin";
    case Action::kOpenSettings:
      return "Settings";
  }
  return "";
}

bool ParseAction(const std::string& text, Action* out) {
  for (Action action : AllActions()) {
    if (text == ActionId(action)) {
      *out = action;
      return true;
    }
  }
  return false;
}

bool KeyCombo::operator==(const KeyCombo& other) const {
  return vk == other.vk && shift == other.shift && ctrl == other.ctrl && alt == other.alt;
}

std::string KeyCombo::ToString() const {
  std::string text;
  if (ctrl) text += "Ctrl+";
  if (alt) text += "Alt+";
  if (shift) text += "Shift+";

  switch (vk) {
    case VK_TAB:
      return text + "Tab";
    case VK_ESCAPE:
      return text + "Esc";
    case VK_F1:
      return text + "F1";
    case VK_SPACE:
      return text + "Space";
    case VK_RETURN:
      return text + "Enter";
    default:
      break;
  }
  if (vk >= 'A' && vk <= 'Z') {
    text.push_back(static_cast<char>(vk));
    return text;
  }
  if (vk >= '0' && vk <= '9') {
    text.push_back(static_cast<char>(vk));
    return text;
  }
  return text + "Key " + std::to_string(vk);
}

std::vector<Binding> DefaultBindings() {
  return {
      // Shift+Tab opens friends and worlds. Tab alone is the game's own player
      // list, so the shift is what keeps the two apart.
      Binding{Action::kOpenFriends, KeyCombo{VK_TAB, true, false, false}},
      // Shift+H opens the HUD editor: F1 is the game's debug screen.
      Binding{Action::kOpenHudEditor, KeyCombo{'H', true, false, false}},
      // Esc closes the HUD. Inside the editor it is handled by the editor
      // first, so a player can always get back out.
      Binding{Action::kCloseHud, KeyCombo{VK_ESCAPE, false, false, false}},
      Binding{Action::kToggleHud, KeyCombo{'H', false, true, false}},
      Binding{Action::kOpenChat, KeyCombo{'Y', false, false, false}},
      Binding{Action::kOpenSkin, KeyCombo{'U', false, false, false}},
      Binding{Action::kOpenSettings, KeyCombo{'O', false, true, false}},
  };
}

bool BindingConflict(const std::vector<Binding>& bindings, const KeyCombo& combo, Action except,
                     Action* holder) {
  for (const Binding& binding : bindings) {
    if (binding.action == except) continue;
    if (binding.combo == combo) {
      if (holder != nullptr) *holder = binding.action;
      return true;
    }
  }
  return false;
}

bool ResolveBinding(const std::vector<Binding>& bindings, const KeyCombo& combo, Action* out) {
  for (const Binding& binding : bindings) {
    if (binding.combo == combo) {
      *out = binding.action;
      return true;
    }
  }
  return false;
}

// ---------- HUD ----------

const char* AnchorId(Anchor anchor) {
  switch (anchor) {
    case Anchor::kTopLeft:
      return "top_left";
    case Anchor::kTopCenter:
      return "top_center";
    case Anchor::kTopRight:
      return "top_right";
    case Anchor::kBottomLeft:
      return "bottom_left";
    case Anchor::kBottomCenter:
      return "bottom_center";
    case Anchor::kBottomRight:
      return "bottom_right";
  }
  return "top_left";
}

bool ParseAnchor(const std::string& text, Anchor* out) {
  for (Anchor anchor : {Anchor::kTopLeft, Anchor::kTopCenter, Anchor::kTopRight,
                        Anchor::kBottomLeft, Anchor::kBottomCenter, Anchor::kBottomRight}) {
    if (text == AnchorId(anchor)) {
      *out = anchor;
      return true;
    }
  }
  return false;
}

std::vector<HudElement> DefaultHud() {
  return {
      HudElement{"fps", "FPS", Anchor::kTopLeft, true, 0, 0},
      HudElement{"coords", "Coordinates", Anchor::kTopLeft, true, 0, 22},
      HudElement{"ping", "Ping", Anchor::kTopLeft, true, 0, 44},
      HudElement{"memory", "Memory", Anchor::kTopRight, true, 0, 0},
      HudElement{"clock", "Clock", Anchor::kTopRight, true, 0, 22},
      HudElement{"keystrokes", "Keystrokes", Anchor::kBottomLeft, true, 0, 0},
  };
}

bool MoveElement(std::vector<HudElement>* layout, const std::string& id, Anchor anchor) {
  for (HudElement& element : *layout) {
    if (element.id == id) {
      element.anchor = anchor;
      return true;
    }
  }
  return false;
}

int ToggleElement(std::vector<HudElement>* layout, const std::string& id) {
  for (HudElement& element : *layout) {
    if (element.id == id) {
      element.visible = !element.visible;
      return element.visible ? 1 : 0;
    }
  }
  return -1;
}

// ---------- settings ----------

std::string SettingsToJson(const Settings& settings) {
  std::string json = "{\n";
  json += "  \"provider\": \"" + std::string(ProviderId(settings.provider)) + "\",\n";
  json += "  \"playerName\": \"" + EscapeForJson(settings.player_name) + "\",\n";
  json += "  \"loader\": \"" + std::string(LoaderId(settings.loader)) + "\",\n";
  json += "  \"version\": \"" + EscapeForJson(settings.version) + "\",\n";
  json += "  \"heapMiB\": " + std::to_string(settings.heap_mib) + ",\n";
  json += std::string("  \"hudEnabled\": ") + (settings.hud_enabled ? "true" : "false") + ",\n";
  json += "  \"javaPath\": \"" + EscapeForJson(settings.java_path) + "\",\n";

  json += "  \"bindings\": [\n";
  for (size_t i = 0; i < settings.bindings.size(); ++i) {
    const Binding& binding = settings.bindings[i];
    json += "    {\"action\": \"" + std::string(ActionId(binding.action)) + "\", \"vk\": " +
            std::to_string(binding.combo.vk) + ", \"shift\": " +
            (binding.combo.shift ? "true" : "false") + ", \"ctrl\": " +
            (binding.combo.ctrl ? "true" : "false") + ", \"alt\": " +
            (binding.combo.alt ? "true" : "false") + "}";
    json += (i + 1 < settings.bindings.size()) ? ",\n" : "\n";
  }
  json += "  ],\n";

  json += "  \"hud\": [\n";
  for (size_t i = 0; i < settings.hud.size(); ++i) {
    const HudElement& element = settings.hud[i];
    json += "    {\"id\": \"" + EscapeForJson(element.id) + "\", \"label\": \"" +
            EscapeForJson(element.label) + "\", \"anchor\": \"" + AnchorId(element.anchor) +
            "\", \"visible\": " + (element.visible ? "true" : "false") +
            ", \"offsetX\": " + std::to_string(element.offset_x) +
            ", \"offsetY\": " + std::to_string(element.offset_y) + "}";
    json += (i + 1 < settings.hud.size()) ? ",\n" : "\n";
  }
  json += "  ]\n}\n";
  return json;
}

Settings SettingsFromJson(const std::string& json) {
  Settings settings;
  std::string text;

  if (JsonString(json, "provider", &text)) ParseProvider(text, &settings.provider);
  if (JsonString(json, "playerName", &text)) settings.player_name = text;
  if (JsonString(json, "loader", &text)) ParseLoader(text, &settings.loader);
  if (JsonString(json, "version", &text)) settings.version = text;
  if (JsonString(json, "javaPath", &text)) settings.java_path = text;

  double number = 0;
  if (JsonNumber(json, "heapMiB", &number) && number >= 512 && number <= 65536) {
    settings.heap_mib = static_cast<uint32_t>(number);
  }
  const size_t hudSwitch = json.find("\"hudEnabled\"");
  if (hudSwitch != std::string::npos) {
    const size_t value = json.find(':', hudSwitch);
    if (value != std::string::npos) settings.hud_enabled = json.compare(value + 1, 4, "true") == 0;
  }

  // The bindings array: one object per action, and an action the file names but
  // this build does not know is simply skipped.
  const size_t bindings = json.find("\"bindings\"");
  if (bindings != std::string::npos) {
    size_t cursor = json.find('[', bindings);
    if (cursor != std::string::npos) {
      std::vector<Binding> parsed;
      while (true) {
        const size_t open = json.find('{', cursor);
        if (open == std::string::npos) break;
        const size_t close = json.find('}', open);
        if (close == std::string::npos) break;
        const std::string object = json.substr(open, close - open + 1);
        cursor = close + 1;

        std::string actionId;
        double vk = 0;
        if (!JsonString(object, "action", &actionId)) continue;
        if (!JsonNumber(object, "vk", &vk)) continue;
        Action action;
        if (!ParseAction(actionId, &action)) continue;

        Binding binding;
        binding.action = action;
        binding.combo.vk = static_cast<int>(vk);
        binding.combo.shift = object.find("\"shift\": true") != std::string::npos;
        binding.combo.ctrl = object.find("\"ctrl\": true") != std::string::npos;
        binding.combo.alt = object.find("\"alt\": true") != std::string::npos;
        parsed.push_back(binding);

        const size_t arrayEnd = json.find(']', close);
        if (arrayEnd != std::string::npos && cursor > arrayEnd) break;
      }
      // A file that lists some actions keeps those, and the rest stay at their
      // defaults: a half-written settings file must not unbind the HUD.
      for (const Binding& fallback : DefaultBindings()) {
        bool present = false;
        for (const Binding& binding : parsed) {
          if (binding.action == fallback.action) present = true;
        }
        if (!present) parsed.push_back(fallback);
      }
      settings.bindings = parsed;
    }
  }

  // The HUD layout: every element the file knows replaces its default.
  const size_t hud = json.find("\"hud\"");
  if (hud != std::string::npos) {
    size_t cursor = json.find('[', hud);
    if (cursor != std::string::npos) {
      std::vector<HudElement> parsed;
      while (true) {
        const size_t open = json.find('{', cursor);
        if (open == std::string::npos) break;
        const size_t close = json.find('}', open);
        if (close == std::string::npos) break;
        const std::string object = json.substr(open, close - open + 1);
        cursor = close + 1;

        HudElement element;
        std::string anchorId;
        if (!JsonString(object, "id", &element.id)) continue;
        JsonString(object, "label", &element.label);
        if (JsonString(object, "anchor", &anchorId)) ParseAnchor(anchorId, &element.anchor);
        element.visible = object.find("\"visible\": true") != std::string::npos;
        double offsetX = 0, offsetY = 0;
        if (JsonNumber(object, "offsetX", &offsetX)) element.offset_x = static_cast<int>(offsetX);
        if (JsonNumber(object, "offsetY", &offsetY)) element.offset_y = static_cast<int>(offsetY);
        parsed.push_back(element);

        const size_t arrayEnd = json.find(']', close);
        if (arrayEnd != std::string::npos && cursor > arrayEnd) break;
      }
      std::vector<HudElement> merged = DefaultHud();
      for (const HudElement& saved : parsed) {
        bool replaced = false;
        for (HudElement& element : merged) {
          if (element.id == saved.id) {
            element = saved;
            replaced = true;
          }
        }
        if (!replaced) merged.push_back(saved);
      }
      settings.hud = merged;
    }
  }
  return settings;
}

// ---------- instances ----------

std::string InstanceId(Loader loader, const GameVersion& version) {
  return std::string(LoaderId(loader)) + "-" + version.ToString();
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

std::string LaunchCommand::command_line() const {
  std::string line = QuoteForWindows(java);
  for (const std::string& argument : arguments) {
    line += " ";
    // The arguments that carry a path are quoted where they are built; the rest
    // cannot contain a space, so quoting them again would show up in game.
    line += argument.find(' ') != std::string::npos && argument.front() != '"'
                ? QuoteForWindows(argument)
                : argument;
  }
  return line;
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
