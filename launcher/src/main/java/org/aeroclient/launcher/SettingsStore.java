// SPDX-License-Identifier: MIT
package org.aeroclient.launcher;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.Properties;

/** Stores only non-sensitive preferences, never accounts or authentication tokens. */
public final class SettingsStore {
    private final Path file;
    public SettingsStore(Path file) { this.file = file.toAbsolutePath(); }
    public LauncherSettings load() throws IOException {
        if (!Files.exists(file)) return LauncherSettings.defaults();
        var values = new Properties();
        try {
            try (var reader = Files.newBufferedReader(file, StandardCharsets.UTF_8)) { values.load(reader); }
            return new LauncherSettings(LauncherSettings.Theme.valueOf(values.getProperty("theme", "SYSTEM")),
                Integer.parseInt(values.getProperty("memoryMiB", "4096")));
        } catch (IllegalArgumentException ex) { throw new IOException("Invalid local settings", ex); }
    }
    public void save(LauncherSettings settings) throws IOException {
        Files.createDirectories(file.getParent());
        Path temporary = Files.createTempFile(file.getParent(), ".aero-", ".tmp");
        try {
            Files.writeString(temporary, "theme=" + settings.theme() + "\nmemoryMiB=" + settings.memoryMiB() + "\n", StandardCharsets.UTF_8);
            try { Files.move(temporary, file, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING); }
            catch (AtomicMoveNotSupportedException ex) { Files.move(temporary, file, StandardCopyOption.REPLACE_EXISTING); }
        } finally { Files.deleteIfExists(temporary); }
    }
}
