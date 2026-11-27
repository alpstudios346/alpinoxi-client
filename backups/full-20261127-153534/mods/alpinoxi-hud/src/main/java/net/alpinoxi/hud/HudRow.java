package net.alpinoxi.hud;

/**
 * One HUD element's resolved position.
 *
 * <p>Parsed from the JSON the Rust bridge returns, so the renderer never
 * recomputes an anchor: it just draws at {@link #x()}, {@link #y()} plus the
 * pixel offset.
 */
public record HudRow(String id, String label, String anchor, float x, float y, int offsetX, int offsetY) {
}
