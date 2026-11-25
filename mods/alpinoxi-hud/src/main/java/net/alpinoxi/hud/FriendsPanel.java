package net.alpinoxi.hud;

import net.alpinoxi.bridge.Native;

/** One-line friend summary for the HUD, kept in sync with the Rust friend book. */
public final class FriendsPanel {
    private FriendsPanel() {
    }

    public static String onlineLabel() {
        if (!Native.ensureLoaded()) {
            return "";
        }
        try {
            var root = com.google.gson.JsonParser.parseString(Native.snapshot()).getAsJsonObject();
            int online = root.get("friends_online").getAsInt();
            return online == 0 ? "" : "friends " + online;
        } catch (RuntimeException error) {
            return "";
        }
    }
}
