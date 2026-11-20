// SPDX-License-Identifier: MIT
package org.aeroclient.launcher.install;

import java.io.Closeable;
import java.io.IOException;
import java.util.concurrent.CancellationException;

public final class Cancellation {
    private volatile boolean cancelled;
    private final java.util.Map<Thread, Closeable> active = new java.util.HashMap<>();
    public synchronized void cancel() {
        cancelled = true;
        var closing = java.util.List.copyOf(active.values());
        Thread.ofVirtual().start(() -> { for (Closeable resource : closing) try { resource.close(); } catch (IOException ignored) { } });
    }
    public void check() {
        if (cancelled || Thread.currentThread().isInterrupted()) throw new CancellationException("Installation cancelled; verified files kept for retry.");
    }
    public synchronized void attach(Closeable resource) {
        check();
        active.put(Thread.currentThread(), resource);
    }
    public synchronized void detach() { active.remove(Thread.currentThread()); }
}
