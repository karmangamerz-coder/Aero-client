// SPDX-License-Identifier: MIT
package org.aeroclient.launcher;
public record LauncherSettings(Theme theme, int memoryMiB) {
    public enum Theme { SYSTEM, LIGHT, DARK }
    public LauncherSettings {
        if (theme == null) throw new IllegalArgumentException("Theme is required");
        if (memoryMiB < 1024 || memoryMiB > 32768) {
            throw new IllegalArgumentException("Memory must be between 1024 and 32768 MiB");
        }
    }
    public static LauncherSettings defaults() { return new LauncherSettings(Theme.SYSTEM, 4096); }
}
