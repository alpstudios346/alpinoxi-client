package net.alpinoxi.perf;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

/**
 * The mod's own settings screen, reachable from Mod Menu.
 *
 * <p>It shows the live numbers, the diagnosis and the two switches. Everything
 * that decides *what* to recommend lives in the Rust core; this screen only asks.
 */
public final class PerfScreen extends Screen {
    public PerfScreen() {
        super(Text.translatable("screen.alpinoxi.perf"));
    }

    @Override
    protected void init() {
        this.addDrawableChild(ButtonWidget.builder(
                        Text.literal("HUD: " + (PerfConfig.showHud() ? "on" : "off")),
                        widget -> {
                            PerfConfig.setHud(!PerfConfig.showHud());
                            widget.setMessage(Text.literal("HUD: " + (PerfConfig.showHud() ? "on" : "off")));
                        })
                .dimensions(this.width / 2 - 150, 70, 300, 20)
                .build());
        this.addDrawableChild(ButtonWidget.builder(
                        Text.literal("Render governor: " + (PerfConfig.governorEnabled() ? "on" : "off")),
                        widget -> {
                            PerfConfig.setGovernor(!PerfConfig.governorEnabled());
                            widget.setMessage(Text.literal(
                                    "Render governor: " + (PerfConfig.governorEnabled() ? "on" : "off")));
                        })
                .dimensions(this.width / 2 - 150, 94, 300, 20)
                .build());
        this.addDrawableChild(ButtonWidget.builder(Text.translatable("gui.done"), widget -> this.close())
                .dimensions(this.width / 2 - 60, this.height - 32, 120, 20)
                .build());
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        this.renderBackground(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(this.textRenderer, this.title, this.width / 2, 20, 0xFFFFFF);
        String stats = String.format(
                "avg %.0f fps · 1%% low %.0f fps · worst %.1f ms",
                FrameMeter.averageFps(),
                FrameMeter.onePercentLowFps(),
                FrameMeter.percentileMillis(99.9f));
        context.drawCenteredTextWithShadow(this.textRenderer, Text.literal(stats), this.width / 2, 40, 0xFFB0D8FF);
        context.drawCenteredTextWithShadow(
                this.textRenderer,
                Text.literal(PerfReport.diagnosisLine()),
                this.width / 2,
                54,
                0xFF9AE6B4);
        super.render(context, mouseX, mouseY, delta);
    }
}
