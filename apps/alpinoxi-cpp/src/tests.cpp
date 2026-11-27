// The launcher's decisions, tested without a window.
//
// Build: cmake --build build --target alpinoxi-tests && build/alpinoxi-tests
// A launcher that cannot be tested on a build machine cannot be trusted on a
// player's.

#include <cstdio>
#include <string>
#include <vector>

#include "app.h"

using namespace alpinoxi;

namespace {

int gFailures = 0;
int gChecks = 0;

void Check(bool condition, const char* what) {
  ++gChecks;
  if (!condition) {
    ++gFailures;
    std::printf("  FAILED: %s\n", what);
  }
}

void CheckEqual(const std::string& actual, const std::string& expected, const char* what) {
  ++gChecks;
  if (actual != expected) {
    ++gFailures;
    std::printf("  FAILED: %s\n    expected: %s\n    actual:   %s\n", what, expected.c_str(),
                actual.c_str());
  }
}

void TestVersionsParse() {
  GameVersion version;
  Check(GameVersion::Parse("1.21.1", &version) && version.major == 1 && version.minor == 21 &&
            version.patch == 1,
        "1.21.1 parses");
  Check(GameVersion::Parse("1.8", &version) && version.patch == 0, "1.8 parses with patch 0");
  Check(GameVersion::Parse("26.3", &version) && version.major == 26, "26.3 parses");
  Check(GameVersion::Parse(" 1.20 ", &version) && version.minor == 20, "spaces are trimmed");
}

void TestSnapshotsRefused() {
  GameVersion version;
  for (const char* bad : {"24w14a", "1.21-pre1", "1.21.1.2", "", "latest", "1"}) {
    Check(!GameVersion::Parse(bad, &version), bad);
  }
}

void TestVersionOrder() {
  std::vector<GameVersion> versions;
  for (const char* text : {"1.21.1", "1.8", "1.14", "1.21", "26.3"}) {
    GameVersion version;
    GameVersion::Parse(text, &version);
    versions.push_back(version);
  }
  for (size_t i = 0; i + 1 < versions.size(); ++i) {
    for (size_t j = i + 1; j < versions.size(); ++j) {
      if (versions[j] < versions[i]) std::swap(versions[i], versions[j]);
    }
  }
  CheckEqual(versions[0].ToString(), "1.8", "1.8 sorts first");
  CheckEqual(versions[1].ToString(), "1.14", "1.14 second");
  CheckEqual(versions[2].ToString(), "1.21", "1.21 third");
  CheckEqual(versions[3].ToString(), "1.21.1", "1.21.1 after 1.21");
  CheckEqual(versions[4].ToString(), "26.3", "26.3 last");
}

void TestFabricHasNoOldVersions() {
  // Fabric's own index: 266 stable versions, the oldest 1.14. Asking for 1.13.2
  // answers HTTP 400, so the launcher refuses it before trying.
  LoaderTarget target;
  Check(LoaderTarget::Make(Loader::kFabric, "1.8", &target) == LoaderError::kBeforeTheLoader,
        "Fabric refuses 1.8");
  Check(LoaderTarget::Make(Loader::kFabric, "1.13.2", &target) == LoaderError::kBeforeTheLoader,
        "Fabric refuses 1.13.2");
  Check(LoaderTarget::Make(Loader::kFabric, "1.14", &target) == LoaderError::kNone,
        "Fabric accepts 1.14");
  Check(LoaderTarget::Make(Loader::kQuilt, "1.14.4", &target) == LoaderError::kNone,
        "Quilt accepts 1.14.4");
  Check(LoaderTarget::Make(Loader::kQuilt, "1.13.2", &target) == LoaderError::kBeforeTheLoader,
        "Quilt refuses 1.13.2");
}

void TestForgeCoversEverything() {
  for (const char* version : {"1.8", "1.12.2", "1.16.5", "1.21.1", "26.3"}) {
    LoaderTarget target;
    Check(LoaderTarget::Make(Loader::kForge, version, &target) == LoaderError::kNone, version);
  }
}

void TestSnapshotRefusedEverywhere() {
  for (Loader loader : AllLoaders()) {
    LoaderTarget target;
    Check(LoaderTarget::Make(loader, "24w14a", &target) == LoaderError::kUnreadableVersion,
          LoaderLabel(loader));
  }
}

void TestLoadersFor() {
  // Vanilla is always offered: the game runs without a loader, and the player
  // must be able to choose that.
  Check(LoadersFor("1.8").size() == 2, "1.8 has vanilla and Forge");
  Check(LoadersFor("1.8").back() == Loader::kForge, "and the loader there is Forge");
  Check(LoadersFor("1.14").size() == 4, "1.14 has vanilla and all three loaders");
  Check(LoadersFor("1.21.1").size() == 4, "1.21.1 has vanilla and all three");
  Check(LoadersFor("snapshot").empty(), "a snapshot has none");
}

void TestUrls() {
  LoaderTarget fabric;
  LoaderTarget::Make(Loader::kFabric, "1.21.1", &fabric);
  Check(fabric.ProfileUrl().find("meta.fabricmc.net") != std::string::npos, "fabric url host");
  Check(fabric.ProfileUrl().find("1.21.1") != std::string::npos, "fabric url version");

  LoaderTarget forge;
  LoaderTarget::Make(Loader::kForge, "1.8", &forge);
  Check(forge.ProfileUrl().find("maven.minecraftforge.net") != std::string::npos, "forge url host");

  for (Loader loader : AllLoaders()) {
    Check(std::string(LoaderVersionsUrl(loader)).rfind("https://", 0) == 0, "https metadata");
  }
}

void TestMessageIsReadable() {
  LoaderTarget target;
  LoaderTarget::Make(Loader::kFabric, "1.8", &target);
  target.loader = Loader::kFabric;
  const std::string message = target.Message(LoaderError::kBeforeTheLoader);
  Check(message.find("Fabric") != std::string::npos, "the message names the loader");
  Check(message.find("1.14") != std::string::npos, "the message names where it starts");
}

void TestHeapCap() {
  Check(PlanHeapMiB(99999, 8192) == 4096, "half of 8 GiB");
  Check(PlanHeapMiB(99999, 65536) == 8192, "the ceiling");
  Check(PlanHeapMiB(1024, 8192) == 1024, "the wish, when it fits");
  Check(PlanHeapMiB(256, 8192) == 1024, "never below a usable floor");
}

void TestOfflineUuid() {
  // The values Minecraft itself produces. A wrong UUID puts a cracked player in
  // a different slot than their skin expects.
  CheckEqual(OfflineUuid("Notch"), "b50ad385-829d-3141-a216-7e7d7539ba7f", "Notch");
  CheckEqual(OfflineUuid("Alpinoxi"), "7cce3372-aec0-3d25-9588-23579d5fcd8a", "Alpinoxi");
  CheckEqual(OfflineUuid("Steve"), "5627dd98-e6be-3c21-b8a8-e92344183641", "Steve");
  CheckEqual(OfflineUuid("Player"), "a01e3843-e521-3998-958a-f459800e4d11", "Player");
}

void TestPlayerName() {
  Check(ValidPlayerName("Alpinoxi"), "Alpinoxi is a name");
  Check(!ValidPlayerName("ab"), "two characters is too short");
  Check(!ValidPlayerName("this_name_is_too_long"), "seventeen characters is too long");
  Check(!ValidPlayerName("has space"), "spaces are not allowed");
  Check(!ValidPlayerName("naïve"), "only ASCII is allowed");
}

void TestLoaderIds() {
  for (Loader loader : AllLoaders()) {
    Loader parsed;
    Check(ParseLoader(LoaderId(loader), &parsed) && parsed == loader, "id round-trips");
  }
  Loader neo;
  Check(ParseLoader("neoforge", &neo) && neo == Loader::kForge, "neoforge is forge");
  Loader vanilla;
  Check(ParseLoader("vanilla", &vanilla) && vanilla == Loader::kVanilla,
        "vanilla is a loader too");
}


void TestPkceMatchesTheSpec() {
  // RFC 7636's own worked example.
  CheckEqual(PkceChallenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
             "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM", "RFC 7636 challenge");
  Check(RandomUrlSafe(48).size() == 64, "48 bytes are 64 url-safe characters");
  Check(RandomUrlSafe(24).size() == 32, "24 bytes are 32 url-safe characters");
  Check(RandomUrlSafe(32) != RandomUrlSafe(32), "two calls never agree");
}

void TestSignInUrls() {
  SignInRequest ms = StartSignIn(Provider::kMicrosoft, "http://127.0.0.1:51234/callback");
  Check(ms.authorize_url.find("login.live.com") != std::string::npos, "Microsoft host");
  Check(ms.authorize_url.find("code_challenge_method=S256") != std::string::npos, "PKCE S256");
  Check(ms.authorize_url.find(ms.state) != std::string::npos, "state is in the URL");
  Check(ms.authorize_url.find("00000000402b5328") != std::string::npos, "Minecraft's client id");

  SignInRequest ely = StartSignIn(Provider::kElyBy, "http://127.0.0.1:51234/callback");
  Check(ely.authorize_url.find("account.ely.by") != std::string::npos, "Ely.by host");
  Check(ely.authorize_url.find("alpinoxi-client") != std::string::npos, "our client id");

  Check(StartSignIn(Provider::kOffline, "x").authorize_url.empty(), "offline opens no browser");
}

void TestJvmFlags() {
  const auto small = JvmFlags(4096);
  bool hasG1 = false;
  for (const auto& flag : small) {
    if (flag == "-XX:+UseG1GC") hasG1 = true;
  }
  Check(hasG1, "G1 is on");
  Check(small.back().find("aikars.flags") != std::string::npos, "the flags name their source");

  bool pretouchSmall = false;
  for (const auto& flag : small) {
    if (flag == "-XX:+AlwaysPreTouch") pretouchSmall = true;
  }
  Check(!pretouchSmall, "4 GiB does not pre-touch");

  bool pretouchBig = false;
  for (const auto& flag : JvmFlags(8192)) {
    if (flag == "-XX:+AlwaysPreTouch") pretouchBig = true;
  }
  Check(pretouchBig, "8 GiB does");
}

void TestJson() {
  const std::string profile =
      R"({"id":"1.21.1","libraries":[{"name":"a"},{"name":"b"}],"mainClass":"KnotClient","n":42})";
  std::string text;
  Check(JsonString(profile, "mainClass", &text) && text == "KnotClient", "reads a string");
  Check(JsonString(profile, "id", &text) && text == "1.21.1", "reads the first key too");
  Check(!JsonString(profile, "missing", &text), "a missing key says so");
  double number = 0;
  Check(JsonNumber(profile, "n", &number) && number == 42, "reads a number");
  Check(JsonArrayCount(profile, "libraries") == 2, "counts the libraries");
  Check(JsonArrayCount(profile, "nothing") == 0, "an absent array is zero");

  const std::string escaped = R"({"msg":"he said \"hi\"\n"})";
  Check(JsonString(escaped, "msg", &text) && text == "he said \"hi\"\n", "unescapes quotes");
}

void TestCommandLine() {
  LoaderTarget target;
  LoaderTarget::Make(Loader::kFabric, "1.21.1", &target);
  Account account = MakeOfflineAccount("Alpinoxi");

  const LaunchCommand command =
      BuildLaunchCommand("C:\\Java\\bin\\java.exe", "C:\\Games\\My Instance",
                         "C:\\Games\\assets", "17", "a.jar;b.jar", target, account, 4096);

  Check(command.java.find("java.exe") != std::string::npos, "the java path");
  Check(command.arguments[0] == "-Xms4096M", "the initial heap");
  Check(command.arguments[1] == "-Xmx4096M", "the maximum heap");

  bool mainClass = false, username = false, uuid = false, token = false;
  for (size_t i = 0; i < command.arguments.size(); ++i) {
    if (command.arguments[i] == "net.fabricmc.loader.impl.launch.knot.KnotClient") mainClass = true;
    if (command.arguments[i] == "--username" && command.arguments[i + 1] == "Alpinoxi") username = true;
    if (command.arguments[i] == "--uuid" && command.arguments[i + 1] == account.uuid) uuid = true;
    if (command.arguments[i] == "--accessToken" && command.arguments[i + 1] == "0") token = true;
  }
  Check(mainClass, "the loader's main class");
  Check(username, "the player name");
  Check(uuid, "the offline UUID");
  Check(token, "a cracked client sends token 0");

  CheckEqual(QuoteForWindows("a b"), "\"a b\"", "a space is quoted");
  CheckEqual(QuoteForWindows("plain"), "plain", "a plain argument is not");
}

void TestProviders() {
  CheckEqual(ProviderLabel(Provider::kMicrosoft), "Microsoft", "Microsoft label");
  CheckEqual(ProviderLabel(Provider::kElyBy), "Ely.by", "Ely.by label");
  CheckEqual(ProviderLabel(Provider::kOffline), "Cracked", "offline is shown as Cracked");
  for (Provider provider : AllProviders()) {
    Provider parsed;
    Check(ParseProvider(ProviderId(provider), &parsed) && parsed == provider, "id round-trips");
  }
  Provider parsed;
  Check(ParseProvider("cracked", &parsed) && parsed == Provider::kOffline, "cracked is offline");
}

void TestOfflineAccount() {
  Account account = MakeOfflineAccount("Alpinoxi");
  CheckEqual(account.name, "Alpinoxi", "the name is kept");
  CheckEqual(account.uuid, "7cce3372-aec0-3d25-9588-23579d5fcd8a", "and its UUID");
  CheckEqual(account.access_token, "0", "a cracked account carries token 0");
  Check(!ValidPlayerName(MakeOfflineAccount("x").name), "a bad name makes no account");
}


void TestVanillaIsALoader() {
  CheckEqual(LoaderLabel(Loader::kVanilla), "Vanilla", "vanilla has a label");
  CheckEqual(LoaderId(Loader::kVanilla), "vanilla", "and an id");
  CheckEqual(LoaderMainClass(Loader::kVanilla), "net.minecraft.client.main.Main",
             "vanilla starts the game itself");
  Loader parsed;
  Check(ParseLoader("vanilla", &parsed) && parsed == Loader::kVanilla, "parses");
  Check(AllLoaders().size() == 4, "four loaders, vanilla first");
  Check(AllLoaders().front() == Loader::kVanilla, "and it is first");
}

void TestTargetLabel() {
  LoaderTarget fabric;
  LoaderTarget::Make(Loader::kFabric, "1.21.1", &fabric);
  CheckEqual(fabric.Label(), "Fabric 1.21.1", "the rail shows the loader and the version");

  LoaderTarget vanilla;
  LoaderTarget::Make(Loader::kVanilla, "1.21.1", &vanilla);
  CheckEqual(vanilla.Label(), "1.21.1", "vanilla is just the version");
}

void TestSettingsRoundTrip() {
  Settings settings;
  settings.provider = Provider::kElyBy;
  settings.player_name = "Alpinoxi";
  settings.loader = Loader::kQuilt;
  settings.version = "1.20.1";
  settings.heap_mib = 6144;
  settings.hud_enabled = false;
  settings.java_path = "C:\\Java\\bin\\java.exe";

  const Settings read = SettingsFromJson(SettingsToJson(settings));
  Check(read.provider == Provider::kElyBy, "provider survives the file");
  CheckEqual(read.player_name, "Alpinoxi", "name survives");
  Check(read.loader == Loader::kQuilt, "loader survives");
  CheckEqual(read.version, "1.20.1", "version survives");
  Check(read.heap_mib == 6144, "heap survives");
  Check(!read.hud_enabled, "hud off survives");
  CheckEqual(read.java_path, settings.java_path, "the java path survives");

  const Settings blank = SettingsFromJson("{}");
  Check(blank.provider == Provider::kOffline, "an empty file is the default");
  Check(blank.heap_mib == 4096, "with 4 GiB");
  Check(blank.hud_enabled, "and the HUD on");
}

void TestInstanceIds() {
  GameVersion version;
  GameVersion::Parse("1.21.1", &version);
  CheckEqual(InstanceId(Loader::kFabric, version), "fabric-1.21.1", "fabric id");
  CheckEqual(InstanceId(Loader::kVanilla, version), "vanilla-1.21.1", "vanilla id");
  Check(InstanceId(Loader::kFabric, version) != InstanceId(Loader::kQuilt, version),
        "two loaders never share a folder");
}

}  // namespace

int main() {
  TestVersionsParse();
  TestSnapshotsRefused();
  TestVersionOrder();
  TestFabricHasNoOldVersions();
  TestForgeCoversEverything();
  TestSnapshotRefusedEverywhere();
  TestLoadersFor();
  TestUrls();
  TestMessageIsReadable();
  TestHeapCap();
  TestOfflineUuid();
  TestPlayerName();
  TestLoaderIds();
  TestPkceMatchesTheSpec();
  TestSignInUrls();
  TestJvmFlags();
  TestJson();
  TestCommandLine();
  TestProviders();
  TestOfflineAccount();
  TestVanillaIsALoader();
  TestTargetLabel();
  TestSettingsRoundTrip();
  TestInstanceIds();

  if (gFailures == 0) {
    std::printf("OK: %d checks, the loader rules and the offline UUID hold\n", gChecks);
    return 0;
  }
  std::printf("FAILED: %d of %d checks\n", gFailures, gChecks);
  return 1;
}
