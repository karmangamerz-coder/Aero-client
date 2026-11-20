// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.launcher;
import java.awt.*;
import java.io.IOException;
import java.nio.file.Path;
import javax.swing.*;

public final class AeroLauncher {
    private AeroLauncher() {}
    public static void main(String[] args) {
        if (args.length == 1 && args[0].equals("--self-check")) {
            System.out.println("Aero Client foundation OK; authentication and game launch unavailable.");
            return;
        }
        SwingUtilities.invokeLater(AeroLauncher::show);
    }
    private static void show() {
        var store = new SettingsStore(Path.of(System.getProperty("user.home"), ".aero-client", "settings.properties"));
        LauncherSettings settings = LauncherSettings.defaults();
        String status = "Ready. No network connections or telemetry.";
        try { settings = store.load(); }
        catch (IOException ex) { status = "Could not read settings. Defaults shown; Save replaces settings."; }
        var window = new JFrame("Aero Client — Foundation");
        window.setDefaultCloseOperation(WindowConstants.EXIT_ON_CLOSE);
        var panel = new JPanel(new GridLayout(0, 1, 10, 10));
        panel.setBorder(BorderFactory.createEmptyBorder(24, 24, 24, 24));
        panel.add(new JLabel("Aero Client — free, open source, community built"));
        panel.add(new JLabel("Milestone 1: launcher foundation. Minecraft integration is not available yet."));
        panel.add(new JLabel("Theme preference (applied by a future UI milestone)"));
        var theme = new JComboBox<>(LauncherSettings.Theme.values());
        theme.setSelectedItem(settings.theme());
        panel.add(theme);
        panel.add(new JLabel("Planned game memory allocation (MiB)"));
        var memory = new JSpinner(new SpinnerNumberModel(settings.memoryMiB(), 1024, 32768, 512));
        panel.add(memory);
        var message = new JLabel(status);
        var save = new JButton("Save local preferences");
        save.addActionListener(event -> {
            try {
                memory.commitEdit();
                store.save(new LauncherSettings((LauncherSettings.Theme) theme.getSelectedItem(), (Integer) memory.getValue()));
                message.setText("Preferences saved on this device.");
            } catch (IOException | java.text.ParseException | IllegalArgumentException ex) {
                message.setText("Could not save preferences. Check the values and directory permissions.");
            }
        });
        panel.add(save);
        var play = new JButton("Play unavailable — Microsoft authentication required in Milestone 2");
        play.setEnabled(false);
        panel.add(play);
        panel.add(message);
        window.setContentPane(panel);
        window.pack();
        window.setLocationRelativeTo(null);
        window.setVisible(true);
    }
}
