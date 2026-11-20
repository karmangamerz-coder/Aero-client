// SPDX-License-Identifier: MIT
package org.aeroclient.launcher.install;

import com.google.gson.*;
import org.junit.jupiter.api.*;
import org.junit.jupiter.api.io.TempDir;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.zip.*;
import static org.junit.jupiter.api.Assertions.*;

class InstallationSafetyTest {
    @TempDir Path root;
    @Test void rejectsTraversalReservedNamesAndWindowsStreams() {
        for (String name : List.of("../outside", "/absolute", "C:/absolute", "one/../../escape", "one\\two", "file:stream", "CON.txt", "NUL", "path/..", "name.", "foo//bar"))
            assertThrows(IOException.class, () -> SafeFiles.resolve(root, name), name);
    }
    @Test void rejectsSymlinkEscapeWhereSupported() throws Exception {
        Path outside = Files.createDirectory(root.resolve("outside")); Path inside = Files.createDirectory(root.resolve("inside"));
        try { Files.createSymbolicLink(inside.resolve("link"), outside); }
        catch (IOException | UnsupportedOperationException ex) { Assumptions.abort("Host does not permit symlink creation"); }
        assertThrows(IOException.class, () -> SafeFiles.resolve(inside, "link/file"));
    }
    @Test void instancesHaveIndependentUserDataAndStableDefaults() throws Exception {
        var store = new Instances(root);
        var a = store.create("One", "1.8.9", Instances.Loader.VANILLA, false);
        var b = store.create("One", "1.8.9", Instances.Loader.VANILLA, false);
        assertNotEquals(a.id(), b.id());
        Files.writeString(store.directory(a.id()).resolve("game/saves/keep.txt"), "user save");
        store.status(a, Instances.Status.INSTALLING, "testing");
        assertEquals(Instances.Status.INTERRUPTED, store.read(a.id()).status());
        assertFalse(Files.exists(store.directory(b.id()).resolve("game/saves/keep.txt")));
        assertEquals("user save", Files.readString(store.directory(a.id()).resolve("game/saves/keep.txt")));
        assertEquals(store.create("Default", "1.21.11", Instances.Loader.FABRIC, true).id(), store.create("Default", "1.21.11", Instances.Loader.FABRIC, true).id());
    }
    @Test void rejectsUnsupportedVersionAndLoaderPairs() {
        var store = new Instances(root);
        assertThrows(IOException.class, () -> store.create("Bad", "99.9", Instances.Loader.VANILLA, false));
        assertThrows(IOException.class, () -> store.create("Bad", "1.8.9", Instances.Loader.FABRIC, false));
        assertThrows(IOException.class, () -> store.create("Bad", "26.3", Instances.Loader.FORGE, false));
    }
    @Test void nativeZipSlipCannotEscape() throws Exception {
        Path zip = root.resolve("native.zip");
        try (var out = new ZipOutputStream(Files.newOutputStream(zip))) {
            out.putNextEntry(new ZipEntry("../escape.dll")); out.write(new byte[]{1, 2}); out.closeEntry();
        }
        assertThrows(IOException.class, () -> Libraries.extractNatives(zip, root.resolve("natives"), Set.of(), new Cancellation()));
        assertFalse(Files.exists(root.resolve("escape.dll")));
    }
    @Test void nativeExtractionHonorsExclusions() throws Exception {
        Path zip = root.resolve("native.zip");
        try (var out = new ZipOutputStream(Files.newOutputStream(zip))) {
            for (String name : List.of("META-INF/signature", "skip/ignore", "lwjgl.dll")) {
                out.putNextEntry(new ZipEntry(name)); out.write(new byte[]{1, 2}); out.closeEntry();
            }
        }
        Libraries.extractNatives(zip, root.resolve("natives"), Set.of("skip/"), new Cancellation());
        assertTrue(Files.exists(root.resolve("natives/lwjgl.dll"))); assertFalse(Files.exists(root.resolve("natives/skip/ignore")));
    }
    @Test void unavailableVersionsAreNotInvented() throws Exception {
        var manifest = Metadata.parse("{\"versions\":[{\"id\":\"1.8.9\"}]}").getAsJsonObject();
        assertEquals("1.8.9", Metadata.findVersion(manifest, "1.8.9").get("id").getAsString());
        assertThrows(IOException.class, () -> Metadata.findVersion(manifest, "26.3"));
    }
    @Test void windowsRulesAndMavenPathsAreValidated() throws Exception {
        assertFalse(Libraries.allowed(Metadata.parse("{\"rules\":[{\"action\":\"allow\",\"os\":{\"name\":\"linux\"}}]}").getAsJsonObject()));
        assertTrue(Libraries.allowed(Metadata.parse("{\"rules\":[{\"action\":\"allow\",\"os\":{\"name\":\"windows\"}}]}").getAsJsonObject()));
        assertEquals("org/example/core/1.0/core-1.0-natives.jar", Libraries.mavenPath("org.example:core:1.0:natives"));
        assertThrows(IOException.class, () -> Libraries.mavenPath("org.example:../escape:1.0"));
    }
    @Test void runtimeVersionsAreParsedWithoutAssumingLatestJava() {
        assertEquals(8, Runtimes.parseMajor("1.8.0_51")); assertEquals(21, Runtimes.parseMajor("21.0.13")); assertEquals(25, Runtimes.parseMajor("25.0.1+8"));
    }
    @Test void malformedInstanceMetadataIsRejected() throws Exception {
        var store = new Instances(root); var instance = store.create("Test", "1.8.9", Instances.Loader.VANILLA, false);
        Files.writeString(store.directory(instance.id()).resolve("instance.json"), "{\"schema\":999}");
        assertThrows(IOException.class, () -> store.read(instance.id()));
    }
    @Test void forgePublishedChecksumsDoNotRequireMissingSidecars() throws Exception {
        try (var downloads = new Downloads()) {
            var library = Metadata.parse("{\"name\":\"org.example:test:1.0\",\"checksums\":[\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"]}").getAsJsonObject();
            var artifact = Libraries.mavenArtifact(library, downloads, new Cancellation());
            assertEquals(2, artifact.alternativeHashes().size());
        }
    }
    @Test void incompatibleCustomJavaFallsBackWithoutInventingRuntime() throws Exception {
        Path executable = Path.of(System.getProperty("java.home"), "bin", System.getProperty("os.name").startsWith("Windows") ? "java.exe" : "java");
        assertEquals(21, Runtimes.probe(executable).major());
        Path cache = Files.createDirectory(root.resolve("metadata"));
        Files.writeString(cache.resolve("runtimes.json"), "{\"windows-x64\":{}}");
        var messages = new ArrayList<String>();
        try (var downloads = new Downloads()) {
            var manager = new Runtimes(downloads, new Metadata(downloads, cache), root.resolve("runtimes"));
            var required = Metadata.parse("{\"component\":\"missing-component\",\"majorVersion\":999}").getAsJsonObject();
            assertThrows(IOException.class, () -> manager.ensure(required, executable, new Cancellation(), p -> { }, messages::add));
            assertTrue(messages.stream().anyMatch(text -> text.contains("automatically selecting Java 999")));
        }
    }
}
