// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.launcher.install;

import com.sun.net.httpserver.HttpServer;
import org.junit.jupiter.api.*;
import org.junit.jupiter.api.io.TempDir;
import java.io.*;
import java.net.*;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.*;
import static org.junit.jupiter.api.Assertions.*;

class DownloadsTest {
    @TempDir Path root;
    private HttpServer server;
    private Downloads downloads;
    private URI url;
    private ExecutorService serverWorkers;
    private final byte[] payload = "verified Minecraft fixture, not a game binary".getBytes(java.nio.charset.StandardCharsets.UTF_8);
    @BeforeEach void start() throws Exception {
        server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
        serverWorkers = Executors.newCachedThreadPool(); server.setExecutor(serverWorkers);
        server.start(); url = URI.create("http://127.0.0.1:" + server.getAddress().getPort() + "/file");
        downloads = new Downloads(Set.of("127.0.0.1"), true);
    }
    @AfterEach void stop() { downloads.close(); server.stop(0); serverWorkers.shutdownNow(); }
    private Downloads.Artifact artifact() throws Exception {
        Path expected = root.resolve("expected"); Files.write(expected, payload);
        return new Downloads.Artifact(url, Downloads.hash(expected), payload.length);
    }
    @Test void verifiesAndReusesFilesWithoutNetwork() throws Exception {
        var requests = new AtomicInteger();
        server.createContext("/file", exchange -> { requests.incrementAndGet(); exchange.sendResponseHeaders(200, payload.length); exchange.getResponseBody().write(payload); exchange.close(); });
        var artifact = artifact();
        Path installed = downloads.fetch(artifact, root, "shared/file.jar", new Cancellation(), p -> { });
        assertArrayEquals(payload, Files.readAllBytes(installed));
        downloads.fetch(artifact, root, "shared/file.jar", new Cancellation(), p -> { });
        assertEquals(1, requests.get());
        Files.writeString(installed, "corrupt");
        downloads.fetch(artifact, root, "shared/file.jar", new Cancellation(), p -> { });
        assertEquals(2, requests.get()); assertArrayEquals(payload, Files.readAllBytes(installed));
    }
    @Test void resumesInterruptedPartialWithRange() throws Exception {
        var requestedRange = new AtomicReference<String>();
        Files.write(root.resolve("file.part"), Arrays.copyOf(payload, 9));
        server.createContext("/file", exchange -> {
            requestedRange.set(exchange.getRequestHeaders().getFirst("Range"));
            exchange.getResponseHeaders().set("Content-Range", "bytes 9-" + (payload.length - 1) + "/" + payload.length);
            exchange.sendResponseHeaders(206, payload.length - 9); exchange.getResponseBody().write(payload, 9, payload.length - 9); exchange.close();
        });
        downloads.fetch(artifact(), root, "file", new Cancellation(), p -> { });
        assertEquals("bytes=9-", requestedRange.get()); assertArrayEquals(payload, Files.readAllBytes(root.resolve("file")));
    }
    @Test void serverIgnoringRangeRestartsSafely() throws Exception {
        Files.write(root.resolve("file.part"), Arrays.copyOf(payload, 9));
        server.createContext("/file", exchange -> { exchange.sendResponseHeaders(200, payload.length); exchange.getResponseBody().write(payload); exchange.close(); });
        downloads.fetch(artifact(), root, "file", new Cancellation(), p -> { });
        assertArrayEquals(payload, Files.readAllBytes(root.resolve("file")));
    }
    @Test void retriesTransientFailure() throws Exception {
        var count = new AtomicInteger();
        server.createContext("/file", exchange -> {
            if (count.incrementAndGet() == 1) exchange.sendResponseHeaders(503, -1);
            else { exchange.sendResponseHeaders(200, payload.length); exchange.getResponseBody().write(payload); }
            exchange.close();
        });
        downloads.fetch(artifact(), root, "file", new Cancellation(), p -> { }); assertEquals(2, count.get());
    }
    @Test void neverPublishesCorruptDownload() throws Exception {
        var count = new AtomicInteger(); byte[] bad = new byte[payload.length];
        server.createContext("/file", exchange -> { count.incrementAndGet(); exchange.sendResponseHeaders(200, bad.length); exchange.getResponseBody().write(bad); exchange.close(); });
        assertThrows(IOException.class, () -> downloads.fetch(artifact(), root, "file", new Cancellation(), p -> { }));
        assertFalse(Files.exists(root.resolve("file"))); assertEquals(3, count.get());
    }
    @Test void cancellationKeepsPartialAndDoesNotPublish() throws Exception {
        server.createContext("/file", exchange -> { exchange.sendResponseHeaders(200, payload.length); exchange.getResponseBody().write(payload); exchange.close(); });
        var token = new Cancellation();
        assertThrows(CancellationException.class, () -> downloads.fetch(artifact(), root, "file", token, p -> token.cancel()));
        assertFalse(Files.exists(root.resolve("file"))); assertTrue(Files.exists(root.resolve("file.part")));
    }
    @Test void untrustedRedirectCannotDownload() throws Exception {
        server.createContext("/file", exchange -> { exchange.getResponseHeaders().set("Location", "https://example.org/evil"); exchange.sendResponseHeaders(302, -1); exchange.close(); });
        assertThrows(IOException.class, () -> downloads.fetch(artifact(), root, "file", new Cancellation(), p -> { }));
    }
    @Test void productionPolicyRejectsHttpCredentialsAndLookalikeHosts() throws Exception {
        try (var production = new Downloads()) {
            for (String uri : List.of("http://piston-meta.mojang.com/file", "https://piston-meta.mojang.com.evil.org/file", "https://user@piston-meta.mojang.com/file", "https://piston-meta.mojang.com:123/file"))
                assertThrows(IOException.class, () -> production.validate(URI.create(uri)));
        }
    }
    @Test void batchConcurrencyIsBoundedAndEveryFileVerified() throws Exception {
        var active = new AtomicInteger(); var peak = new AtomicInteger();
        server.createContext("/file", exchange -> {
            int current = active.incrementAndGet(); peak.accumulateAndGet(current, Math::max);
            try {
                try { Thread.sleep(50); } catch (InterruptedException ex) { Thread.currentThread().interrupt(); }
                exchange.sendResponseHeaders(200, payload.length); exchange.getResponseBody().write(payload);
            } finally { active.decrementAndGet(); exchange.close(); }
        });
        var expected = artifact(); var requests = new ArrayList<Downloads.Request>();
        for (int i = 0; i < 8; i++) requests.add(new Downloads.Request(expected, root, "batch/file-" + i));
        downloads.fetchAll(requests, new Cancellation(), p -> { });
        assertTrue(peak.get() <= 3); assertTrue(peak.get() > 1);
        for (var request : requests) assertTrue(Downloads.valid(root.resolve(request.relative()), expected));
    }
    @Test void batchFailureCancelsRemainingTransfers() throws Exception {
        server.createContext("/file", exchange -> { exchange.sendResponseHeaders(404, -1); exchange.close(); });
        var expected = artifact(); var token = new Cancellation();
        assertThrows(IOException.class, () -> downloads.fetchAll(List.of(
            new Downloads.Request(expected, root, "bad/one"), new Downloads.Request(expected, root, "bad/two")), token, p -> { }));
        assertThrows(CancellationException.class, token::check);
        assertFalse(Files.exists(root.resolve("bad/one")));
    }
}
