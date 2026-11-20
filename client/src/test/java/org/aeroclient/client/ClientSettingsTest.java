// SPDX-License-Identifier: MIT
package org.aeroclient.client;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;
class ClientSettingsTest {
    @Test void defaultsAreUsable() { assertEquals(100, ClientSettings.defaults().hudScalePercent()); }
    @Test void acceptsBoundaryScales() {
        assertDoesNotThrow(() -> new ClientSettings(false, true, 50));
        assertDoesNotThrow(() -> new ClientSettings(true, false, 200));
    }
    @Test void rejectsUnusableScales() {
        assertThrows(IllegalArgumentException.class, () -> new ClientSettings(true, false, 49));
        assertThrows(IllegalArgumentException.class, () -> new ClientSettings(true, false, 201));
    }
}
