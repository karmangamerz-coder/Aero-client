// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.client;

import java.util.LinkedHashMap;
import java.util.Map;

/** Resolution-independent HUD positions; adapters translate these to pixels. */
public record HudLayout(Map<String, Placement> placements) {
    public HudLayout {
        placements = Map.copyOf(placements == null ? Map.of() : new LinkedHashMap<>(placements));
    }

    public HudLayout move(String moduleId, float x, float y, float scale) {
        var updated = new LinkedHashMap<>(placements);
        updated.put(moduleId, new Placement(x, y, scale));
        return new HudLayout(updated);
    }

    public record Placement(float x, float y, float scale) {
        public Placement {
            if (!Float.isFinite(x) || !Float.isFinite(y) || x < 0 || x > 1 || y < 0 || y > 1) {
                throw new IllegalArgumentException("HUD coordinates must be normalized");
            }
            if (!Float.isFinite(scale) || scale < .5f || scale > 2f) {
                throw new IllegalArgumentException("HUD scale must be 0.5–2.0");
            }
        }
    }
}
