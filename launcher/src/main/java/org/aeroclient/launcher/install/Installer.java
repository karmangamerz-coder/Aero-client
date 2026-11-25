// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.launcher.install;

import com.google.gson.*;
import java.io.IOException;
import java.net.URI;
import java.nio.channels.*;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.CancellationException;
import java.util.function.Consumer;

public final class Installer implements AutoCloseable {
    private final Path root;
    private final Downloads downloads = new Downloads();
    private final Metadata metadata;
    private final Instances instances;
    public Installer(Path root) {
        this.root = root.toAbsolutePath().normalize();
        this.metadata = new Metadata(downloads, this.root.resolve("metadata"));
        this.instances = new Instances(this.root.resolve("instances"));
    }
    public Metadata metadata() { return metadata; }
    public Instances instances() { return instances; }
    public static void requireWindows() throws IOException {
        if (!System.getProperty("os.name").startsWith("Windows") || !Set.of("amd64", "x86_64").contains(System.getProperty("os.arch")))
            throw new IOException("M2 game installation supports Windows 10/11 x64 only. The launcher shell can still be built on other platforms.");
    }
    public Instances.Instance install(Instances.Instance instance, Path customJava, Cancellation token,
                                      Consumer<Downloads.Progress> progress, Consumer<String> message) throws IOException, InterruptedException {
        requireWindows();
        Files.createDirectories(root);
        try (var channel = FileChannel.open(SafeFiles.resolve(root, "installation.lock"), StandardOpenOption.CREATE, StandardOpenOption.WRITE)) {
            FileLock lock;
            try { lock = channel.tryLock(); } catch (OverlappingFileLockException ex) { throw new IOException("Another installation is active. Wait for it or cancel it first.", ex); }
            if (lock == null) throw new IOException("Another Aero process is installing. Close it and retry.");
            try (lock) { return perform(instance, customJava, token, progress, message); }
        }
    }
    private Instances.Instance perform(Instances.Instance instance, Path customJava, Cancellation token,
                                        Consumer<Downloads.Progress> progress, Consumer<String> message) throws IOException, InterruptedException {
        instances.status(instance, Instances.Status.INSTALLING, "Resolving verified metadata");
        try {
            token.check();
            Path directory = instances.directory(instance.id());
            message.accept("Resolving official Minecraft " + instance.version() + " metadata");
            JsonObject version = metadata.version(instance.version(), token);
            if (!version.has("javaVersion")) throw new IOException("Official metadata does not specify a Java runtime; refusing to guess");
            Path runtime = new Runtimes(downloads, metadata, root.resolve("runtimes"))
                .ensure(version.getAsJsonObject("javaVersion"), customJava, token, progress, message);
            JsonObject loader = new Loaders(downloads, metadata, root.resolve("metadata")).profile(instance, directory, token, progress);
            SafeFiles.write(SafeFiles.resolve(directory, "installation/version.json"), version.toString());
            if (loader != null) SafeFiles.write(SafeFiles.resolve(directory, "installation/loader.json"), loader.toString());
            message.accept("Verifying Minecraft client and libraries");
            var client = Metadata.artifact(version.getAsJsonObject("downloads").getAsJsonObject("client"));
            downloads.fetch(client, root.resolve("shared"), "clients/" + client.sha1() + ".jar", token, progress);
            Path natives = SafeFiles.resolve(directory, "installation/natives");
            var classpath = new ArrayList<>(Libraries.install(version.getAsJsonArray("libraries"), root.resolve("shared/libraries"), natives, downloads, token, progress));
            if (loader != null) classpath.addAll(Libraries.install(loader.getAsJsonArray("libraries"), root.resolve("shared/libraries"), natives, downloads, token, progress));
            message.accept("Verifying assets; existing verified objects are reused");
            var assetSpec = Metadata.artifact(version.getAsJsonObject("assetIndex"));
            Path indexFile = downloads.fetch(assetSpec, root.resolve("shared/assets"), "indexes/" + assetSpec.sha1() + ".json", token, progress);
            JsonObject index = Metadata.parse(Files.readString(indexFile)).getAsJsonObject();
            boolean virtual = index.has("virtual") && index.get("virtual").getAsBoolean();
            boolean resources = index.has("map_to_resources") && index.get("map_to_resources").getAsBoolean();
            var requiredAssets = new LinkedHashMap<String, Downloads.Request>();
            for (var entry : index.getAsJsonObject("objects").entrySet()) {
                JsonObject object = entry.getValue().getAsJsonObject(); String hash = object.get("hash").getAsString();
                if (!hash.matches("[0-9a-f]{40}")) throw new IOException("Invalid asset hash");
                requiredAssets.putIfAbsent(hash, new Downloads.Request(new Downloads.Artifact(
                    URI.create("https://resources.download.minecraft.net/" + hash.substring(0, 2) + "/" + hash), hash, object.get("size").getAsLong()),
                    root.resolve("shared/assets"), "objects/" + hash.substring(0, 2) + "/" + hash));
            }
            downloads.fetchAll(new ArrayList<>(requiredAssets.values()), token, progress);
            int completed = 0; int total = index.getAsJsonObject("objects").size();
            for (var entry : index.getAsJsonObject("objects").entrySet()) {
                token.check(); JsonObject object = entry.getValue().getAsJsonObject();
                String hash = object.get("hash").getAsString();
                if (!hash.matches("[0-9a-f]{40}")) throw new IOException("Invalid asset hash");
                String relative = "objects/" + hash.substring(0, 2) + "/" + hash;
                Path asset = SafeFiles.resolve(root.resolve("shared/assets"), relative);
                if (virtual || resources) {
                    Path mapping = SafeFiles.resolve(directory, (resources ? "game/resources/" : "installation/virtual-assets/") + entry.getKey());
                    Files.createDirectories(mapping.getParent());
                    if (Files.exists(mapping) && !Downloads.hash(mapping).equals(hash))
                        throw new IOException("An existing legacy resource differs from official data; it was preserved: " + mapping);
                    if (!Files.exists(mapping)) Files.copy(asset, mapping);
                }
                if (++completed % 50 == 0) message.accept("Assets verified: " + completed + " / " + total);
            }
            token.check();
            var plan = new JsonObject(); plan.addProperty("schema", 1); plan.addProperty("minecraft", instance.version());
            plan.addProperty("runtime", runtime.toString()); plan.addProperty("clientSha1", client.sha1());
            plan.addProperty("assetIndexSha1", assetSpec.sha1()); plan.addProperty("assetIndexId", version.getAsJsonObject("assetIndex").get("id").getAsString());
            plan.add("libraries", new Gson().toJsonTree(classpath));
            plan.addProperty("authenticationRequired", true); plan.addProperty("launchEnabled", false);
            SafeFiles.write(SafeFiles.resolve(directory, "installation/plan.json"), new GsonBuilder().setPrettyPrinting().create().toJson(plan));
            message.accept("Installation verified. Playing requires M3 authentication, which is not implemented.");
            return instances.status(instance, Instances.Status.READY, "Files verified; Play unavailable until M3 authentication");
        } catch (CancellationException ex) {
            instances.status(instance, Instances.Status.CANCELLED, "Cancelled; Retry or Repair resumes partial files"); throw ex;
        } catch (IOException | InterruptedException | RuntimeException ex) {
            instances.status(instance, Instances.Status.FAILED, "Installation failed; Retry or Repair after resolving the reported error"); throw ex;
        }
    }
    @Override public void close() { downloads.close(); }
}
