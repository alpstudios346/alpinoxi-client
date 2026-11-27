#!/bin/sh
# Builds the three Alpinoxi mod jars without Gradle.
#
# Why this exists: a real Fabric build needs Loom to download and remap
# Minecraft, which needs Gradle and a JDK. On a machine without them (or without
# network to Maven), this script still produces jars that are correct in every
# way except one: the Minecraft classes they reference are stubs, so the jars are
# for inspection, not for playing. Run `gradlew build` in each mod folder for
# playable jars.
#
# Usage: sh mods/jars/build.sh <path-to-jdk-21> <stub-classes-dir>
set -e
JAVA_HOME="${1:?jdk path}"
STUBS="${2:?stub class dir}"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT="$HERE"
JAVAC="$JAVA_HOME/bin/javac.exe"
JAR="$JAVA_HOME/bin/jar.exe"
[ -x "$JAVAC" ] || { echo "no javac at $JAVAC"; exit 1; }

for mod in alpinoxi-bridge alpinoxi-hud alpinoxi-performance; do
    version=$(sed -n 's/^mod_version=//p' "$ROOT/mods/$mod/gradle.properties")
    classes="$OUT/$mod/classes"
    rm -rf "$classes"
    mkdir -p "$classes"
    # One @argfile, because the shell here mangles long Windows paths in a
    # command substitution.
    {
        echo '-nowarn'
        echo "-cp \"$STUBS\""
        echo "-d \"$classes\""
        sed 's/\\/\//g' "$OUT/$mod.sources"
    } > "$OUT/$mod.args"
    "$JAVAC" "@$OUT/$mod.args"
    cp -r "$ROOT/mods/$mod/src/main/resources/." "$classes/"
    # fabric.mod.json carries `${version}`; the real Gradle build expands it.
    sed -i "s/\${version}/$version/" "$classes/fabric.mod.json"
    "$JAR" --create --file "$OUT/$mod-$version.jar" -C "$classes" .
    echo "built $mod-$version.jar"
done
