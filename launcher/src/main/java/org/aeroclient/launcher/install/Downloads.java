// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.launcher.install;

import java.io.*;
import java.net.URI;
import java.net.http.*;
import java.nio.file.*;
import java.security.*;
import java.time.Duration;
import java.util.*;
import java.util.concurrent.*;
import java.util.function.Consumer;

/** One transfer at a time per install. Only verified files become visible at final paths. */
public final class Downloads implements AutoCloseable {
    public record Request(Artifact artifact, Path root, String relative) { }
    public record Progress(String item, long completed, long total) { }
    public record Artifact(URI url, String sha1, long size, Set<String> alternativeHashes) {
        public Artifact(URI url, String sha1, long size) { this(url, sha1, size, Set.of()); }
        public Artifact {
            if (url == null || sha1 == null || !sha1.matches("[a-fA-F0-9]{40}") || size < -1)
                throw new IllegalArgumentException("Missing or invalid artifact integrity metadata");
            alternativeHashes = Set.copyOf(alternativeHashes);
            if (alternativeHashes.stream().anyMatch(hash -> !hash.matches("[a-fA-F0-9]{40}")))
                throw new IllegalArgumentException("Invalid alternate checksum");
        }
    }
    private static final Set<String> HOSTS = Set.of("piston-meta.mojang.com", "piston-data.mojang.com",
        "launchermeta.mojang.com", "launcher.mojang.com", "resources.download.minecraft.net",
        "libraries.minecraft.net", "meta.fabricmc.net", "maven.fabricmc.net", "maven.minecraftforge.net",
        "files.minecraftforge.net", "repo.maven.apache.org");
    private final HttpClient client = HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(20))
        .followRedirects(HttpClient.Redirect.NEVER).build();
    private final Set<String> hosts;
    private final boolean localTesting;
    private final ScheduledThreadPoolExecutor watchdog = new ScheduledThreadPoolExecutor(1, runnable -> {
        var thread = new Thread(runnable, "aero-transfer-timeout"); thread.setDaemon(true); return thread;
    });
    public Downloads() { this(HOSTS, false); }
    Downloads(Set<String> hosts, boolean localTesting) {
        this.hosts = Set.copyOf(hosts); this.localTesting = localTesting;
        watchdog.setRemoveOnCancelPolicy(true); watchdog.setKeepAliveTime(5, TimeUnit.SECONDS); watchdog.allowCoreThreadTimeOut(true);
    }
    private InputStream watched(InputStream input) {
        var activity = new java.util.concurrent.atomic.AtomicLong(System.nanoTime());
        var timeout = watchdog.scheduleAtFixedRate(() -> {
            if (System.nanoTime() - activity.get() > Duration.ofSeconds(30).toNanos()) {
                try { input.close(); } catch (IOException ignored) { }
            }
        }, 5, 5, TimeUnit.SECONDS);
        return new FilterInputStream(input) {
            @Override public int read(byte[] data, int offset, int length) throws IOException {
                int read = in.read(data, offset, length); activity.set(System.nanoTime()); return read;
            }
            @Override public int read() throws IOException { int read = in.read(); activity.set(System.nanoTime()); return read; }
            @Override public void close() throws IOException { timeout.cancel(false); super.close(); }
        };
    }
    public void validate(URI uri) throws IOException {
        if ((!"https".equals(uri.getScheme()) && !(localTesting && "http".equals(uri.getScheme()))) ||
            uri.getUserInfo() != null || uri.getFragment() != null || !hosts.contains(uri.getHost()) ||
            (!localTesting && uri.getPort() != -1 && uri.getPort() != 443))
            throw new IOException("Untrusted download URL rejected: " + uri);
    }
    private HttpResponse<InputStream> open(URI uri, long offset, Cancellation cancellation) throws IOException, InterruptedException {
        for (int redirects = 0; redirects < 5; redirects++) {
            cancellation.check(); validate(uri);
            var request = HttpRequest.newBuilder(uri).timeout(Duration.ofSeconds(45)).header("User-Agent", "AeroClient/1.0.0-beta.1");
            if (offset > 0) request.header("Range", "bytes=" + offset + "-");
            var pending = client.sendAsync(request.GET().build(), HttpResponse.BodyHandlers.ofInputStream());
            HttpResponse<InputStream> response;
            try {
                while (true) {
                    cancellation.check();
                    try { response = pending.get(200, TimeUnit.MILLISECONDS); break; }
                    catch (TimeoutException ignored) { }
                }
            } catch (ExecutionException ex) { throw new IOException("Connection failed: " + uri.getHost(), ex.getCause()); }
            catch (CancellationException | InterruptedException ex) { pending.cancel(true); throw ex; }
            if (Set.of(301, 302, 303, 307, 308).contains(response.statusCode())) {
                response.body().close();
                uri = uri.resolve(response.headers().firstValue("Location").orElseThrow(() -> new IOException("Redirect has no destination")));
            } else return response;
        }
        throw new IOException("Too many download redirects");
    }
    public byte[] metadata(URI uri, Cancellation cancellation) throws IOException, InterruptedException {
        IOException failure = null;
        for (int attempt = 0; attempt < 3; attempt++) {
            try {
                var response = open(uri, 0, cancellation);
                try (var body = watched(response.body())) {
                    cancellation.attach(body);
                    if (response.statusCode() != 200) throw new IOException("Metadata HTTP " + response.statusCode() + " from " + uri.getHost());
                    byte[] bytes = body.readNBytes(8 * 1024 * 1024 + 1);
                    cancellation.check();
                    if (bytes.length > 8 * 1024 * 1024) throw new IOException("Metadata exceeds safety limit");
                    return bytes;
                } finally { cancellation.detach(); }
            } catch (IOException ex) { cancellation.check(); failure = ex; pause(attempt, cancellation); }
        }
        throw new IOException("Cannot fetch metadata. Check your connection and retry. " + failure.getMessage(), failure);
    }
    public static String hash(Path file) throws IOException {
        try (var input = Files.newInputStream(file)) {
            var digest = MessageDigest.getInstance("SHA-1");
            byte[] buffer = new byte[65536]; int read;
            while ((read = input.read(buffer)) != -1) digest.update(buffer, 0, read);
            return HexFormat.of().formatHex(digest.digest());
        } catch (NoSuchAlgorithmException ex) { throw new AssertionError(ex); }
    }
    public static boolean valid(Path path, Artifact artifact) throws IOException {
        if (!Files.isRegularFile(path, LinkOption.NOFOLLOW_LINKS) || (artifact.size() >= 0 && Files.size(path) != artifact.size())) return false;
        String actual = hash(path);
        return actual.equalsIgnoreCase(artifact.sha1()) || artifact.alternativeHashes().stream().anyMatch(actual::equalsIgnoreCase);
    }
    public Path fetch(Artifact artifact, Path root, String relative, Cancellation cancellation,
                      Consumer<Progress> progress) throws IOException, InterruptedException {
        validate(artifact.url()); cancellation.check();
        Path destination = SafeFiles.resolve(root, relative);
        if (valid(destination, artifact)) { progress.accept(new Progress(relative, Files.size(destination), Files.size(destination))); return destination; }
        Files.createDirectories(destination.getParent());
        Path part = SafeFiles.resolve(root, relative + ".part");
        IOException failure = null;
        for (int attempt = 0; attempt < 3; attempt++) {
            cancellation.check();
            try {
                long offset = Files.exists(part) ? Files.size(part) : 0;
                if (artifact.size() >= 0 && offset >= artifact.size()) { Files.deleteIfExists(part); offset = 0; }
                var response = open(artifact.url(), offset, cancellation);
                try (var input = watched(response.body())) {
                    cancellation.attach(input);
                    int status = response.statusCode();
                    if (status == 416) { Files.deleteIfExists(part); throw new IOException("Partial file must restart"); }
                    if (status != 200 && status != 206) throw new IOException("Download HTTP " + status + " from " + artifact.url().getHost());
                    if (status == 206 && !response.headers().firstValue("Content-Range").orElse("").startsWith("bytes " + offset + "-"))
                        throw new IOException("Invalid server resume range");
                    if (status == 200) offset = 0;
                    try (var output = Files.newOutputStream(part, StandardOpenOption.CREATE, StandardOpenOption.WRITE,
                            offset == 0 ? StandardOpenOption.TRUNCATE_EXISTING : StandardOpenOption.APPEND)) {
                        byte[] buffer = new byte[65536]; int read; long count = offset;
                        while ((read = input.read(buffer)) != -1) {
                            cancellation.check(); count += read;
                            if (count > (artifact.size() >= 0 ? artifact.size() : 1024L * 1024 * 1024))
                                throw new IOException("Artifact exceeds declared or allowed size");
                            output.write(buffer, 0, read); progress.accept(new Progress(relative, count, artifact.size()));
                        }
                    }
                } finally { cancellation.detach(); }
                cancellation.check();
                if (!valid(part, artifact)) { Files.deleteIfExists(part); throw new IOException("Integrity mismatch for " + relative); }
                SafeFiles.resolve(root, relative); SafeFiles.replace(part, destination); return destination;
            } catch (IOException ex) { cancellation.check(); failure = ex; pause(attempt, cancellation); }
        }
        throw new IOException("Failed to install " + relative + ". Check connection/disk permissions, then Retry or Repair. " + failure.getMessage(), failure);
    }
    private static void pause(int attempt, Cancellation cancellation) throws InterruptedException {
        if (attempt == 2) return;
        for (int i = 0; i < (attempt + 1) * 3; i++) { cancellation.check(); Thread.sleep(100); }
    }
    /** At most three requests in flight; no unbounded task queue or idle transfer threads. */
    public void fetchAll(List<Request> requests, Cancellation token, Consumer<Progress> progress) throws IOException, InterruptedException {
        if (requests.isEmpty()) return;
        var pool = Executors.newFixedThreadPool(3, runnable -> { var thread = new Thread(runnable, "aero-download"); thread.setDaemon(true); return thread; });
        var completion = new ExecutorCompletionService<Void>(pool);
        int submitted = 0, completed = 0;
        try {
            while (completed < requests.size()) {
                token.check();
                while (submitted < requests.size() && submitted - completed < 3) {
                    Request request = requests.get(submitted++);
                    completion.submit(() -> { fetch(request.artifact(), request.root(), request.relative(), token, progress); return null; });
                }
                var result = completion.poll(200, TimeUnit.MILLISECONDS);
                if (result == null) continue;
                try { result.get(); }
                catch (ExecutionException ex) {
                    token.cancel();
                    if (ex.getCause() instanceof CancellationException cancelled) throw cancelled;
                    throw new IOException("A required download failed; Retry resumes verified progress. " + ex.getCause().getMessage(), ex.getCause());
                }
                completed++;
            }
        } finally { pool.shutdownNow(); pool.awaitTermination(10, TimeUnit.SECONDS); }
    }
    @Override public void close() { watchdog.shutdownNow(); client.close(); }
}
