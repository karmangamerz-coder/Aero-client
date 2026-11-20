// SPDX-License-Identifier: MIT
package org.aeroclient.launcher.install;

import com.google.gson.*;
import java.io.IOException;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.TimeUnit;
import java.util.function.Consumer;
import java.util.regex.Pattern;

public final class Runtimes {
    private final Downloads downloads;
    private final Metadata metadata;
    private final Path root;
    public Runtimes(Downloads downloads, Metadata metadata, Path root) { this.downloads = downloads; this.metadata = metadata; this.root = root; }
    public record JavaInfo(Path executable, int major, boolean x64) { }
    public static int parseMajor(String version) {
        String value = version.startsWith("1.") ? version.substring(2) : version;
        return Integer.parseInt(value.split("[._+\\-]", 2)[0]);
    }
    public static JavaInfo probe(Path executable) throws IOException, InterruptedException {
        if (!Files.isRegularFile(executable)) throw new IOException("Java executable does not exist");
        Path output = Files.createTempFile("aero-java-check-", ".txt");
        Process process = null;
        try {
            process = new ProcessBuilder(executable.toAbsolutePath().toString(), "-XshowSettings:properties", "-version")
                .redirectErrorStream(true).redirectOutput(output.toFile()).start();
            if (!process.waitFor(8, TimeUnit.SECONDS)) throw new IOException("Java version check timed out");
            if (Files.size(output) > 65536) throw new IOException("Unexpected Java version output");
            String text = Files.readString(output);
            var version = Pattern.compile("java.version\\s*=\\s*([^\\s]+)").matcher(text);
            var arch = Pattern.compile("os.arch\\s*=\\s*(amd64|x86_64)(?:\\s|$)").matcher(text);
            if (process.exitValue() != 0 || !version.find()) throw new IOException("Could not identify selected Java runtime");
            return new JavaInfo(executable.toAbsolutePath(), parseMajor(version.group(1)), arch.find());
        } finally {
            if (process != null && process.isAlive()) { process.destroyForcibly(); process.waitFor(2, TimeUnit.SECONDS); }
            Files.deleteIfExists(output);
        }
    }
    public static List<Path> candidates() {
        var result = new LinkedHashSet<Path>();
        for (String name : List.of("JAVA_HOME", "JDK_HOME")) {
            String home = System.getenv(name); if (home != null) result.add(Path.of(home, "bin", "java.exe"));
        }
        String path = System.getenv("PATH");
        if (path != null) for (String item : path.split(java.io.File.pathSeparator)) {
            try { if (!item.isBlank()) result.add(Path.of(item, "java.exe")); } catch (InvalidPathException ignored) { }
        }
        for (String location : List.of("C:/Program Files/Java", "C:/Program Files/Eclipse Adoptium", "C:/Program Files/Microsoft")) {
            Path folder = Path.of(location);
            if (Files.isDirectory(folder)) try (var homes = Files.list(folder)) {
                homes.filter(Files::isDirectory).limit(30).forEach(home -> result.add(home.resolve("bin/java.exe")));
            } catch (IOException ignored) { }
        }
        Path launcherHome = Path.of(System.getProperty("java.home")).toAbsolutePath().normalize();
        return result.stream().filter(Files::isRegularFile).filter(p -> !p.toAbsolutePath().normalize().startsWith(launcherHome)).toList();
    }
    public Path ensure(JsonObject requirement, Path custom, Cancellation token, Consumer<Downloads.Progress> progress,
                       Consumer<String> message) throws IOException, InterruptedException {
        int major = requirement.get("majorVersion").getAsInt();
        if (custom != null) {
            token.check();
            try {
                JavaInfo info = probe(custom);
                if (info.major() == major && info.x64()) { message.accept("Using compatible custom Java " + major); return info.executable(); }
            } catch (IOException | IllegalArgumentException ignored) { }
            message.accept("Custom Java is incompatible; automatically selecting Java " + major);
        }
        for (Path candidate : candidates()) {
            token.check();
            try {
                JavaInfo info = probe(candidate);
                if (info.major() == major && info.x64()) { message.accept("Reusing detected Java " + major); return candidate; }
            } catch (IOException | IllegalArgumentException ignored) { }
        }
        String component = requirement.get("component").getAsString();
        JsonObject index = metadata.cached("runtimes", Metadata.RUNTIMES, false, token).getAsJsonObject();
        JsonObject platform = index.getAsJsonObject("windows-x64");
        if (platform == null || !platform.has(component) || platform.getAsJsonArray(component).isEmpty())
            throw new IOException("Official runtime metadata has no Windows x64 runtime for " + component);
        JsonObject selected = platform.getAsJsonArray(component).get(0).getAsJsonObject();
        var manifestArtifact = Metadata.artifact(selected.getAsJsonObject("manifest"));
        Path manifestFile = downloads.fetch(manifestArtifact, root, "manifests/" + manifestArtifact.sha1() + ".json", token, progress);
        JsonObject manifest = Metadata.parse(Files.readString(manifestFile)).getAsJsonObject();
        Path runtime = SafeFiles.resolve(root, component + "/" + manifestArtifact.sha1());
        message.accept("Verifying shared Java " + major + "; missing or damaged files will download");
        var requiredFiles = new ArrayList<Downloads.Request>();
        for (var entry : manifest.getAsJsonObject("files").entrySet()) {
            token.check(); JsonObject item = entry.getValue().getAsJsonObject();
            Path target = SafeFiles.resolve(runtime, entry.getKey());
            switch (item.get("type").getAsString()) {
                case "directory" -> Files.createDirectories(target);
                case "file" -> requiredFiles.add(new Downloads.Request(Metadata.artifact(item.getAsJsonObject("downloads").getAsJsonObject("raw")), runtime, entry.getKey()));
                default -> throw new IOException("Unsupported runtime link/type; no files were executed");
            }
        }
        downloads.fetchAll(requiredFiles, token, progress);
        JavaInfo info = probe(SafeFiles.resolve(runtime, "bin/java.exe"));
        if (info.major() != major || !info.x64()) throw new IOException("Downloaded runtime does not match official version requirements");
        return info.executable();
    }
}
