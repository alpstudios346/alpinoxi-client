package net.alpinoxi.bridge;

/**
 * Java side of the bridge.
 *
 * <p>Every method here is a thin wrapper: the mods call these, and either the
 * native library or {@link HudState} decides. A missing library is not an error
 * the player should ever see: the same rules run in Java, so the HUD, the
 * keybinds and the chat work with no DLL at all.
 */
public final class Native {
    private static volatile boolean loaded;
    private static volatile String loadError;

    private Native() {
    }

    /**
     * Tries the native library once. Returns true when it is usable; the mods
     * then fall back to the Java rules if it is not, so a missing DLL never
     * stops the game from starting.
     */
    public static synchronized boolean ensureLoaded() {
        if (loaded) {
            return true;
        }
        if (loadError != null) {
            return false;
        }
        try {
            String explicit = System.getProperty("alpinoxi.bridge");
            if (explicit != null && !explicit.isBlank()) {
                System.load(explicit);
            } else {
                System.loadLibrary("alpinoxi_bridge");
            }
            loaded = true;
            return true;
        } catch (Throwable error) {
            loadError = error.getClass().getSimpleName() + ": " + error.getMessage();
            BridgeLog.info("Alpinoxi bridge running the Java rules ({})", loadError);
            return false;
        }
    }

    public static boolean isLoaded() {
        return loaded;
    }

    public static String loadError() {
        return loadError;
    }

    // ---------- HUD ----------

    /**
     * The HUD layout as JSON: {"hud_enabled":bool,"elements":[{id,label,anchor,x,y,
     * offset_x,offset_y}]}. x and y are the anchor's fractions of the screen, so
     * a row lands in the same corner on any resolution.
     */
    public static String hudLayout() {
        if (ensureLoaded()) {
            try {
                return nativeHudLayout();
            } catch (Throwable error) {
                BridgeLog.warn("native hudLayout failed, using Java ({})", error.toString());
            }
        }
        StringBuilder json = new StringBuilder(256);
        json.append("{\"hud_enabled\":").append(HudState.hudEnabled()).append(",\"elements\":[");
        boolean first = true;
        for (HudState.Element element : HudState.elements()) {
            if (!first) {
                json.append(',');
            }
            first = false;
            json.append("{\"id\":\"").append(element.id()).append("\",\"label\":\"")
                    .append(element.label()).append("\",\"anchor\":\"")
                    .append(element.anchor().id()).append("\",\"x\":")
                    .append(element.anchor().dx() < 0 ? 1.0f : 0.0f).append(",\"y\":")
                    .append(element.anchor().dy() < 0 ? 1.0f : 0.0f).append(",\"offset_x\":")
                    .append(element.offsetX()).append(",\"offset_y\":").append(element.offsetY())
                    .append(",\"visible\":").append(element.visible()).append('}');
        }
        json.append("]}");
        return json.toString();
    }

    /** Moves one HUD element; false when the id or the anchor is unknown. */
    public static boolean setAnchor(String elementId, String anchor) {
        if (ensureLoaded()) {
            try {
                return nativeSetAnchor(elementId, anchor);
            } catch (Throwable error) {
                BridgeLog.warn("native setAnchor failed, using Java ({})", error.toString());
            }
        }
        return HudState.setAnchor(elementId, anchor);
    }

    /** Toggles one element: 1 visible, 0 hidden, -1 unknown id. */
    public static int toggleElement(String elementId) {
        if (ensureLoaded()) {
            try {
                return nativeToggleElement(elementId);
            } catch (Throwable error) {
                BridgeLog.warn("native toggleElement failed, using Java ({})", error.toString());
            }
        }
        return HudState.toggle(elementId);
    }

    // ---------- chat ----------

    /** Feeds one chat line in; returns {mentions, unread} or {error}. */
    public static String receiveChat(String from, String provider, String channel, String text,
                                     long timestamp) {
        if (ensureLoaded()) {
            try {
                return nativeReceiveChat(from, provider, channel, text, timestamp);
            } catch (Throwable error) {
                BridgeLog.warn("native receiveChat failed, using Java ({})", error.toString());
            }
        }
        if (from == null || text == null || text.isBlank()) {
            return "{\"error\":\"empty\"}";
        }
        boolean mentioned = HudState.receive(from, provider == null ? "unknown" : provider,
                channel == null ? "chat" : channel, text, timestamp);
        return "{\"mentions\":" + mentioned + ",\"unread\":" + HudState.unread() + "}";
    }

    // ---------- friends and hosted worlds ----------

    /**
     * The friend book as JSON, one row per friend: {provider, uuid, name,
     * presence, world}. Presence is offline, launcher or in_game.
     */
    public static String friends() {
        if (ensureLoaded()) {
            try {
                return nativeFriends();
            } catch (Throwable error) {
                BridgeLog.warn("native friends failed, using Java ({})", error.toString());
            }
        }
        StringBuilder json = new StringBuilder(256);
        json.append("{\"online\":").append(Friends.onlineCount()).append(",\"joinable\":")
                .append(Friends.joinable().size()).append(",\"friends\":[");
        boolean first = true;
        for (Friends.Friend friend : Friends.all()) {
            if (!first) {
                json.append(',');
            }
            first = false;
            json.append("{\"provider\":\"").append(friend.provider()).append("\",\"uuid\":\"")
                    .append(friend.uuid()).append("\",\"name\":\"").append(friend.name())
                    .append("\",\"presence\":\"").append(friend.presence().id())
                    .append("\",\"world\":\"").append(friend.worldId() == null ? "" : friend.worldId())
                    .append("\"}");
        }
        json.append("]}");
        return json.toString();
    }

    /** Every open world as JSON, with the code each one hands out. */
    public static String worlds() {
        if (ensureLoaded()) {
            try {
                return nativeWorlds();
            } catch (Throwable error) {
                BridgeLog.warn("native worlds failed, using Java ({})", error.toString());
            }
        }
        StringBuilder json = new StringBuilder(256);
        json.append("{\"current\":\"").append(HostWorld.current() == null ? "" : HostWorld.current())
                .append("\",\"worlds\":[");
        boolean first = true;
        for (HostWorld.World world : HostWorld.openWorlds()) {
            if (!first) {
                json.append(',');
            }
            first = false;
            json.append("{\"id\":\"").append(world.id()).append("\",\"name\":\"")
                    .append(world.name()).append("\",\"owner\":\"").append(world.ownerKey())
                    .append("\",\"access\":\"").append(world.access().id())
                    .append("\",\"code\":\"").append(world.code()).append("\",\"players\":")
                    .append(world.players()).append(",\"max\":").append(world.maxPlayers())
                    .append('}');
        }
        json.append("]}");
        return json.toString();
    }

    /** Opens, closes or re-opens this player's world. Returns the code, or "". */
    public static String hostWorld(String worldId, String name, String access) {
        if (ensureLoaded()) {
            try {
                return nativeHostWorld(worldId, name, access);
            } catch (Throwable error) {
                BridgeLog.warn("native hostWorld failed, using Java ({})", error.toString());
            }
        }
        HostWorld.Access mode = HostWorld.Access.byId(access);
        if (mode == HostWorld.Access.CLOSED) {
            HostWorld.close(worldId);
            return "";
        }
        return HostWorld.open(worldId, name, HudState.Settings.instance(), mode).code();
    }

    /** Asks to join a world by id or code. Returns OK, or the reason it is not. */
    public static String joinWorld(String worldIdOrCode, String code) {
        if (ensureLoaded()) {
            try {
                return nativeJoinWorld(worldIdOrCode, code);
            } catch (Throwable error) {
                BridgeLog.warn("native joinWorld failed, using Java ({})", error.toString());
            }
        }
        HostWorld.JoinResult result =
                HostWorld.mayJoin(worldIdOrCode, HudState.Settings.instance(), true, code);
        if (result == HostWorld.JoinResult.OK) {
            HostWorld.World world = HostWorld.find(worldIdOrCode);
            HostWorld.enter(world == null ? worldIdOrCode : world.id());
        }
        return result.message();
    }

    // ---------- performance ----------

    /**
     * Frame and heap advice as JSON: {"advice":"…","heap_mib":n,"pressure":"low"}.
     * The rules are the launcher's own: a client that is short of heap is told
     * so, and one that has plenty is told nothing at all.
     */
    public static String performanceReport(int heapMib, String installedModsCsv) {
        if (ensureLoaded()) {
            try {
                return nativePerformanceReport(heapMib, installedModsCsv);
            } catch (Throwable error) {
                BridgeLog.warn("native performanceReport failed, using Java ({})", error.toString());
            }
        }
        return PerformanceRules.report(heapMib, installedModsCsv);
    }

    // ---------- the debug screen ----------

    /** Whole bridge state as JSON, for the debug screen. */
    public static String snapshot() {
        if (ensureLoaded()) {
            try {
                return nativeSnapshot();
            } catch (Throwable error) {
                BridgeLog.warn("native snapshot failed, using Java ({})", error.toString());
            }
        }
        StringBuilder json = new StringBuilder(256);
        json.append("{\"source\":\"java\",\"hud_enabled\":").append(HudState.hudEnabled())
                .append(",\"elements\":").append(HudState.elements().size())
                .append(",\"unread\":").append(HudState.unread())
                .append(",\"launcher\":").append(HudState.launcherPresent())
                .append(",\"instance\":\"").append(HudState.Settings.instance())
                .append("\",\"heap_mib\":").append(HudState.Settings.heapMib()).append('}');
        return json.toString();
    }

    /** Bridge version. The Java rules and the native library report the same one. */
    public static String bridgeVersion() {
        if (ensureLoaded()) {
            try {
                return nativeBridgeVersion();
            } catch (Throwable error) {
                BridgeLog.warn("native bridgeVersion failed, using Java ({})", error.toString());
            }
        }
        return "0.3.0-java";
    }

    // ---------- the native library, when it is there ----------

    private static native String nativeHudLayout();

    private static native String nativeResolveKey(String key, int mods);

    private static native String nativeReceiveChat(String from, String provider, String channel,
                                                   String text, long timestamp);

    private static native boolean nativeSetAnchor(String elementId, String anchor);

    private static native int nativeToggleElement(String elementId);

    private static native String nativePerformanceReport(int heapMib, String installedModsCsv);

    private static native String nativeSnapshot();

    private static native String nativeBridgeVersion();

    private static native String nativeFriends();

    private static native String nativeWorlds();

    private static native String nativeHostWorld(String worldId, String name, String access);

    private static native String nativeJoinWorld(String worldIdOrCode, String code);
}
