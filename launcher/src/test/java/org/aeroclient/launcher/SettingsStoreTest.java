// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.launcher;
import java.nio.file.*;
import java.io.IOException;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import static org.junit.jupiter.api.Assertions.*;
class SettingsStoreTest {
    @TempDir Path directory;
    @Test void missingSettingsReturnDefaultsWithoutWriting() throws Exception {
        Path file = directory.resolve("missing/settings.properties");
        assertEquals(LauncherSettings.defaults(), new SettingsStore(file).load());
        assertFalse(Files.exists(file));
    }
    @Test void savesAndReplacesPreferences() throws Exception {
        var store = new SettingsStore(directory.resolve("nested/settings.properties"));
        store.save(LauncherSettings.defaults());
        var changed = new LauncherSettings(LauncherSettings.Theme.DARK, 8192);
        store.save(changed);
        assertEquals(changed, store.load());
        try (var files = Files.list(directory.resolve("nested"))) { assertEquals(1, files.count()); }
    }
    @Test void corruptSettingsAreReported() throws Exception {
        Path file = directory.resolve("settings.properties");
        for (String content : new String[]{"memoryMiB=oops", "memoryMiB=1", "theme=UNKNOWN", "theme=\\uZZZZ"}) {
            Files.writeString(file, content);
            assertThrows(IOException.class, () -> new SettingsStore(file).load());
        }
    }
    @Test void invalidMemoryAndThemeAreRejected() {
        assertThrows(IllegalArgumentException.class, () -> new LauncherSettings(null, 4096));
        assertThrows(IllegalArgumentException.class, () -> new LauncherSettings(LauncherSettings.Theme.SYSTEM, 32769));
    }
}
