// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.launcher;
import org.aeroclient.launcher.install.*;
import java.nio.file.Path;

/** Non-Application entry point supports opt-in diagnostics without starting JavaFX. */
public final class AeroLauncher {
    private AeroLauncher() { }
    public static void main(String[] args) throws Exception {
        if (args.length == 1 && args[0].equals("--self-check")) {
            System.out.println("Aero Client M2 installation foundation OK; authenticated play disabled."); return;
        }
        if (args.length > 0 && args[0].equals("--verify-metadata")) {
            try (var installer = new Installer(dataRoot())) {
                var token = new Cancellation(); var manifest = installer.metadata().versions(true, token);
                for (String id : Metadata.TARGETS) {
                    Metadata.findVersion(manifest, id);
                    var version = installer.metadata().version(id, token);
                    System.out.println(id + " available; Java " + version.getAsJsonObject("javaVersion"));
                }
            }
            return;
        }
        if (args.length >= 3 && args[0].equals("--install")) {
            try (var installer = new Installer(dataRoot())) {
                var instance = installer.instances().create("Aero Default " + args[1], args[1], Instances.Loader.valueOf(args[2].toUpperCase()), true);
                System.out.println(installer.install(instance, null, new Cancellation(), p -> { }, System.out::println));
            }
            return;
        }
        AeroApplication.launch(AeroApplication.class, args);
    }
    public static Path dataRoot() {
        return Path.of(System.getProperty("aero.dataDir", Path.of(System.getProperty("user.home"), ".aero-client", "data").toString()));
    }
}
