package net.alpinoxi.hud;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.ChatScreen;

/**
 * Opens the game's chat box from a keybind.
 *
 * <p>Alpinoxi keeps the vanilla chat widget, because players expect `T` and `/`
 * to behave exactly as before; the difference is that Alpinoxi also routes
 * friend messages into tabs through the Rust chat state.
 */
public final class ChatScreenBridge {
    private ChatScreenBridge() {
    }

    public static void focusChat() {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.currentScreen == null) {
            client.setScreen(new ChatScreen(""));
        }
    }
}
