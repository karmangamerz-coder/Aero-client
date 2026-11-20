// SPDX-License-Identifier: MIT
package org.aeroclient.launcher.install;

import com.google.gson.*;
import java.io.IOException;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.function.Consumer;
import java.util.zip.ZipFile;

public final class Loaders {
    private final Downloads downloads;
    private final Metadata metadata;
    private final Path cache;
    public Loaders(Downloads downloads, Metadata metadata, Path cache) { this.downloads = downloads; this.metadata = metadata; this.cache = cache; }
    public JsonObject profile(Instances.Instance instance, Path directory, Cancellation token,
                              Consumer<Downloads.Progress> progress) throws IOException, InterruptedException {
        if (instance.loader() == Instances.Loader.VANILLA) return null;
        if (instance.loader() == Instances.Loader.FABRIC) {
            if (instance.version().equals("1.8.9")) throw new IOException("Fabric does not support the selected legacy target");
            Path pin = SafeFiles.resolve(directory, "fabric-version.txt");
            String loaderVersion;
            if (Files.exists(pin)) loaderVersion = Files.readString(pin).trim();
            else {
                JsonArray available = metadata.cached("fabric-" + instance.version(),
                    URI.create("https://meta.fabricmc.net/v2/versions/loader/" + instance.version()), false, token).getAsJsonArray();
                loaderVersion = null;
                for (JsonElement element : available) {
                    JsonObject loader = element.getAsJsonObject().getAsJsonObject("loader");
                    if (loader.get("stable").getAsBoolean()) { loaderVersion = loader.get("version").getAsString(); break; }
                }
                if (loaderVersion == null) throw new IOException("No stable Fabric loader is available for " + instance.version() + ". Choose Vanilla.");
            }
            if (!loaderVersion.matches("[0-9]+\\.[0-9]+\\.[0-9]+")) throw new IOException("Invalid pinned Fabric loader version");
            JsonObject result = metadata.cached("fabric-profile-" + instance.version() + "-" + loaderVersion,
                URI.create("https://meta.fabricmc.net/v2/versions/loader/" + instance.version() + "/" + loaderVersion + "/profile/json"), false, token).getAsJsonObject();
            if (!instance.version().equals(result.get("inheritsFrom").getAsString())) throw new IOException("Fabric profile version mismatch");
            SafeFiles.write(pin, loaderVersion + "\n");
            return result;
        }
        if (!instance.version().equals("1.8.9")) throw new IOException("Legacy Forge installer is restricted to Minecraft 1.8.9");
        String version = "1.8.9-11.15.1.2318-1.8.9";
        String base = "https://maven.minecraftforge.net/net/minecraftforge/forge/" + version + "/forge-" + version;
        URI url = URI.create(base + "-installer.jar");
        String hash = new String(downloads.metadata(URI.create(url + ".sha1"), token), StandardCharsets.UTF_8).trim().split("\\s+")[0];
        Path installer = downloads.fetch(new Downloads.Artifact(url, hash, -1), cache, "forge/" + version + "-installer.jar", token, progress);
        JsonObject profile;
        // Read data only. Never execute third-party installers or their processors.
        try (var zip = new ZipFile(installer.toFile())) {
            var entry = zip.getEntry("install_profile.json");
            if (entry == null) throw new IOException("Forge installer has no legacy install profile");
            try (var stream = zip.getInputStream(entry)) {
                byte[] bytes = stream.readNBytes(4 * 1024 * 1024 + 1);
                if (bytes.length > 4 * 1024 * 1024) throw new IOException("Forge profile is too large");
                profile = Metadata.parse(new String(bytes, StandardCharsets.UTF_8)).getAsJsonObject().getAsJsonObject("versionInfo");
            }
        }
        if (profile == null || !instance.version().equals(profile.get("inheritsFrom").getAsString())) throw new IOException("Unsupported Forge installer profile");
        String universalHash = new String(downloads.metadata(URI.create(base + "-universal.jar.sha1"), token), StandardCharsets.UTF_8).trim().split("\\s+")[0];
        for (var item : profile.getAsJsonArray("libraries")) {
            JsonObject library = item.getAsJsonObject();
            if (library.get("name").getAsString().equals("net.minecraftforge:forge:" + version)) {
                var artifact = new JsonObject();
                artifact.addProperty("url", base + "-universal.jar"); artifact.addProperty("sha1", universalHash);
                artifact.addProperty("path", Libraries.mavenPath(library.get("name").getAsString()));
                var spec = new JsonObject(); spec.add("artifact", artifact); library.add("downloads", spec);
            }
        }
        return profile;
    }
}
