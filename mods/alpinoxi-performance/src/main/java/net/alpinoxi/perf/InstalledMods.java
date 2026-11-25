package net.alpinoxi.perf;

import net.fabricmc.loader.api.FabricLoader;

/** Which Alpinoxi-known performance mods are actually present. */
public final class InstalledMods {
    private static final String[] KNOWN = {
        "sodium", "lithium", "ferrite-core", "krypton", "modernfix",
        "entityculling", "indium", "immediatelyfast", "memoryleakfix"
    };

    private InstalledMods() {
    }

    public static String csv() {
        StringBuilder builder = new StringBuilder();
        for (String id : KNOWN) {
            if (FabricLoader.getInstance().isModLoaded(id)) {
                if (!builder.isEmpty()) {
                    builder.append(',');
                }
                builder.append(id);
            }
        }
        return builder.toString();
    }
}
