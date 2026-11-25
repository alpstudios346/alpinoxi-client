package net.alpinoxi.hud;

import java.util.List;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

/**
 * The in-game HUD editor, opened with a keybind (Shift+H by default).
 *
 * <p>No commands, no config files: a row per element with a position cycler, and
 * a preview of the current layout behind it. Changes are applied to the bridge,
 * which is the same state the launcher writes.
 */
public final class HudEditorScreen extends Screen {
    private static final String[] ANCHORS = {
        "Top left", "Top right", "Bottom left", "Bottom right", "Bottom centre"
    };

    public HudEditorScreen() {
        super(Text.translatable("screen.alpinoxi.hud_editor"));
    }

    @Override
    protected void init() {
        List<HudRow> rows = HudLayout.rows();
        int y = 40;
        for (HudRow row : rows) {
            int index = indexOf(row.anchor());
            ButtonWidget button = ButtonWidget.builder(
                            Text.literal(row.label() + " · " + row.anchor()),
                            widget -> {
                                int next = (indexOf(currentAnchor(row.id())) + 1) % ANCHORS.length;
                                HudEditorBridge.setAnchor(row.id(), ANCHORS[next]);
                                widget.setMessage(Text.literal(row.label() + " · " + ANCHORS[next]));
                            })
                    .dimensions(this.width / 2 - 150, y, 300, 20)
                    .build();
            this.addDrawableChild(button);
            y += 24;
        }
        this.addDrawableChild(ButtonWidget.builder(
                        Text.translatable("gui.done"), widget -> this.close())
                .dimensions(this.width / 2 - 60, this.height - 32, 120, 20)
                .build());
    }

    private static int indexOf(String anchor) {
        String normalised = anchor.toLowerCase();
        for (int index = 0; index < ANCHORS.length; index++) {
            if (ANCHORS[index].toLowerCase().startsWith(normalised.split(" ")[0])) {
                return index;
            }
        }
        return 0;
    }

    private static String currentAnchor(String id) {
        return HudLayout.rows().stream()
                .filter(row -> row.id().equals(id))
                .map(HudRow::anchor)
                .findFirst()
                .orElse("Top left");
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        this.renderBackground(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(this.textRenderer, this.title, this.width / 2, 16, 0xFFFFFF);
        super.render(context, mouseX, mouseY, delta);
    }

    @Override
    public boolean shouldPause() {
        return false;
    }
}
