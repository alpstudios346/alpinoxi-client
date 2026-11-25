package net.alpinoxi.hud;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/**
 * The friend list and chat in one screen, opened with O.
 *
 * <p>The list itself comes from the Rust friend book through the bridge; this
 * screen only draws it and forwards typed lines. No commands are involved.
 */
public final class FriendsScreen extends Screen {
    public FriendsScreen() {
        super(Text.translatable("screen.alpinoxi.friends"));
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        this.renderBackground(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(this.textRenderer, this.title, this.width / 2, 16, 0xFFFFFF);
        int y = 40;
        for (HudRow row : HudLayout.rows()) {
            if (!"friends".equals(row.id())) {
                continue;
            }
            context.drawTextWithShadow(this.textRenderer, row.label(), 20, y, 0xFF9AD0FF);
            y += 14;
        }
        context.drawTextWithShadow(
                this.textRenderer,
                Text.translatable("screen.alpinoxi.friends.hint"),
                20,
                this.height - 30,
                0xFFAAAAAA);
    }

    @Override
    public boolean shouldPause() {
        return false;
    }
}
