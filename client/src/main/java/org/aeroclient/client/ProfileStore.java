// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.client;

import java.io.IOException;
import java.io.Reader;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.LinkedHashMap;
import java.util.Properties;
import java.util.UUID;

/** Atomic, dependency-free profile persistence for version-specific adapters. */
public final class ProfileStore {
    private ProfileStore() {}

    public static void save(Path file, ClientProfile profile, ModuleRegistry registry) throws IOException {
        var properties = new Properties();
        properties.setProperty("schema", "1");
        properties.setProperty("name", profile.name());
        for (var module : registry.all()) {
            properties.setProperty("module." + module.id(), Boolean.toString(profile.isEnabled(module)));
        }
        for (var entry : profile.hud().placements().entrySet()) {
            var placement = entry.getValue();
            properties.setProperty("hud." + entry.getKey(),
                    placement.x() + "," + placement.y() + "," + placement.scale());
        }
        var parent = file.toAbsolutePath().getParent();
        if (parent == null) throw new IOException("Profile path has no parent");
        Files.createDirectories(parent);
        var temporary = parent.resolve(file.getFileName() + "." + UUID.randomUUID() + ".tmp");
        try (Writer writer = Files.newBufferedWriter(temporary, StandardCharsets.UTF_8)) {
            properties.store(writer, "Aero Client profile");
        }
        try {
            Files.move(temporary, file, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING);
        } catch (java.nio.file.AtomicMoveNotSupportedException ignored) {
            Files.move(temporary, file, StandardCopyOption.REPLACE_EXISTING);
        }
    }

    public static ClientProfile load(Path file, ModuleRegistry registry) throws IOException {
        if (!Files.exists(file)) return ClientProfile.defaults();
        var properties = new Properties();
        try (Reader reader = Files.newBufferedReader(file, StandardCharsets.UTF_8)) {
            properties.load(reader);
        }
        if (!"1".equals(properties.getProperty("schema"))) throw new IOException("Unsupported profile schema");
        var enabled = new LinkedHashMap<String, Boolean>();
        for (var module : registry.all()) {
            var value = properties.getProperty("module." + module.id());
            if (value != null) {
                if (!value.equals("true") && !value.equals("false")) throw new IOException("Invalid module state");
                enabled.put(module.id(), Boolean.parseBoolean(value));
            }
        }
        var placements = new LinkedHashMap<String, HudLayout.Placement>();
        for (var key : properties.stringPropertyNames()) {
            if (!key.startsWith("hud.")) continue;
            var id = key.substring(4);
            if (registry.find(id).isEmpty()) continue;
            var parts = properties.getProperty(key).split(",", -1);
            if (parts.length != 3) throw new IOException("Invalid HUD placement");
            try {
                placements.put(id, new HudLayout.Placement(
                        Float.parseFloat(parts[0]), Float.parseFloat(parts[1]), Float.parseFloat(parts[2])));
            } catch (IllegalArgumentException error) {
                throw new IOException("Invalid HUD placement", error);
            }
        }
        try {
            return new ClientProfile(properties.getProperty("name", "Default"), enabled, new HudLayout(placements));
        } catch (IllegalArgumentException error) {
            throw new IOException("Invalid profile", error);
        }
    }
}
