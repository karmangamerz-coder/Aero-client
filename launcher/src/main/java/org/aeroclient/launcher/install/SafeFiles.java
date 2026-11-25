// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.launcher.install;

import java.io.IOException;
import java.nio.file.*;
import java.util.Locale;

public final class SafeFiles {
    private SafeFiles() { }
    public static Path resolve(Path root, String relative) throws IOException {
        if (relative.isBlank() || relative.contains("\\") || relative.contains(":") || relative.startsWith("/"))
            throw new IOException("Unsafe installation path: " + relative);
        for (String part : relative.split("/")) {
            String base = part.split("\\.", 2)[0].toUpperCase(Locale.ROOT);
            if (part.isBlank() || part.equals(".") || part.equals("..") || part.endsWith(".") || part.endsWith(" ") ||
                part.matches(".*[<>\"|?*\\x00-\\x1f].*") || base.matches("CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9]"))
                throw new IOException("Unsafe installation path: " + relative);
        }
        Path normalized = root.toAbsolutePath().normalize();
        Files.createDirectories(normalized);
        Path target = normalized.resolve(relative).normalize();
        if (!target.startsWith(normalized)) throw new IOException("Path escapes installation directory");
        Path realRoot = normalized.toRealPath();
        for (Path current = target; current != null && current.startsWith(normalized); current = current.getParent()) {
            if (Files.exists(current, LinkOption.NOFOLLOW_LINKS) &&
                (Files.isSymbolicLink(current) || !current.toRealPath().startsWith(realRoot)))
                throw new IOException("Linked installation paths are not allowed: " + current);
        }
        return target;
    }
    public static void replace(Path temporary, Path target) throws IOException {
        try { Files.move(temporary, target, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING); }
        catch (AtomicMoveNotSupportedException ex) { Files.move(temporary, target, StandardCopyOption.REPLACE_EXISTING); }
    }
    public static void write(Path target, String text) throws IOException {
        Files.createDirectories(target.getParent());
        Path temporary = Files.createTempFile(target.getParent(), ".aero-", ".tmp");
        try { Files.writeString(temporary, text); replace(temporary, target); }
        finally { Files.deleteIfExists(temporary); }
    }
}
