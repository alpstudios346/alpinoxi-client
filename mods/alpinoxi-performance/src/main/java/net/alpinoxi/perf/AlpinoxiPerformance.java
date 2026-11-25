package net.alpinoxi.perf;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.alpinoxi.bridge.BridgeLog;
import net.alpinoxi.bridge.Native;

/**
 * Alpinoxi Performance: measure, then recommend.
 *
 * <p>This mod does not patch rendering. Sodium already does that better, and two
 * mods fighting over the same pipeline is how FPS gets *worse*. What it adds is
 * the part clients usually get wrong: honest frame-time numbers, a diagnosis that
 * names the real bottleneck, and a render-distance governor that only moves one
 * chunk at a time.
 */
public final class AlpinoxiPerformance implements ClientModInitializer {
    private static long lastFrameNanos;
    private static int ticks;

    @Override
    public void onInitializeClient() {
        Native.ensureLoaded();

        HudRenderCallback.EVENT.register((context, tickDelta) -> {
            long now = System.nanoTime();
            if (lastFrameNanos != 0L) {
                FrameMeter.record((now - lastFrameNanos) / 1_000_000.0f);
            }
            lastFrameNanos = now;
            PerfHud.render(context, tickDelta);
        });

        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            if (++ticks % 40 == 0) {
                PerfReport.refresh();
            }
        });

        BridgeLog.info("Alpinoxi Performance ready (bridge {})", Native.isLoaded() ? "native" : "fallback");
    }
}
