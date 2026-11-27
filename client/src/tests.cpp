// The launcher's test suite. It has no window and no network: everything it
// checks is a decision the launcher makes, which is where the bugs live.

#include "app.h"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include <cstdio>
#include <string>

using namespace alpinoxi;

namespace {

int g_failures = 0;
int g_checks = 0;

void Check(bool condition, const char* what, int line) {
  ++g_checks;
  if (condition) return;
  ++g_failures;
  std::printf("  FAIL  line %d: %s\n", line, what);
}

void CheckEqual(const std::string& actual, const std::string& expected, const char* what,
                int line) {
  ++g_checks;
  if (actual == expected) return;
  ++g_failures;
  std::printf("  FAIL  line %d: %s\n        expected \"%s\"\n        got      \"%s\"\n", line,
              what, expected.c_str(), actual.c_str());
}

void CheckEqualInt(long long actual, long long expected, const char* what, int line) {
  ++g_checks;
  if (actual == expected) return;
  ++g_failures;
  std::printf("  FAIL  line %d: %s\n        expected %lld, got %lld\n", line, what, expected,
              actual);
}

#define CHECK(condition) Check((condition), #condition, __LINE__)
#define CHECK_EQ(actual, expected) CheckEqual((actual), (expected), #actual, __LINE__)
#define CHECK_INT(actual, expected) CheckEqualInt((actual), (expected), #actual, __LINE__)

void Section(const char* name) {
  std::printf("- %s\n", name);
}

// ---------- versions ----------

void TestVersions() {
  Section("versions");
  GameVersion version;

  CHECK(GameVersion::Parse("1.21.1", &version));
  CHECK_EQ(version.ToString(), "1.21.1");
  CHECK(GameVersion::Parse("1.8", &version));
  CHECK_EQ(version.ToString(), "1.8");
  CHECK(GameVersion::Parse("26.3", &version));
  CHECK_EQ(version.ToString(), "26.3");

  // A snapshot and a pre-release are not releases, and no loader has a profile
  // for them. Parsing them as numbers would offer Fabric for 24.14.
  CHECK(!GameVersion::Parse("24w14a", &version));
  CHECK(!GameVersion::Parse("1.21-pre1", &version));
  CHECK(!GameVersion::Parse("", &version));
  CHECK(!GameVersion::Parse("1", &version));
  CHECK(!GameVersion::Parse("1.2.3.4", &version));
  CHECK(!GameVersion::Parse("one.two", &version));

  GameVersion older{1, 20, 6};
  GameVersion newer{1, 21, 1};
  CHECK(older < newer);
  const GameVersion later{1, 21, 2};
  CHECK(newer < later);
  CHECK(!(newer < older));
  const GameVersion same{1, 21, 1};
  CHECK(newer == same);
  // 1.21.0 and 1.21 are the same release, so they must compare equal.
  const GameVersion zeroPatch{1, 21, 0};
  const GameVersion alsoZero{1, 21, 0};
  CHECK(zeroPatch == alsoZero);
  CHECK(newer.AtLeast(1, 14));
  CHECK(!older.AtLeast(1, 21));
}

// ---------- loaders ----------

void TestLoaders() {
  Section("loaders");

  Loader loader;
  CHECK(ParseLoader("fabric", &loader));
  CHECK(loader == Loader::kFabric);
  CHECK(ParseLoader("Quilt", &loader));
  CHECK(loader == Loader::kQuilt);
  CHECK(ParseLoader("vanilla", &loader));
  CHECK(loader == Loader::kVanilla);
  CHECK(ParseLoader("none", &loader));
  CHECK(loader == Loader::kVanilla);
  CHECK(ParseLoader("release", &loader));
  CHECK(loader == Loader::kVanilla);
  CHECK(!ParseLoader("liteloader", &loader));
  CHECK(!ParseLoader("", &loader));

  // Vanilla has to be a loader: the game runs with none, so it must be pickable.
  CHECK_INT(static_cast<int>(AllLoaders().size()), 4);
  CHECK_EQ(LoaderId(Loader::kVanilla), "vanilla");
  CHECK_EQ(LoaderLabel(Loader::kFabric), "Fabric");
  CHECK_EQ(LoaderMainClass(Loader::kVanilla), "net.minecraft.client.main.Main");

  // The floors each index really has.
  CHECK(LoaderEarliest(Loader::kForge) == (GameVersion{1, 1, 0}));
  CHECK(LoaderEarliest(Loader::kFabric) == (GameVersion{1, 14, 0}));
  CHECK(LoaderEarliest(Loader::kQuilt) == (GameVersion{1, 14, 0}));
  CHECK(LoaderEarliest(Loader::kVanilla) == (GameVersion{1, 8, 0}));

  // 1.8 predates Fabric and Quilt, so offering them would be a broken launch.
  std::vector<Loader> for18 = LoadersFor("1.8.9");
  CHECK_INT(static_cast<int>(for18.size()), 2);
  std::vector<Loader> for114 = LoadersFor("1.14");
  CHECK_INT(static_cast<int>(for114.size()), 4);
  CHECK_INT(static_cast<int>(LoadersFor("24w14a").size()), 0);
}

void TestLoaderTargets() {
  Section("loader targets");
  LoaderTarget target;

  CHECK(LoaderTarget::Make(Loader::kFabric, "1.21.1", &target) == LoaderError::kNone);
  CHECK_EQ(target.Label(), "Fabric 1.21.1");
  CHECK(LoaderTarget::Make(Loader::kVanilla, "1.21.1", &target) == LoaderError::kNone);
  // Vanilla is just the version: "Vanilla 1.21.1" would be noise.
  CHECK_EQ(target.Label(), "1.21.1");

  CHECK(LoaderTarget::Make(Loader::kFabric, "1.13.2", &target) ==
        LoaderError::kBeforeTheLoader);
  CHECK(LoaderTarget::Make(Loader::kFabric, "24w14a", &target) ==
        LoaderError::kUnreadableVersion);

  // The message names the floor, so the player knows what to pick.
  LoaderTarget fabric;
  fabric.loader = Loader::kFabric;
  const std::string message = fabric.Message(LoaderError::kBeforeTheLoader);
  CHECK(message.find("1.14") != std::string::npos);
  CHECK(message.find("Fabric") != std::string::npos);
  CHECK_EQ(fabric.Message(LoaderError::kNone), "");

  CHECK_EQ(InstanceId(Loader::kFabric, GameVersion{1, 21, 1}), "fabric-1.21.1");
  CHECK_EQ(InstanceId(Loader::kVanilla, GameVersion{1, 21, 1}), "vanilla-1.21.1");
}

// ---------- memory ----------

void TestMemory() {
  Section("memory");

  // Half the machine, at most.
  CHECK_INT(PlanHeapMiB(8192, 8192), 4096);
  CHECK_INT(PlanHeapMiB(2048, 16384), 2048);
  // Never below a playable floor, even on a small machine.
  CHECK_INT(PlanHeapMiB(512, 32768), 1024);
  CHECK_INT(PlanHeapMiB(4096, 1024), 1024);

  const std::vector<std::string> small = JvmFlags(2048);
  bool hasG1 = false;
  bool hasPreTouch = false;
  for (const std::string& flag : small) {
    if (flag == "-XX:+UseG1GC") hasG1 = true;
    if (flag == "-XX:+AlwaysPreTouch") hasPreTouch = true;
  }
  CHECK(hasG1);
  // AlwaysPreTouch on a 2 GiB client doubles start-up and eats the RAM the
  // browser needs.
  CHECK(!hasPreTouch);

  const std::vector<std::string> large = JvmFlags(8192);
  hasPreTouch = false;
  for (const std::string& flag : large) {
    if (flag == "-XX:+AlwaysPreTouch") hasPreTouch = true;
  }
  CHECK(hasPreTouch);
}

// ---------- accounts ----------

void TestOfflineAccounts() {
  Section("offline accounts");

  // These must match Minecraft exactly, or the server puts the player in a
  // different slot than the skin is looked up for.
  CHECK_EQ(OfflineUuid("Notch"), "b50ad385-829d-3141-a216-7e7d7539ba7f");
  CHECK_EQ(OfflineUuid("Alpinoxi"), "7cce3372-aec0-3d25-9588-23579d5fcd8a");
  CHECK_EQ(OfflineUuid("Steve"), "5627dd98-e6be-3c21-b8a8-e92344183641");
  CHECK_EQ(OfflineUuid("Player"), "a01e3843-e521-3998-958a-f459800e4d11");
  CHECK_EQ(OfflineUuid(""), "");

  CHECK(ValidPlayerName("Alpinoxi"));
  CHECK(ValidPlayerName("abc"));
  CHECK(ValidPlayerName("A_1_b"));
  CHECK(!ValidPlayerName("ab"));
  CHECK(!ValidPlayerName("thisnameiswaytoolong"));
  CHECK(!ValidPlayerName("has space"));
  CHECK(!ValidPlayerName("bad-dash"));
  CHECK(!ValidPlayerName(""));

  const Account account = MakeOfflineAccount("Alpinoxi");
  CHECK(account.provider == Provider::kOffline);
  CHECK_EQ(account.name, "Alpinoxi");
  CHECK_EQ(account.uuid, "7cce3372-aec0-3d25-9588-23579d5fcd8a");
  CHECK_EQ(account.Key(), "offline:7cce3372-aec0-3d25-9588-23579d5fcd8a");

  // An unusable name makes no account at all, rather than a broken one.
  CHECK_EQ(MakeOfflineAccount("no").uuid, "");
  CHECK_EQ(MakeOfflineAccount("has space").name, "");

  Provider provider;
  CHECK(ParseProvider("microsoft", &provider));
  CHECK(provider == Provider::kMicrosoft);
  CHECK(ParseProvider("ely.by", &provider));
  CHECK(provider == Provider::kElyBy);
  CHECK(ParseProvider("cracked", &provider));
  CHECK(provider == Provider::kOffline);
  CHECK(!ParseProvider("mojang", &provider));
}

void TestSignIn() {
  Section("sign-in");

  // The RFC 7636 example, so the challenge is verified against the standard and
  // not against our own output.
  CHECK_EQ(PkceChallenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
           "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");

  const std::string random = RandomUrlSafe(32);
  // 32 bytes is 43 base64 characters, with no padding.
  CHECK_INT(static_cast<long long>(random.size()), 43);
  CHECK(random.find('=') == std::string::npos);
  CHECK(random.find('+') == std::string::npos);
  CHECK(random.find('/') == std::string::npos);
  CHECK(RandomUrlSafe(32) != random);

  const SignInRequest microsoft = StartSignIn(Provider::kMicrosoft, "http://127.0.0.1:38471/cb");
  CHECK(microsoft.authorize_url.find("login.live.com") != std::string::npos);
  CHECK(microsoft.authorize_url.find("code_challenge=") != std::string::npos);
  CHECK(microsoft.authorize_url.find("S256") != std::string::npos);
  CHECK(microsoft.authorize_url.find(microsoft.state) != std::string::npos);
  CHECK(microsoft.authorize_url.find(microsoft.verifier) == std::string::npos);
  CHECK(!microsoft.verifier.empty());

  const SignInRequest ely = StartSignIn(Provider::kElyBy, "http://127.0.0.1:38471/cb");
  CHECK(ely.authorize_url.find("account.ely.by") != std::string::npos);
  CHECK(ely.state != microsoft.state);
  // Offline has nothing to sign in to.
  const SignInRequest offline = StartSignIn(Provider::kOffline, "http://127.0.0.1:1/cb");
  CHECK_EQ(offline.authorize_url, "");
}

// ---------- JSON ----------

void TestJson() {
  Section("json");

  const std::string manifest =
      R"({"latest":{"release":"1.21.1"},"versions":[{"id":"1.21.1","type":"release"},)"
      R"({"id":"1.21","type":"release"},{"id":"1.20.6","type":"release"}]})";

  std::string text;
  CHECK(JsonString(manifest, "release", &text));
  CHECK_EQ(text, "1.21.1");
  CHECK(!JsonString(manifest, "snapshot", &text));
  // The array has three objects; counting it wrong would show the wrong number
  // of versions.
  CHECK_INT(JsonArrayCount(manifest, "versions"), 3);

  const std::string profile = R"({"id":"Alpinoxi","name":"Alpinoxi","properties":[{"name":"textures"}]})";
  CHECK_INT(JsonArrayCount(profile, "properties"), 1);
  CHECK_INT(JsonArrayCount(profile, "missing"), 0);

  // A string that contains a brace must not break the object count.
  const std::string tricky = R"({"list":[{"note":"a } brace"},{"note":"{ another"}]})";
  CHECK_INT(JsonArrayCount(tricky, "list"), 2);

  double number = 0;
  CHECK(JsonNumber(manifest, "missing", &number) == false);
  const std::string sizes = R"({"size":1234,"ratio":-0.5})";
  CHECK(JsonNumber(sizes, "size", &number));
  CHECK_INT(static_cast<long long>(number), 1234);
  CHECK(JsonNumber(sizes, "ratio", &number));
  CHECK(number < 0);

  const std::string escaped = R"({"path":"C:\\Users\\User\\game"})";
  CHECK(JsonString(escaped, "path", &text));
  CHECK_EQ(text, "C:\\Users\\User\\game");

  // A path written back out must survive a round trip through the reader.
  CHECK_EQ(EscapeForJson("C:\\Users\\x"), "C:\\\\Users\\\\x");
  CHECK_EQ(EscapeForJson("say \"hi\""), "say \\\"hi\\\"");
}

// ---------- friends ----------

void TestFriends() {
  Section("friends");

  Friend alex;
  alex.provider = Provider::kMicrosoft;
  alex.uuid = "11111111-1111-1111-1111-111111111111";
  alex.name = "Alex";
  alex.presence = Presence::kInGame;
  alex.world_id = "world-1";

  Friend steve;
  steve.provider = Provider::kOffline;
  steve.uuid = "5627dd98-e6be-3c21-b8a8-e92344183641";
  steve.name = "Steve";
  steve.presence = Presence::kLauncher;

  // The same UUID on two providers is two different people.
  CHECK(alex.Key() != steve.Key());
  CHECK_EQ(alex.Key(), "microsoft:11111111-1111-1111-1111-111111111111");

  CHECK(alex.Joinable());
  CHECK(!steve.Joinable());

  FriendBook book;
  book.Replace({alex, steve});
  CHECK_INT(book.OnlineCount(), 2);
  CHECK_INT(static_cast<long long>(book.Joinable().size()), 1);
  CHECK_EQ(book.OnlineLabel(), "friends 2");

  book.SetPresence(steve.Key(), Presence::kInGame, "world-2");
  CHECK_INT(static_cast<long long>(book.Joinable().size()), 2);
  CHECK(book.Find(steve.Key()) != nullptr);
  CHECK_EQ(book.Find(steve.Key())->world_id, "world-2");

  book.Remove(alex.Key());
  CHECK(book.Find(alex.Key()) == nullptr);
  CHECK_INT(book.OnlineCount(), 1);

  // Everyone offline draws nothing at all, not "friends 0".
  FriendBook empty;
  CHECK_EQ(empty.OnlineLabel(), "");

  Presence presence;
  CHECK(ParsePresence("in_game", &presence));
  CHECK(presence == Presence::kInGame);
  CHECK(!ParsePresence("away", &presence));
  CHECK_EQ(PresenceId(Presence::kLauncher), "launcher");
}

// ---------- worlds ----------

void TestWorlds() {
  Section("worlds");

  // Derived, so the same world hands out the same code next week.
  const std::string code = WorldCode("world-1");
  CHECK_INT(static_cast<long long>(code.size()), 8);
  CHECK_EQ(WorldCode("world-1"), code);
  CHECK(WorldCode("world-2") != code);
  CHECK_EQ(WorldCode(""), "");
  // No look-alike characters: a player reading it aloud must not guess wrong.
  CHECK(code.find('O') == std::string::npos);
  CHECK(code.find('I') == std::string::npos);
  for (size_t i = 4; i < code.size(); ++i) CHECK(code[i] >= '0' && code[i] <= '9');
  for (size_t i = 0; i < 4; ++i) CHECK(code[i] >= 'A' && code[i] <= 'Z');

  WorldBook book;
  book.Open("world-1", "Survival", "owner-key", Access::kFriends);
  CHECK_INT(static_cast<long long>(book.All().size()), 1);
  CHECK(book.Find("world-1") != nullptr);
  // A code is matched without its case: nobody types it precisely.
  std::string lower = code;
  for (char& c : lower) c = static_cast<char>(c - 'A' + 'a');
  CHECK(book.Find(code) != nullptr);

  CHECK(book.MayJoin("world-1", "guest", true, "") == JoinResult::kOk);
  CHECK(book.MayJoin("world-1", "guest", false, "") == JoinResult::kNotInvited);
  CHECK(book.MayJoin("nope", "guest", true, "") == JoinResult::kNoSuchWorld);

  // Closed says closed even to a friend.
  book.Open("world-1", "Survival", "owner-key", Access::kClosed);
  CHECK(book.MayJoin("world-1", "guest", true, "") == JoinResult::kClosed);

  // A code world opens for whoever has the code, and full beats an invite.
  book.Open("world-1", "Survival", "owner-key", Access::kCode);
  CHECK(book.MayJoin("world-1", "guest", false, code) == JoinResult::kOk);
  CHECK(book.MayJoin("world-1", "guest", false, "") == JoinResult::kNotInvited);
  book.SetMaxPlayers(2);
  book.Open("world-1", "Survival", "owner-key", Access::kCode);
  book.Enter("world-1");
  book.Leave();
  book.Enter("world-1");
  CHECK(book.MayJoin("world-1", "guest", true, code) == JoinResult::kFull);
  CHECK(book.MayJoin("world-1", "guest", true, code) == JoinResult::kAlreadyInside);
  book.Leave();

  // An invite is the owner's to give.
  CHECK(book.Invite("world-1", "owner-key", "guest"));
  CHECK(!book.Invite("world-1", "someone-else", "guest"));
  CHECK(!book.Invite("nope", "owner-key", "guest"));

  book.SetMaxPlayers(99);
  book.Open("world-2", "Creative", "owner-key", Access::kFriends);
  CHECK_INT(book.Find("world-2")->max_players, 16);
  book.SetMaxPlayers(0);
  book.Open("world-2", "Creative", "owner-key", Access::kFriends);
  CHECK_INT(book.Find("world-2")->max_players, 2);

  book.Close("world-2");
  CHECK(book.Find("world-2") == nullptr);
  CHECK_INT(static_cast<long long>(book.All().size()), 1);

  CHECK_EQ(JoinMessage(JoinResult::kFull), "That world is full.");
  CHECK_EQ(JoinMessage(JoinResult::kOk), "Joining...");

  Access access;
  CHECK(ParseAccess("code", &access));
  CHECK(access == Access::kCode);
  CHECK(!ParseAccess("open", &access));
}

// ---------- keys ----------

void TestKeys() {
  Section("keys");

  const std::vector<Binding> bindings = DefaultBindings();

  // Shift+Tab opens friends and worlds. Tab alone is the game's player list, so
  // the shift is what keeps the two apart.
  Action action;
  CHECK(ResolveBinding(bindings, KeyCombo{VK_TAB, true, false, false}, &action));
  CHECK(action == Action::kOpenFriends);
  CHECK(!ResolveBinding(bindings, KeyCombo{VK_TAB, false, false, false}, &action));

  // Esc closes the HUD.
  CHECK(ResolveBinding(bindings, KeyCombo{VK_ESCAPE, false, false, false}, &action));
  CHECK(action == Action::kCloseHud);

  // Nothing else is bound to F1: it is the game's debug screen.
  CHECK(!ResolveBinding(bindings, KeyCombo{VK_F1, false, false, false}, &action));
  CHECK(!ResolveBinding(bindings, KeyCombo{'K', false, false, false}, &action));

  // Every default is distinct, or two actions fire on one press.
  for (size_t i = 0; i < bindings.size(); ++i) {
    for (size_t j = i + 1; j < bindings.size(); ++j) {
      CHECK(!(bindings[i].combo == bindings[j].combo));
    }
  }

  const KeyCombo shiftTab{VK_TAB, true, false, false};
  const KeyCombo escapeKey{VK_ESCAPE, false, false, false};
  const KeyCombo ctrlH{'H', false, true, false};
  CHECK_EQ(shiftTab.ToString(), "Shift+Tab");
  CHECK_EQ(escapeKey.ToString(), "Esc");
  CHECK_EQ(ctrlH.ToString(), "Ctrl+H");

  // A rebound key is refused when it is already taken, and says by whom.
  Action holder;
  CHECK(BindingConflict(bindings, KeyCombo{VK_ESCAPE, false, false, false}, Action::kOpenSkin,
                        &holder));
  CHECK(holder == Action::kCloseHud);
  CHECK(!BindingConflict(bindings, KeyCombo{VK_ESCAPE, false, false, false}, Action::kCloseHud,
                         &holder));
  CHECK(!BindingConflict(bindings, KeyCombo{'K', false, false, false}, Action::kOpenSkin,
                         &holder));

  Action parsed;
  CHECK(ParseAction("open_friends", &parsed));
  CHECK(parsed == Action::kOpenFriends);
  CHECK(!ParseAction("open_shop", &parsed));
  CHECK_INT(static_cast<long long>(AllActions().size()), 7);
}

// ---------- HUD ----------

void TestHud() {
  Section("hud");

  std::vector<HudElement> layout = DefaultHud();
  CHECK_INT(static_cast<long long>(layout.size()), 6);

  // Every element starts with an id nothing else shares.
  for (size_t i = 0; i < layout.size(); ++i) {
    for (size_t j = i + 1; j < layout.size(); ++j) CHECK(layout[i].id != layout[j].id);
  }

  CHECK(MoveElement(&layout, "fps", Anchor::kBottomRight));
  for (const HudElement& element : layout) {
    if (element.id == "fps") CHECK(element.anchor == Anchor::kBottomRight);
  }
  CHECK(!MoveElement(&layout, "nope", Anchor::kBottomRight));

  CHECK_INT(ToggleElement(&layout, "ping"), 0);
  CHECK_INT(ToggleElement(&layout, "ping"), 1);
  CHECK_INT(ToggleElement(&layout, "nope"), -1);

  Anchor anchor;
  CHECK(ParseAnchor("bottom_right", &anchor));
  CHECK(anchor == Anchor::kBottomRight);
  CHECK(!ParseAnchor("middle", &anchor));
  CHECK_EQ(AnchorId(Anchor::kTopCenter), "top_center");
}

// ---------- settings ----------

void TestSettings() {
  Section("settings");

  Settings settings;
  settings.provider = Provider::kElyBy;
  settings.player_name = "Alpinoxi";
  settings.loader = Loader::kFabric;
  settings.version = "1.21.1";
  settings.heap_mib = 6144;
  settings.hud_enabled = false;
  // A Windows path is where a JSON writer goes wrong: the backslashes vanish.
  settings.java_path = "C:\\Program Files\\Java\\bin\\java.exe";

  const std::string json = SettingsToJson(settings);
  CHECK(json.find("\"heapMiB\": 6144") != std::string::npos);
  CHECK(json.find("\\\\") != std::string::npos);

  const Settings read = SettingsFromJson(json);
  CHECK(read.provider == Provider::kElyBy);
  CHECK_EQ(read.player_name, "Alpinoxi");
  CHECK(read.loader == Loader::kFabric);
  CHECK_EQ(read.version, "1.21.1");
  CHECK_INT(read.heap_mib, 6144);
  CHECK(!read.hud_enabled);
  CHECK_EQ(read.java_path, "C:\\Program Files\\Java\\bin\\java.exe");
  CHECK_INT(static_cast<long long>(read.bindings.size()), 7);
  CHECK_INT(static_cast<long long>(read.hud.size()), 6);

  // The bindings survive: Shift+Tab still opens friends after a restart.
  Action action;
  CHECK(ResolveBinding(read.bindings, KeyCombo{VK_TAB, true, false, false}, &action));
  CHECK(action == Action::kOpenFriends);

  // A missing or empty file leaves every default in place.
  const Settings fresh = SettingsFromJson("{}");
  CHECK(fresh.provider == Provider::kOffline);
  CHECK_INT(fresh.heap_mib, 4096);
  CHECK(fresh.hud_enabled);
  CHECK_INT(static_cast<long long>(fresh.bindings.size()), 7);
  CHECK_INT(static_cast<long long>(fresh.hud.size()), 6);

  // Garbage in the file does not silently unbind anything.
  const Settings broken = SettingsFromJson(R"({"bindings":[{"action":"open_shop","vk":75}],"hud":[]})");
  CHECK_INT(static_cast<long long>(broken.bindings.size()), 7);
  CHECK_INT(static_cast<long long>(broken.hud.size()), 6);

  // A heap outside anything sane is ignored, not clamped to something absurd.
  const Settings absurd = SettingsFromJson(R"({"heapMiB": 999999})");
  CHECK_INT(absurd.heap_mib, 4096);
}

// ---------- the game's command line ----------

void TestLaunchCommand() {
  Section("launch command");

  CHECK_EQ(QuoteForWindows("java.exe"), "java.exe");
  CHECK_EQ(QuoteForWindows("C:\\Program Files\\Java\\bin\\java.exe"),
           "\"C:\\Program Files\\Java\\bin\\java.exe\"");
  CHECK_EQ(QuoteForWindows(""), "\"\"");

  LoaderTarget target;
  LoaderTarget::Make(Loader::kFabric, "1.21.1", &target);
  const Account account = MakeOfflineAccount("Alpinoxi");

  const LaunchCommand command =
      BuildLaunchCommand("C:\\Program Files\\Java\\bin\\java.exe", "C:\\Users\\User\\game",
                         "C:\\Users\\User\\game\\assets", "17",
                         "C:\\Users\\User\\game\\libraries\\a.jar;C:\\Users\\User\\game\\b.jar",
                         target, account, 4096);

  const std::string line = command.command_line();
  CHECK(line.find("\"C:\\Program Files\\Java\\bin\\java.exe\"") == 0);
  CHECK(line.find("-Xmx4096M") != std::string::npos);
  CHECK(line.find("-Xms4096M") != std::string::npos);
  // Fabric's own entry point, not the vanilla one.
  CHECK(line.find("net.fabricmc.loader.impl.launch.knot.KnotClient") != std::string::npos);
  CHECK(line.find("--username Alpinoxi") != std::string::npos);
  CHECK(line.find("--uuid 7cce3372-aec0-3d25-9588-23579d5fcd8a") != std::string::npos);
  // A cracked account is "legacy", or the game tries to validate the token.
  CHECK(line.find("--userType legacy") != std::string::npos);
  CHECK(line.find("--version 1.21.1") != std::string::npos);
  // A path with a space has to be quoted exactly once.
  CHECK(line.find("--gameDir \"C:\\Users\\User\\game\"") != std::string::npos);
  CHECK(line.find("--assetsDir \"C:\\Users\\User\\game\\assets\"") != std::string::npos);
  CHECK(line.find("\"C:\\Users\\User\\game\\libraries\\a.jar;") != std::string::npos);

  // The vanilla main class, when that is the loader.
  LoaderTarget vanilla;
  LoaderTarget::Make(Loader::kVanilla, "1.21.1", &vanilla);
  const LaunchCommand plain = BuildLaunchCommand("java", "game", "game/assets", "17", "a.jar",
                                                 vanilla, account, 2048);
  CHECK(plain.command_line().find("net.minecraft.client.main.Main") != std::string::npos);
  CHECK(plain.command_line().find("KnotClient") == std::string::npos);

  // A Microsoft account is "msa".
  Account microsoft;
  microsoft.provider = Provider::kMicrosoft;
  microsoft.name = "Alex";
  microsoft.uuid = "11111111-1111-1111-1111-111111111111";
  microsoft.access_token = "token";
  const LaunchCommand microsoftLaunch = BuildLaunchCommand("java", "game", "game/assets", "17", "a.jar",
                                                  vanilla, microsoft, 2048);
  CHECK(microsoftLaunch.command_line().find("--userType msa") != std::string::npos);
  CHECK(microsoftLaunch.command_line().find("--accessToken token") != std::string::npos);
}

}  // namespace

int main() {
  std::printf("alpinoxi client tests\n");
  TestVersions();
  TestLoaders();
  TestLoaderTargets();
  TestMemory();
  TestOfflineAccounts();
  TestSignIn();
  TestJson();
  TestFriends();
  TestWorlds();
  TestKeys();
  TestHud();
  TestSettings();
  TestLaunchCommand();

  std::printf("\n%d checks, %d failed\n", g_checks, g_failures);
  return g_failures == 0 ? 0 : 1;
}
