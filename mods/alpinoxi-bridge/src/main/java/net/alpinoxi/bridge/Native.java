package net.alpinoxi.bridge;

/**
 * Java side of the Rust bridge.
 *
 * <p>Every method here is a thin wrapper: the mods call these, the native library
 * decides. Signatures must match {@code packages/alpinoxi_jni/src/lib.rs} exactly
 * (JNI name mangling: {@code Java_net_alpinoxi_bridge_Native_<method>}).
 */
public final class Native {
    private static volatile boolean loaded;
    private static volatile String loadError;

    private Native() {
    }

    /**
     * Loads the native library once. Returns true when the bridge is usable; the
     * mods then fall back to their built-in defaults if it is not, so a missing
     * DLL never stops the game from starting.
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
            BridgeLog.warn("Alpinoxi bridge unavailable, using Java defaults ({})", loadError);
            return false;
        }
    }

    public static boolean isLoaded() {
        return loaded;
    }

    public static String loadError() {
        return loadError;
    }

    /** JSON array of HUD rows: id, label, anchor, x, y, offset_x, offset_y. */
    public static native String hudLayout();

    /** One of: nothing, open_hud_editor, toggle_hud, toggle:<element>, open_friends, open_chat. */
    public static native String resolveKey(String key, int mods);

    /** Feeds one chat line in; returns {mentions, unread} or {error}. */
    public static native String receiveChat(String from, String provider, String channel, String text, long timestamp);

    /** Moves one HUD element; false when the id or the anchor is unknown. */
    public static native boolean setAnchor(String elementId, String anchor);

    /** Toggles one element: 1 visible, 0 hidden, -1 unknown id. */
    public static native int toggleElement(String elementId);

    /** Frame/heap advice as JSON. */
    public static native String performanceReport(int heapMib, String installedModsCsv);

    /** Whole bridge state as JSON, for the debug screen. */
    public static native String snapshot();

    /** Bridge (Rust crate) version. */
    public static native String bridgeVersion();
}
