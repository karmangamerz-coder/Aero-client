// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.client;

import org.junit.jupiter.api.Test;
import java.time.Duration;
import java.util.Map;
import java.util.Optional;
import static org.junit.jupiter.api.Assertions.*;

class ClientFoundationTest {
    @Test void builtinsAreUniqueAndServerSafe() {
        var modules = ModuleRegistry.builtins().all();
        assertEquals(14, modules.size());
        assertEquals(modules.size(), modules.stream().map(ModuleDefinition::id).distinct().count());
        assertTrue(modules.stream().allMatch(ModuleDefinition::serverSafe));
    }

    @Test void profilesToggleKnownModulesOnly() {
        var registry = ModuleRegistry.builtins();
        var profile = ClientProfile.defaults();
        assertTrue(profile.isEnabled(registry.find("fps").orElseThrow()));
        assertFalse(profile.toggle("fps", registry).isEnabled(registry.find("fps").orElseThrow()));
        assertThrows(IllegalArgumentException.class, () -> profile.toggle("reach", registry));
    }

    @Test void hudPositionsStayOnScreen() {
        var layout = new HudLayout(Map.of()).move("fps", .9f, .1f, 1.25f);
        assertEquals(.9f, layout.placements().get("fps").x());
        assertThrows(IllegalArgumentException.class, () -> layout.move("fps", 1.1f, 0, 1));
    }

    @Test void mediaProgressIsClamped() {
        var state = new MusicState("Track", "Artist", Optional.empty(), Duration.ofSeconds(20),
                Duration.ofSeconds(10), true, MusicState.Controls.NONE);
        assertEquals(Duration.ofSeconds(10), state.position());
    }

    @Test void profilesRoundTripAtomically() throws Exception {
        var directory = java.nio.file.Files.createTempDirectory("aero-profile");
        var file = directory.resolve("default.properties");
        var registry = ModuleRegistry.builtins();
        var expected = ClientProfile.defaults().toggle("fps", registry);
        ProfileStore.save(file, expected, registry);
        var actual = ProfileStore.load(file, registry);
        assertEquals(expected.name(), actual.name());
        assertEquals(expected.hud(), actual.hud());
        for (var module : registry.all()) {
            assertEquals(expected.isEnabled(module), actual.isEnabled(module));
        }
    }
}
