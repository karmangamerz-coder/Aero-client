// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.client;

import java.net.URI;
import java.time.Duration;
import java.util.Optional;

/** A snapshot pushed by an OS media adapter; the core never polls a music service. */
public record MusicState(
        String title, String artist, Optional<URI> artwork, Duration position,
        Duration duration, boolean playing, Controls controls) {
    public MusicState {
        title = title == null ? "" : title;
        artist = artist == null ? "" : artist;
        artwork = artwork == null ? Optional.empty() : artwork;
        position = position == null || position.isNegative() ? Duration.ZERO : position;
        duration = duration == null || duration.isNegative() ? Duration.ZERO : duration;
        controls = controls == null ? Controls.NONE : controls;
        if (!duration.isZero() && position.compareTo(duration) > 0) position = duration;
    }

    public record Controls(boolean previous, boolean playPause, boolean next) {
        public static final Controls NONE = new Controls(false, false, false);
    }
}
