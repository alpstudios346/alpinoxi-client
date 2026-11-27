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
}
