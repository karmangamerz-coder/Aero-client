// SPDX-License-Identifier: MIT
package org.aeroclient.launcher.install;

import com.google.gson.*;
import java.io.*;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import java.util.function.Consumer;
import java.util.zip.ZipFile;

public final class Libraries {
    private Libraries() { }
    public static boolean allowed(JsonObject library) {
        if (library.has("clientreq") && !library.get("clientreq").getAsBoolean()) return false;
        if (!library.has("rules")) return true;
        boolean allowed = false;
        for (var value : library.getAsJsonArray("rules")) {
            JsonObject rule = value.getAsJsonObject(); boolean matches = true;
            if (rule.has("os")) {
                JsonObject os = rule.getAsJsonObject("os");
                if (os.has("name")) matches &= os.get("name").getAsString().equals("windows");
                if (os.has("arch")) matches &= "amd64".matches(os.get("arch").getAsString());
                if (os.has("version")) matches &= System.getProperty("os.version").matches(os.get("version").getAsString());
            }
            if (rule.has("features")) for (var feature : rule.getAsJsonObject("features").entrySet())
                matches &= !feature.getValue().getAsBoolean(); // No demo/custom-resolution/quick-play features in M2.
            if (matches) allowed = rule.get("action").getAsString().equals("allow");
        }
        return allowed;
    }
    public static String mavenPath(String coordinate) throws IOException {
        String[] parts = coordinate.split(":");
        if (parts.length < 3 || parts.length > 4 || Arrays.stream(parts).anyMatch(p -> !p.matches("[A-Za-z0-9_.+\\-]+")))
            throw new IOException("Unsupported Maven coordinate: " + coordinate);
        return parts[0].replace('.', '/') + "/" + parts[1] + "/" + parts[2] + "/" + parts[1] + "-" + parts[2] +
            (parts.length == 4 ? "-" + parts[3] : "") + ".jar";
    }
    public static Downloads.Artifact mavenArtifact(JsonObject library, Downloads downloads, Cancellation token) throws IOException, InterruptedException {
        String path = mavenPath(library.get("name").getAsString());
        URI base = URI.create(library.has("url") ? library.get("url").getAsString() : "https://libraries.minecraft.net/");
        URI url = base.resolve(path);
        if (library.has("checksums") && !library.getAsJsonArray("checksums").isEmpty()) {
            var hashes = new HashSet<String>(); library.getAsJsonArray("checksums").forEach(value -> hashes.add(value.getAsString()));
            return new Downloads.Artifact(url, library.getAsJsonArray("checksums").get(0).getAsString(), -1, hashes);
        }
        String hash;
        try {
            hash = library.has("sha1") ? library.get("sha1").getAsString() :
                new String(downloads.metadata(URI.create(url + ".sha1"), token), StandardCharsets.UTF_8).trim().split("\\s+")[0];
        } catch (IOException ex) { throw new IOException("No trusted checksum for " + library.get("name").getAsString() + ". Retry or select a different loader. " + ex.getMessage(), ex); }
        return new Downloads.Artifact(url, hash, library.has("size") ? library.get("size").getAsLong() : -1);
    }
    public static void extractNatives(Path archive, Path destination, Set<String> excludes, Cancellation token) throws IOException {
        try (var zip = new ZipFile(archive.toFile())) {
            var entries = zip.entries(); long total = 0; int count = 0;
            while (entries.hasMoreElements()) {
                token.check(); var entry = entries.nextElement();
                if (++count > 10000) throw new IOException("Native archive contains too many entries");
                if (entry.isDirectory() || entry.getName().startsWith("META-INF/") || excludes.stream().anyMatch(entry.getName()::startsWith)) continue;
                Path target = SafeFiles.resolve(destination, entry.getName());
                Files.createDirectories(target.getParent());
                Path temporary = Files.createTempFile(target.getParent(), ".native-", ".tmp");
                try (var input = zip.getInputStream(entry); var output = Files.newOutputStream(temporary)) {
                    byte[] buffer = new byte[65536]; int read;
                    while ((read = input.read(buffer)) != -1) {
                        token.check(); total += read;
                        if (total > 512L * 1024 * 1024) throw new IOException("Native archive exceeds extraction limit");
                        output.write(buffer, 0, read);
                    }
                } catch (Exception ex) { Files.deleteIfExists(temporary); throw ex; }
                SafeFiles.replace(temporary, target);
            }
        }
    }
    public static List<String> install(JsonArray libraries, Path shared, Path natives, Downloads downloads,
                                       Cancellation token, Consumer<Downloads.Progress> progress) throws IOException, InterruptedException {
        var paths = new ArrayList<String>();
        for (JsonElement value : libraries) {
            token.check(); JsonObject library = value.getAsJsonObject();
            if (!allowed(library)) continue;
            String path = mavenPath(library.get("name").getAsString());
            if (library.has("downloads")) {
                JsonObject spec = library.getAsJsonObject("downloads");
                if (spec.has("artifact")) {
                    JsonObject artifact = spec.getAsJsonObject("artifact");
                    path = artifact.has("path") ? artifact.get("path").getAsString() : path;
                    downloads.fetch(Metadata.artifact(artifact), shared, path, token, progress); paths.add(path);
                }
                if (library.has("natives") && library.getAsJsonObject("natives").has("windows")) {
                    String classifier = library.getAsJsonObject("natives").get("windows").getAsString().replace("${arch}", "64");
                    JsonObject nativeSpec = spec.getAsJsonObject("classifiers").getAsJsonObject(classifier);
                    Path jar = downloads.fetch(Metadata.artifact(nativeSpec), shared, nativeSpec.get("path").getAsString(), token, progress);
                    var excludes = new HashSet<String>();
                    if (library.has("extract") && library.getAsJsonObject("extract").has("exclude"))
                        library.getAsJsonObject("extract").getAsJsonArray("exclude").forEach(e -> excludes.add(e.getAsString()));
                    extractNatives(jar, natives, excludes, token);
                }
            } else {
                downloads.fetch(mavenArtifact(library, downloads, token), shared, path, token, progress); paths.add(path);
            }
        }
        return paths;
    }
}
