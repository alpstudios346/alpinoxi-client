package net.alpinoxi.bridge;

import net.fabricmc.api.ClientModInitializer;

/**
 * Entry point of the bridge mod.
 *
 * <p>It does one job: make sure the native library is loaded before any other
 * Alpinoxi mod asks for a layout. The other mods call
 * {@link Native#ensureLoaded()} themselves too, so load order never matters.
 */
public final class AlpinoxiBridge implements ClientModInitializer {
    @Override
    public void onInitializeClient() {
        boolean ready = Native.ensureLoaded();
        if (ready) {
            BridgeLog.info("Alpinoxi bridge {} loaded", Native.bridgeVersion());
        } else {
            BridgeLog.warn("Alpinoxi bridge not loaded: {}", Native.loadError());
        }
    }
}
