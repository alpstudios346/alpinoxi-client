package net.alpinoxi.hud;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.alpinoxi.bridge.BridgeLog;
import net.alpinoxi.bridge.Native;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import org.lwjgl.glfw.GLFW;

/**
 * Alpinoxi HUD: the mod's entry point.
 *
 * <p>Three registrations only: a key listener, a tick hook that refreshes the
 * layout, and the render callback. Every decision about what to show comes from
 * {@link HudLayout}, which comes from Rust.
 */
public final class AlpinoxiHud implements ClientModInitializer {
    private static KeyBinding openEditor;
    private static KeyBinding openFriends;
    private static KeyBinding openChat;

    @Override
    public void onInitializeClient() {
        Native.ensureLoaded();

        openEditor = KeyBindingHelper.registerKeyBinding(new KeyBinding(
                "key.alpinoxi.hud_editor", InputUtil.Type.KEYSYM, GLFW.GLFW_KEY_H,
                "category.alpinoxi"));
        openFriends = KeyBindingHelper.registerKeyBinding(new KeyBinding(
                "key.alpinoxi.friends", InputUtil.Type.KEYSYM, GLFW.GLFW_KEY_O,
                "category.alpinoxi"));
        openChat = KeyBindingHelper.registerKeyBinding(new KeyBinding(
                "key.alpinoxi.chat", InputUtil.Type.KEYSYM, GLFW.GLFW_KEY_Y,
                "category.alpinoxi"));

        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            HudLayout.tick();
            while (openEditor.wasPressed()) {
                client.setScreen(new HudEditorScreen());
            }
            while (openFriends.wasPressed()) {
                client.setScreen(new FriendsScreen());
            }
            while (openChat.wasPressed()) {
                ChatScreenBridge.focusChat();
            }
        });

        HudRenderCallback.EVENT.register((context, tickDelta) -> HudRenderer.render(context, tickDelta));
        BridgeLog.info("Alpinoxi HUD ready (bridge {})", Native.isLoaded() ? "native" : "fallback");
    }

    /** True when the native bridge answered at least once. */
    public static boolean bridgeReady() {
        return Native.isLoaded();
    }
}
