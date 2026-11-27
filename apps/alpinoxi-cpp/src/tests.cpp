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
  Check(LoadersFor("1.8").size() == 1, "1.8 has Forge only");
  Check(LoadersFor("1.8").front() == Loader::kForge, "and that loader is Forge");
  Check(LoadersFor("1.14").size() == 3, "1.14 has all three");
  Check(LoadersFor("1.21.1").size() == 3, "1.21.1 has all three");
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
  Loader none;
  Check(!ParseLoader("vanilla", &none), "vanilla is not a loader");
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

  if (gFailures == 0) {
    std::printf("OK: %d checks, the loader rules and the offline UUID hold\n", gChecks);
    return 0;
  }
  std::printf("FAILED: %d of %d checks\n", gFailures, gChecks);
  return 1;
}
