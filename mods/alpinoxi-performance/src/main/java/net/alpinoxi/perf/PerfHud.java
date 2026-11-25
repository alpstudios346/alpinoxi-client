package net.alpinoxi.perf;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.text.Text;

/** Draws the measured numbers. One line, no decoration, so it never distracts. */
public final class PerfHud {
    private PerfHud() {
    }

    public static void render(DrawContext context, float tickDelta) {
        if (!PerfConfig.showHud()) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null || client.options.hudHidden) {
            return;
        }
        String line = String.format(
                "%d fps · 1%% %.0f · %.1f ms",
                Math.round(FrameMeter.averageFps()),
                FrameMeter.onePercentLowFps(),
                FrameMeter.percentileMillis(99.0f));
        int x = 4;
        int y = context.getScaledWindowHeight() - 14;
        context.drawTextWithShadow(client.textRenderer, Text.literal(line), x, y, 0xFF8CE0C0);
    }
}
