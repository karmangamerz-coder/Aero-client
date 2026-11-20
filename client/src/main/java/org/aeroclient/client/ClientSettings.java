// SPDX-License-Identifier: MIT
package org.aeroclient.client;

/** Platform-neutral customization defaults; adapters consume these in a later milestone. */
public record ClientSettings(boolean showFps, boolean reducedMotion, int hudScalePercent) {
    public ClientSettings {
        if (hudScalePercent < 50 || hudScalePercent > 200) {
            throw new IllegalArgumentException("HUD scale must be between 50 and 200 percent");
        }
    }
    public static ClientSettings defaults() { return new ClientSettings(true, false, 100); }
}
