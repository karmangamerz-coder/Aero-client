// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.client;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/** Original, server-safe Aero modules. No packet manipulation or anti-cheat bypasses. */
public final class ModuleRegistry {
    private final Map<String, ModuleDefinition> modules;

    public ModuleRegistry(List<ModuleDefinition> definitions) {
        var ordered = new LinkedHashMap<String, ModuleDefinition>();
        for (var definition : definitions) {
            if (ordered.putIfAbsent(definition.id(), definition) != null) {
                throw new IllegalArgumentException("Duplicate module: " + definition.id());
            }
        }
        modules = Map.copyOf(ordered);
    }

    public List<ModuleDefinition> all() { return List.copyOf(modules.values()); }
    public Optional<ModuleDefinition> find(String id) { return Optional.ofNullable(modules.get(id)); }

    public static ModuleRegistry builtins() {
        return new ModuleRegistry(List.of(
                module("fps", "FPS", ModuleCategory.HUD, "Current rendered frames per second."),
                module("cps", "CPS", ModuleCategory.HUD, "Recent primary and secondary clicks."),
                module("keystrokes", "Keystrokes", ModuleCategory.HUD, "Movement and mouse input state."),
                module("ping", "Ping", ModuleCategory.HUD, "Current server latency when available."),
                module("armor", "Armor", ModuleCategory.HUD, "Equipped armor and durability."),
                module("potions", "Effects", ModuleCategory.HUD, "Active effects and remaining time."),
                module("coordinates", "Coordinates", ModuleCategory.HUD, "Player position."),
                module("direction", "Direction", ModuleCategory.HUD, "Facing direction and heading."),
                module("toggle-sprint", "Toggle Sprint", ModuleCategory.QUALITY_OF_LIFE, "Keeps sprint input active until toggled."),
                module("toggle-sneak", "Toggle Sneak", ModuleCategory.QUALITY_OF_LIFE, "Keeps sneak input active until toggled."),
                module("zoom", "Zoom", ModuleCategory.QUALITY_OF_LIFE, "Temporary client-side field-of-view adjustment."),
                module("crosshair", "Crosshair", ModuleCategory.VISUAL, "Customizable local crosshair."),
                module("chat", "Chat", ModuleCategory.VISUAL, "Local chat appearance controls."),
                module("music", "Now Playing", ModuleCategory.MEDIA, "Event-driven operating-system media overlay.")));
    }

    private static ModuleDefinition module(String id, String name, ModuleCategory category, String description) {
        return new ModuleDefinition(id, name, category, description, id.equals("fps"), true);
    }
}
