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

enum class Loader { kVanilla, kForge, kFabric, kQuilt };

const std::vector<Loader>& AllLoaders();
const char* LoaderId(Loader loader);
const char* LoaderLabel(Loader loader);
bool ParseLoader(const std::string& text, Loader* out);
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
  Loader loader = Loader::kVanilla;
  GameVersion version;

  static LoaderError Make(Loader loader, const std::string& version, LoaderTarget* out);
  std::string Message(LoaderError error) const;
  std::string ProfileUrl() const;
  // What the rail and the instance card show: "Fabric 1.21.1".
  std::string Label() const;
};

std::vector<Loader> LoadersFor(const std::string& version);

// ---------- memory ----------

// Never ask for more than half the machine, and never more than the ceiling.
uint32_t PlanHeapMiB(uint32_t wanted, uint32_t systemMiB);

// The G1 flags a client actually benefits from, scaled by heap size. Aikar's
// flags, trimmed: a client is not a server.
std::vector<std::string> JvmFlags(uint32_t heapMiB);

// ---------- accounts ----------

enum class Provider { kMicrosoft, kElyBy, kOffline };

const std::vector<Provider>& AllProviders();
const char* ProviderId(Provider provider);
const char* ProviderLabel(Provider provider);
bool ParseProvider(const std::string& text, Provider* out);

// A signed-in account, as the launcher keeps it.
struct Account {
  Provider provider = Provider::kOffline;
  std::string name;
  std::string uuid;
  std::string access_token;
  // What the sign-in flow left behind for next time.
  std::string refresh_token;
  int64_t expires_at = 0;  // unix seconds; 0 means "no expiry"

  // "provider:uuid": the identity friends and worlds are keyed by.
  std::string Key() const;
};

// The offline UUID Minecraft itself computes: an MD5 of "OfflinePlayer:<name>"
// with the version and variant bits set. Cracked mode must match exactly or the
// server puts the player in a different slot than the skin expects.
std::string OfflineUuid(const std::string& name);

// A player name that can be used offline: 3–16 characters, letters, digits and
// underscore only.
bool ValidPlayerName(const std::string& name);

// Builds the offline account for a name, or an empty account when the name is
// not usable.
Account MakeOfflineAccount(const std::string& name);

// ---------- sign-in URLs ----------

// PKCE: a verifier the app keeps, and the challenge the server sees.
std::string RandomUrlSafe(size_t bytes);  // base64url, no padding
std::string PkceChallenge(const std::string& verifier);  // base64url(SHA256(verifier))

struct SignInRequest {
  std::string authorize_url;
  std::string verifier;
  std::string state;
};

// The URL to open in the browser, with PKCE and state already in place.
SignInRequest StartSignIn(Provider provider, const std::string& redirect_uri);

// ---------- JSON ----------
// The launcher reads a handful of small JSON documents. A full parser is not
// worth its size; these read the fields it needs, and say so when a field is
// missing instead of inventing one.

bool JsonString(const std::string& json, const std::string& key, std::string* out);
bool JsonNumber(const std::string& json, const std::string& key, double* out);
int JsonArrayCount(const std::string& json, const std::string& key);

// ---------- friends ----------

enum class Presence { kOffline, kLauncher, kInGame };

const char* PresenceId(Presence presence);
bool ParsePresence(const std::string& text, Presence* out);

struct Friend {
  Provider provider = Provider::kOffline;
  std::string uuid;
  std::string name;
  Presence presence = Presence::kOffline;
  // The world they are in, when they are hosting one that can be joined.
  std::string world_id;

  std::string Key() const;
  // True only when there is somewhere to go: in game, in a shared world.
  bool Joinable() const;
};

// The friend book. Presence is pushed in by whoever learns it; nothing here
// talks to a network.
class FriendBook {
 public:
  void Replace(std::vector<Friend> friends);
  void SetPresence(const std::string& key, Presence presence, const std::string& world_id);
  const std::vector<Friend>& All() const;
  std::vector<Friend> Joinable() const;
  int OnlineCount() const;
  // Empty when nobody is around, so the HUD draws nothing.
  std::string OnlineLabel() const;
  void Remove(const std::string& key);
  const Friend* Find(const std::string& key) const;

 private:
  std::vector<Friend> friends_;
};

// ---------- hosted worlds ----------

enum class Access { kClosed, kFriends, kCode };

const char* AccessId(Access access);
bool ParseAccess(const std::string& text, Access* out);

struct World {
  std::string id;
  std::string name;
  std::string owner_key;
  Access access = Access::kClosed;
  int players = 1;
  int max_players = 8;

  bool Full() const;
  // The shareable code, derived from the id so it never changes.
  std::string Code() const;
};

enum class JoinResult { kOk, kClosed, kFull, kNotInvited, kNoSuchWorld, kAlreadyInside };

const char* JoinMessage(JoinResult result);

// Four letters and four digits from the world id's own bytes. Derived rather
// than random: the same world hands out the same code next week.
std::string WorldCode(const std::string& world_id);

// The worlds that are open right now, and the rules for joining one.
class WorldBook {
 public:
  void Open(const std::string& id, const std::string& name, const std::string& owner_key,
            Access access);
  void Close(const std::string& id);
  bool Invite(const std::string& id, const std::string& owner_key, const std::string& guest_key);
  const World* Find(const std::string& id_or_code) const;
  // Whether one player may join, and why not when they may not.
  JoinResult MayJoin(const std::string& id_or_code, const std::string& guest_key, bool is_friend,
                     const std::string& code) const;
  void Enter(const std::string& id);
  void Leave();
  const std::string& Current() const;
  const std::vector<World>& All() const;
  void SetMaxPlayers(int players);

 private:
  std::vector<World> worlds_;
  std::vector<std::pair<std::string, std::string>> invites_;  // id|guest -> code
  std::string current_;
  int max_players_ = 8;
};

// ---------- settings ----------
// One file, %APPDATA%/alpinoxi/settings.json, and the launcher reads it back
// the same way every time.

struct Settings {
  Provider provider = Provider::kOffline;
  std::string player_name;
  Loader loader = Loader::kVanilla;
  std::string version = "1.21.1";
  uint32_t heap_mib = 4096;
  bool hud_enabled = true;
  std::string java_path;  // empty means "find one"
};

std::string SettingsToJson(const Settings& settings);
// Reads what it can, and leaves every field it cannot read at its default.
Settings SettingsFromJson(const std::string& json);

// ---------- instances ----------

struct Instance {
  std::string id;       // "fabric-1.21.1"
  std::string name;     // "Fabric 1.21.1"
  LoaderTarget target;
  uint32_t heap_mib = 4096;
  bool installed = false;
};

// The id an instance is stored under.
std::string InstanceId(Loader loader, const GameVersion& version);

// ---------- the command line the game is started with ----------

struct LaunchCommand {
  std::string java;                     // path to java.exe
  std::vector<std::string> arguments;   // everything after it
  std::string working_dir;
  // The whole line, quoted, for CreateProcessW.
  std::string command_line() const;
};

LaunchCommand BuildLaunchCommand(const std::string& java, const std::string& game_dir,
                                 const std::string& assets_dir, const std::string& asset_index,
                                 const std::string& classpath, const LoaderTarget& target,
                                 const Account& account, uint32_t heapMiB);

// Quotes one argument for the Windows command line.
std::string QuoteForWindows(const std::string& argument);

}  // namespace alpinoxi
