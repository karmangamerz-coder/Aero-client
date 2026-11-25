// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.client;

import java.util.Objects;

/** Metadata only: adapters implement behavior without depending on the launcher. */
public record ModuleDefinition(
        String id,
        String name,
        ModuleCategory category,
        String description,
        boolean enabledByDefault,
        boolean serverSafe) {
    public ModuleDefinition {
        if (id == null || !id.matches("[a-z][a-z0-9-]{1,47}")) {
            throw new IllegalArgumentException("Invalid module id");
        }
        if (name == null || name.isBlank() || name.length() > 64) {
            throw new IllegalArgumentException("Invalid module name");
        }
        Objects.requireNonNull(category, "category");
        if (description == null || description.length() > 240) {
            throw new IllegalArgumentException("Invalid module description");
        }
    }
}
