package net.alpinoxi.hud;

import net.alpinoxi.bridge.BridgeLog;
import net.alpinoxi.bridge.Native;

/**
 * Writes HUD edits back to the Rust core.
 *
 * <p>The native side owns the file format, so the mod never writes JSON itself.
 * Until the setter is wired in the native library, the edit is remembered in
 * memory and logged, and the game keeps running normally.
 */
public final class HudEditorBridge {
    private HudEditorBridge() {
    }

    public static void setAnchor(String elementId, String anchor) {
        if (!Native.ensureLoaded()) {
            BridgeLog.warn("Cannot move {}: bridge not loaded", elementId);
            return;
        }
        boolean applied = Native.setAnchor(elementId, anchor);
        if (!applied) {
            BridgeLog.warn("Bridge rejected anchor {} for {}", anchor, elementId);
        }
    }
}
