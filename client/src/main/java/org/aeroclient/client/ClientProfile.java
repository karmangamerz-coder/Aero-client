// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.client;

import java.util.LinkedHashMap;
import java.util.Map;

/** Immutable profile shared by mod-menu, HUD editor and version adapters. */
public record ClientProfile(String name, Map<String, Boolean> enabled, HudLayout hud) {
    public ClientProfile {
        if (name == null || name.isBlank() || name.length() > 64 || name.chars().anyMatch(Character::isISOControl)) {
            throw new IllegalArgumentException("Invalid profile name");
        }
        enabled = Map.copyOf(enabled == null ? Map.of() : new LinkedHashMap<>(enabled));
        hud = hud == null ? new HudLayout(Map.of()) : hud;
    }

    public boolean isEnabled(ModuleDefinition module) {
        return enabled.getOrDefault(module.id(), module.enabledByDefault());
    }

    public ClientProfile toggle(String id, ModuleRegistry registry) {
        var module = registry.find(id).orElseThrow(() -> new IllegalArgumentException("Unknown module: " + id));
        var updated = new LinkedHashMap<>(enabled);
        updated.put(id, !isEnabled(module));
        return new ClientProfile(name, updated, hud);
    }

    public static ClientProfile defaults() { return new ClientProfile("Default", Map.of(), new HudLayout(Map.of())); }
}
