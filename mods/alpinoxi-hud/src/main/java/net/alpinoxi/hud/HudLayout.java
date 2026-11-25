package net.alpinoxi.hud;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import net.alpinoxi.bridge.Native;

/**
 * The HUD layout for this frame.
 *
 * <p>Rust owns the rules; this class only refreshes the list a few times a second
 * and keeps the last good result when the bridge is unavailable. Refreshing on a
 * timer instead of every frame is deliberate: parsing JSON per frame would cost
 * more than it saves.
 */
public final class HudLayout {
    private static final int REFRESH_TICKS = 10;

    private static List<HudRow> rows = List.of();
    private static int ticks;
    private static boolean enabled = true;

    private HudLayout() {
    }

    public static void tick() {
        if (ticks++ % REFRESH_TICKS != 0) {
            return;
        }
        if (!Native.ensureLoaded()) {
            return;
        }
        try {
            String json = Native.hudLayout();
            JsonObject root = JsonParser.parseString(json).getAsJsonObject();
            JsonArray elements = root.getAsJsonArray("elements");
            List<HudRow> parsed = new ArrayList<>(elements.size());
            for (JsonElement element : elements) {
                JsonObject row = element.getAsJsonObject();
                parsed.add(new HudRow(
                        row.get("id").getAsString(),
                        row.get("label").getAsString(),
                        row.get("anchor").getAsString(),
                        row.get("x").getAsFloat(),
                        row.get("y").getAsFloat(),
                        row.get("offset_x").getAsInt(),
                        row.get("offset_y").getAsInt()));
            }
            rows = Collections.unmodifiableList(parsed);
            enabled = root.has("hud_enabled") ? root.get("hud_enabled").getAsBoolean() : true;
        } catch (RuntimeException error) {
            // A malformed answer must not break rendering: keep the last layout.
            net.alpinoxi.bridge.BridgeLog.warn("HUD layout could not be read: {}", error.getMessage());
        }
    }

    public static List<HudRow> rows() {
        return rows;
    }

    public static boolean isEnabled() {
        return enabled;
    }
}
