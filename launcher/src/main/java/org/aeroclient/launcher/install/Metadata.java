// SPDX-License-Identifier: MIT
package org.aeroclient.launcher.install;

import com.google.gson.*;
import java.io.IOException;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.time.Duration;
import java.util.*;

public final class Metadata {
    public static final URI VERSIONS = URI.create("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json");
    public static final URI RUNTIMES = URI.create("https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json");
    public static final List<String> TARGETS = List.of("1.8.9", "1.21.11", "26.2", "26.3");
    private final Downloads downloads;
    private final Path cache;
    public Metadata(Downloads downloads, Path cache) { this.downloads = downloads; this.cache = cache; }
    public JsonElement cached(String key, URI url, boolean refresh, Cancellation token) throws IOException, InterruptedException {
        Path file = SafeFiles.resolve(cache, key + ".json");
        if (!refresh && Files.exists(file) && System.currentTimeMillis() - Files.getLastModifiedTime(file).toMillis() < Duration.ofHours(6).toMillis()) {
            try { return parse(Files.readString(file)); } catch (IOException ignored) { /* Refetch corrupt cache. */ }
        }
        String json = new String(downloads.metadata(url, token), StandardCharsets.UTF_8);
        JsonElement parsed = parse(json);
        SafeFiles.write(file, json);
        return parsed;
    }
    public static JsonElement parse(String json) throws IOException {
        try {
            JsonElement parsed = JsonParser.parseString(json);
            if (!parsed.isJsonObject() && !parsed.isJsonArray()) throw new IOException("Expected JSON metadata object or array");
            return parsed;
        } catch (JsonParseException | IllegalStateException ex) { throw new IOException("Invalid metadata; refresh and retry", ex); }
    }
    public JsonObject versions(boolean refresh, Cancellation token) throws IOException, InterruptedException {
        return cached("versions", VERSIONS, refresh, token).getAsJsonObject();
    }
    public static JsonObject findVersion(JsonObject manifest, String version) throws IOException {
        for (JsonElement entry : manifest.getAsJsonArray("versions")) {
            JsonObject item = entry.getAsJsonObject();
            if (item.get("id").getAsString().equals(version)) return item;
        }
        throw new IOException("Minecraft " + version + " is unavailable in official metadata. Refresh the version list.");
    }
    public JsonObject version(String id, Cancellation token) throws IOException, InterruptedException {
        if (!TARGETS.contains(id)) throw new IOException("Version is outside the supported M2 targets");
        JsonObject entry = findVersion(versions(false, token), id);
        var artifact = new Downloads.Artifact(URI.create(entry.get("url").getAsString()), entry.get("sha1").getAsString(), -1);
        Path path = downloads.fetch(artifact, cache, "versions/" + id + ".json", token, p -> { });
        JsonObject version = parse(Files.readString(path)).getAsJsonObject();
        if (!id.equals(version.get("id").getAsString())) throw new IOException("Version metadata ID mismatch");
        return version;
    }
    public static Downloads.Artifact artifact(JsonObject object) {
        return new Downloads.Artifact(URI.create(object.get("url").getAsString()), object.get("sha1").getAsString(),
            object.has("size") ? object.get("size").getAsLong() : -1);
    }
}
