package net.alpinoxi.hud;

import java.util.List;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;

/**
 * Draws the HUD rows the Rust core decided on.
 *
 * <p>Only formatting lives here: numbers come from the game, positions come from
 * {@link HudLayout}. Each element is a small method so a new element is one entry
 * in Rust plus one case here, never a tangle of conditionals.
 */
public final class HudRenderer {
    private HudRenderer() {
    }

    public static void render(DrawContext context, float tickDelta) {
        if (!HudLayout.isEnabled()) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null || client.options.hudHidden) {
            return;
        }
        TextRenderer text = client.textRenderer;
        int width = context.getScaledWindowWidth();
        int height = context.getScaledWindowHeight();

        for (HudRow row : HudLayout.rows()) {
            int x = Math.round(width * row.x()) + row.offsetX();
            int y = Math.round(height * row.y()) + row.offsetY();
            // Right/bottom anchors need the row width before drawing.
            if (row.anchor().contains("right")) {
                x -= text.getWidth(label(client, row.id()));
            }
            if (row.anchor().contains("bottom") || row.anchor().contains("centre")) {
                y -= text.fontHeight;
            }
            context.drawTextWithShadow(text, label(client, row.id()), x, y, 0xFFFFFFFF);
        }
    }

    private static String label(MinecraftClient client, String id) {
        return switch (id) {
            case "fps" -> client.getCurrentFps() + " FPS";
            case "coords" -> client.player == null ? "" : String.format(
                    "%d %d %d",
                    (int) Math.floor(client.player.getX()),
                    (int) Math.floor(client.player.getY()),
                    (int) Math.floor(client.player.getZ()));
            case "ping" -> {
                PlayerListEntry entry = client.getNetworkHandler() == null || client.player == null
                        ? null
                        : client.getNetworkHandler().getPlayerListEntry(client.player.getUuid());
                yield entry == null ? "singleplayer" : entry.getLatency() + " ms";
            }
            case "server" -> client.getCurrentServerEntry() == null
                    ? "singleplayer"
                    : client.getCurrentServerEntry().address;
            case "keys" -> keystrokes(client);
            case "armor" -> {
                int armor = client.player == null ? 0 : client.player.getArmor();
                yield armor == 0 ? "" : "armor " + armor;
            }
            case "effects" -> {
                int count = client.player == null ? 0 : client.player.getStatusEffects().size();
                yield count == 0 ? "" : "effects " + count;
            }
            case "friends" -> FriendsPanel.onlineLabel();
            case "reach" -> "";
            default -> "";
        };
    }

    private static String keystrokes(MinecraftClient client) {
        StringBuilder builder = new StringBuilder();
        appendKey(builder, client.options.forwardKey);
        appendKey(builder, client.options.leftKey);
        appendKey(builder, client.options.backKey);
        appendKey(builder, client.options.rightKey);
        return builder.toString().trim();
    }

    private static void appendKey(StringBuilder builder, KeyBinding binding) {
        boolean pressed = binding.isPressed();
        String name = binding.getBoundKeyLocalizedText().getString();
        if (name.length() > 1) {
            name = name.substring(0, 1);
        }
        builder.append(pressed ? '[' : ' ').append(name).append(pressed ? ']' : ' ');
    }

    /** Exposed for the debug screen: the key names Rust expects. */
    public static String keyName(KeyBinding binding) {
        InputUtil.Key key = InputUtil.fromTranslationKey(binding.getBoundKeyTranslationKey());
        return key.getTranslationKey().replace("key.keyboard.", "").toUpperCase();
    }
}
