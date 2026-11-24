// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.launcher.install;

import com.google.gson.*;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;

public final class Instances {
    public enum Loader { VANILLA, FABRIC, FORGE }
    public enum Status { NEW, INSTALLING, READY, CANCELLED, FAILED, INTERRUPTED }
    public record Instance(String id, String name, String version, Loader loader, Status status, String detail) {
        @Override public String toString() { return name + " · " + version + " · " + status; }
    }
    private static final Gson JSON = new GsonBuilder().setPrettyPrinting().create();
    private final Path root;
    public Instances(Path root) { this.root = root; }
    public Instance create(String name, String version, Loader loader, boolean defaultInstance) throws IOException {
        if (name == null || name.isBlank() || name.length() > 80 || name.chars().anyMatch(Character::isISOControl))
            throw new IOException("Use an instance name of 1–80 printable characters");
        if (!Metadata.TARGETS.contains(version)) throw new IOException("Select a supported Minecraft version");
        if (loader == Loader.FORGE && !version.equals("1.8.9")) throw new IOException("Legacy Forge is supported only for 1.8.9");
        if (loader == Loader.FABRIC && version.equals("1.8.9")) throw new IOException("Use Forge or Vanilla for 1.8.9");
        String id = (defaultInstance ? UUID.nameUUIDFromBytes(("aero-default:" + version + ":" + loader).getBytes(StandardCharsets.UTF_8)) : UUID.randomUUID()).toString();
        Path directory = directory(id);
        if (Files.exists(directory.resolve("instance.json"))) return read(id);
        Files.createDirectories(directory);
        for (String folder : List.of("saves", "config", "mods", "resourcepacks", "shaderpacks"))
            Files.createDirectories(SafeFiles.resolve(directory, "game/" + folder));
        var instance = new Instance(id, name.trim(), version, loader, Status.NEW, "Not installed");
        save(instance); return instance;
    }
    public Path directory(String id) throws IOException {
        if (!UUID.fromString(id).toString().equals(id)) throw new IOException("Invalid instance ID");
        return SafeFiles.resolve(root, id);
    }
    public void save(Instance instance) throws IOException {
        var object = JSON.toJsonTree(instance).getAsJsonObject(); object.addProperty("schema", 1);
        SafeFiles.write(SafeFiles.resolve(directory(instance.id()), "instance.json"), JSON.toJson(object));
    }
    public Instance status(Instance instance, Status status, String detail) throws IOException {
        var updated = new Instance(instance.id(), instance.name(), instance.version(), instance.loader(), status, detail);
        save(updated); return updated;
    }
    public Instance read(String id) throws IOException {
        try {
            JsonObject object = Metadata.parse(Files.readString(SafeFiles.resolve(directory(id), "instance.json"))).getAsJsonObject();
            if (object.get("schema").getAsInt() != 1) throw new IOException("Unsupported instance schema");
            Instance instance = JSON.fromJson(object, Instance.class);
            if (!id.equals(instance.id()) || !Metadata.TARGETS.contains(instance.version()) || instance.loader() == null ||
                instance.status() == null || instance.name() == null || instance.name().length() > 80)
                throw new IOException("Invalid instance manifest");
            return instance.status() == Status.INSTALLING ? new Instance(id, instance.name(), instance.version(), instance.loader(), Status.INTERRUPTED, "Interrupted installation; Repair to resume") : instance;
        } catch (IllegalArgumentException | IllegalStateException | NullPointerException ex) {
            throw new IOException("Invalid instance manifest: " + id, ex);
        }
    }
    public List<Instance> list() throws IOException {
        if (!Files.exists(root)) return List.of();
        var results = new ArrayList<Instance>();
        try (var entries = Files.list(root)) {
            for (Path path : entries.filter(Files::isDirectory).toList()) {
                if (path.getFileName().toString().matches("[0-9a-f-]{36}")) results.add(read(path.getFileName().toString()));
            }
        }
        return List.copyOf(results);
    }
}
