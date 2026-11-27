// The launcher's decisions, in one header, so they can be tested without a
// window. Everything here is pure: no Win32, no network, no files.

#pragma once

#include <cstdint>
#include <string>
#include <vector>

namespace alpinoxi {

// ---------- versions ----------

// A Minecraft release, as three numbers compared in order.
struct GameVersion {
  uint16_t major = 0;
  uint16_t minor = 0;
  uint16_t patch = 0;

  // Parses "1.8", "1.21.1", "26.3". Snapshots and pre-releases ("24w14a",
  // "1.21-pre1") parse as nothing, because no loader has a profile for them.
  static bool Parse(const std::string& text, GameVersion* out);
  std::string ToString() const;
  bool operator<(const GameVersion& other) const;
  bool operator==(const GameVersion& other) const;
  bool AtLeast(uint16_t major, uint16_t minor) const;
};

// ---------- loaders ----------

enum class Loader { kForge, kFabric, kQuilt };

// Every loader, in the order the interface lists them.
const std::vector<Loader>& AllLoaders();
const char* LoaderId(Loader loader);     // "forge" — settings and the interface
const char* LoaderLabel(Loader loader);  // "Forge" — shown to a player
bool ParseLoader(const std::string& text, Loader* out);

// The oldest Minecraft version each loader ever supported. Facts about the
// loaders, checked against their own metadata: Fabric's index lists 266 stable
// versions and the oldest is 1.14; Quilt forked from Fabric and began at 1.14.4;
// Forge reaches back to 1.1.
GameVersion LoaderEarliest(Loader loader);
const char* LoaderMainClass(Loader loader);
const char* LoaderVersionsUrl(Loader loader);

// ---------- a version and a loader together ----------

enum class LoaderError {
  kNone,
  kUnreadableVersion,
  kBeforeTheLoader,
};

struct LoaderTarget {
  Loader loader = Loader::kForge;
  GameVersion version;

  // Refuses a pair that never existed.
  static LoaderError Make(Loader loader, const std::string& version, LoaderTarget* out);
  // One line a player can read, without jargon.
  std::string Message(LoaderError error) const;
  // The JSON profile describing this loader's libraries and main class.
  std::string ProfileUrl() const;
};

// Every loader that can run this version; empty for a version nothing supports.
std::vector<Loader> LoadersFor(const std::string& version);

// ---------- memory ----------

// Never ask for more than half the machine, and never more than the ceiling.
uint32_t PlanHeapMiB(uint32_t wanted, uint32_t systemMiB);

// ---------- accounts ----------

// The offline UUID Minecraft itself computes: an MD5 of "OfflinePlayer:<name>"
// with the version and variant bits set. Cracked mode must match exactly or the
// server puts the player in a different slot than the skin expects.
std::string OfflineUuid(const std::string& name);

// A player name that can be used offline: 3–16 characters, letters, digits and
// underscore only.
bool ValidPlayerName(const std::string& name);

}  // namespace alpinoxi
