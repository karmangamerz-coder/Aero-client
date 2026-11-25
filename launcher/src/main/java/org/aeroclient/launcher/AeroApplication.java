// SPDX-License-Identifier: GPL-3.0-only
package org.aeroclient.launcher;

import com.google.gson.JsonObject;
import javafx.application.*;
import javafx.beans.property.*;
import javafx.geometry.*;
import javafx.scene.*;
import javafx.scene.control.*;
import javafx.scene.layout.*;
import javafx.scene.shape.SVGPath;
import javafx.stage.*;
import org.aeroclient.launcher.install.*;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.*;

public final class AeroApplication extends Application {
    private final ThreadPoolExecutor worker = new ThreadPoolExecutor(1, 1, 15, TimeUnit.SECONDS,
        new ArrayBlockingQueue<>(8), runnable -> { var thread = new Thread(runnable, "aero-install-worker"); thread.setDaemon(true); return thread; });
    private final Installer installer = new Installer(AeroLauncher.dataRoot());
    private final Path preferences = Path.of(System.getProperty("user.home"), ".aero-client");
    private final SettingsStore settings = new SettingsStore(preferences.resolve("settings.properties"));
    private final BooleanProperty busy = new SimpleBooleanProperty(false);
    private final BooleanProperty metadataReady = new SimpleBooleanProperty(false);
    private final BorderPane root = new BorderPane();
    private final Label status = new Label("Refresh versions to check official availability. No background downloads.");
    private final Label transfer = new Label("No active transfers");
    private final ProgressBar progress = new ProgressBar(0);
    private final ComboBox<String> versions = new ComboBox<>();
    private final ComboBox<Instances.Loader> loaders = new ComboBox<>();
    private final TextField instanceName = new TextField("Aero Default");
    private final CheckBox customInstance = new CheckBox("Create a separate custom instance");
    private final ListView<Instances.Instance> library = new ListView<>();
    private final TextField customJava = new TextField();
    private final ComboBox<LauncherSettings.Theme> theme = new ComboBox<>();
    private final Spinner<Integer> memory = new Spinner<>(1024, 32768, 4096, 512);
    private final List<Button> navigation = new ArrayList<>();
    private Cancellation cancellation = new Cancellation();
    private Instances.Instance retryInstance;
    private String lastPage = "Home";
    private Stage stage;
    private long lastProgress;

    @Override public void start(Stage stage) {
        this.stage = stage; worker.allowCoreThreadTimeOut(true);
        versions.setPromptText("Refresh official versions"); versions.setMaxWidth(Double.MAX_VALUE);
        loaders.getItems().setAll(Instances.Loader.VANILLA, Instances.Loader.FABRIC, Instances.Loader.FORGE);
        loaders.setValue(Instances.Loader.VANILLA); loaders.setMaxWidth(Double.MAX_VALUE);
        versions.valueProperty().addListener((value, old, selected) -> {
            if ("1.8.9".equals(selected) && loaders.getValue() == Instances.Loader.FABRIC) loaders.setValue(Instances.Loader.FORGE);
            if (!"1.8.9".equals(selected) && loaders.getValue() == Instances.Loader.FORGE) loaders.setValue(Instances.Loader.FABRIC);
        });
        theme.getItems().setAll(LauncherSettings.Theme.values()); theme.setValue(LauncherSettings.Theme.SYSTEM);
        theme.valueProperty().addListener((observable, previous, selected) -> root.pseudoClassStateChanged(javafx.css.PseudoClass.getPseudoClass("night"), selected == LauncherSettings.Theme.DARK));
        customJava.setPromptText("Automatic — detect or download compatible Java");
        progress.setMaxWidth(Double.MAX_VALUE); status.setWrapText(true); transfer.setWrapText(true);
        var sidebar = new VBox(6); sidebar.getStyleClass().add("sidebar"); sidebar.setPrefWidth(156);
        var brand = new Label("aero"); brand.getStyleClass().add("brand");
        var edition = new Label("CLIENT"); edition.getStyleClass().add("eyebrow");
        sidebar.getChildren().addAll(brand, edition, new Separator());
        String[] pages = {"Home", "Library", "Discover", "Downloads", "Settings"};
        String[] icons = {"M2 10 L10 3 L18 10 M5 9 V18 H15 V9 M8 18 V12 H12 V18", "M3 3 H8 V18 H3 Z M11 3 H16 V18 H11 Z", "M10 2 A8 8 0 1 0 10 18 A8 8 0 1 0 10 2 M7 13 L9 8 L14 6 L12 11 Z", "M10 2 V13 M5 9 L10 14 L15 9 M3 15 V18 H17 V15", "M3 5 H17 M3 10 H17 M3 15 H17 M7 3 V7 M13 8 V12 M8 13 V17"};
        for (int i = 0; i < pages.length; i++) {
            String page = pages[i]; var button = new Button(page, icon(icons[i]));
            button.getStyleClass().add("nav"); button.setMaxWidth(Double.MAX_VALUE);
            button.setOnAction(event -> showPage(page)); navigation.add(button); sidebar.getChildren().add(button);
        }
        Region space = new Region(); VBox.setVgrow(space, Priority.ALWAYS);
        var footer = new Label("Free & open source\nM2 · Installation"); footer.getStyleClass().add("muted");
        sidebar.getChildren().addAll(space, footer); root.setLeft(sidebar);
        var scene = new Scene(root, 1020, 700);
        scene.getStylesheets().add(Objects.requireNonNull(getClass().getResource("aero.css")).toExternalForm());
        stage.setTitle("Aero Client"); stage.setMinWidth(760); stage.setMinHeight(560); stage.setScene(scene);
        showPage("Home"); stage.show();
        submit(() -> {
            LauncherSettings saved = settings.load();
            String java = Files.exists(preferences.resolve("custom-java.txt")) ? Files.readString(preferences.resolve("custom-java.txt")).trim() : "";
            var instances = installer.instances().list();
            Platform.runLater(() -> { theme.setValue(saved.theme()); memory.getValueFactory().setValue(saved.memoryMiB()); customJava.setText(java); library.getItems().setAll(instances); });
        });
    }
    private static SVGPath icon(String data) { var path = new SVGPath(); path.setContent(data); path.getStyleClass().add("outline-icon"); return path; }
    private static Label title(String text) { var label = new Label(text); label.getStyleClass().add("title"); return label; }
    private static Label paragraph(String text) { var label = new Label(text); label.setWrapText(true); label.getStyleClass().add("muted"); return label; }
    private Button button(String text, Runnable action) { var button = new Button(text); button.setOnAction(event -> action.run()); return button; }
    private void showPage(String name) {
        lastPage = name;
        navigation.forEach(button -> button.pseudoClassStateChanged(javafx.css.PseudoClass.getPseudoClass("selected"), button.getText().equals(name)));
        var content = new VBox(18); content.getStyleClass().add("page"); content.setFillWidth(true);
        switch (name) {
            case "Home" -> home(content);
            case "Library" -> {
                content.getChildren().addAll(title("Your library"), paragraph("Each instance keeps its own saves, mods and settings. Assets and compatible runtimes are shared."), library);
                library.setPrefHeight(300);
                var repair = button("Verify / repair selected", () -> { var selected = library.getSelectionModel().getSelectedItem(); if (selected != null) startInstall(selected); });
                repair.disableProperty().bind(busy.or(library.getSelectionModel().selectedItemProperty().isNull()));
                var refresh = button("Refresh library", this::refreshLibrary); refresh.disableProperty().bind(busy);
                content.getChildren().add(new HBox(10, repair, refresh));
            }
            case "Discover" -> content.getChildren().addAll(title("Discover"), paragraph("Modrinth discovery and full instance management arrive in M4."),
                paragraph("For now, Home installs official Minecraft files with Vanilla, compatible Fabric, or legacy Forge for 1.8.9. No marketplace requests are made from this page."));
            case "Downloads" -> {
                var cancel = button("Cancel", () -> cancellation.cancel()); cancel.disableProperty().bind(busy.not());
                var retry = button("Retry / repair last installation", () -> { if (retryInstance != null) startInstall(retryInstance); }); retry.disableProperty().bind(busy);
                content.getChildren().addAll(title("Downloads"), paragraph("Up to three verified transfers at a time. Partial files stay on this device so Retry can resume."), status, transfer, progress, new FlowPane(10, 10, cancel, retry));
            }
            case "Settings" -> settingsPage(content);
            default -> throw new IllegalArgumentException(name);
        }
        var scroll = new ScrollPane(content); scroll.setFitToWidth(true); scroll.getStyleClass().add("page-scroll"); root.setCenter(scroll);
    }
    private void home(VBox content) {
        var hero = new VBox(10); hero.getStyleClass().add("hero");
        hero.getChildren().addAll(paragraph("MINECRAFT JAVA EDITION"), title("A little fresh air.\nA world of your own."), paragraph("Install your next world. Keep it yours."));
        var play = button("Play", () -> { }); play.getStyleClass().add("play"); play.setDisable(true);
        hero.getChildren().addAll(play, paragraph("Play unlocks in M3 with Microsoft sign-in and ownership verification."));
        var form = new GridPane(); form.setHgap(12); form.setVgap(10);
        var first = new ColumnConstraints(); first.setPercentWidth(50); var second = new ColumnConstraints(); second.setPercentWidth(50); form.getColumnConstraints().addAll(first, second);
        form.addRow(0, new Label("Minecraft version"), new Label("Loader")); form.addRow(1, versions, loaders);
        var refresh = button("Refresh official versions", this::refreshVersions); refresh.disableProperty().bind(busy);
        var install = button("Install / repair", () -> startInstall(null)); install.getStyleClass().add("primary");
        install.disableProperty().bind(busy.or(metadataReady.not()).or(versions.valueProperty().isNull()));
        content.getChildren().addAll(hero, title("Prepare an installation"), form, new HBox(10, refresh, install), customInstance, instanceName,
            paragraph("Game installation supports Windows 10/11 x64. Downloads come from Mojang, Fabric and Forge. No account or telemetry is used by M2."));
    }
    private void settingsPage(VBox content) {
        var browse = button("Browse…", () -> {
            var chooser = new FileChooser(); chooser.setTitle("Select a trusted Java executable");
            chooser.getExtensionFilters().add(new FileChooser.ExtensionFilter("Java executable", "java.exe"));
            var file = chooser.showOpenDialog(stage); if (file != null) customJava.setText(file.getAbsolutePath());
        });
        var automatic = button("Use automatic Java", () -> customJava.clear());
        var save = button("Save preferences", () -> {
            LauncherSettings values = new LauncherSettings(theme.getValue(), memory.getValue()); String java = customJava.getText().trim();
            submit(() -> { settings.save(values); SafeFiles.write(preferences.resolve("custom-java.txt"), java); Platform.runLater(() -> status.setText("Preferences saved on this device.")); });
        }); save.disableProperty().bind(busy);
        content.getChildren().addAll(title("Settings"), new Label("Appearance"), theme, new Label("Game memory (MiB; used after M3)"), memory,
            new Label("Custom game Java (optional)"), customJava, new FlowPane(10, 10, browse, automatic),
            paragraph("Only select an executable you trust. Aero checks its version and falls back automatically if incompatible. Game runtimes stay separate from the launcher runtime."),
            save, paragraph("No telemetry. Preferences and installations remain on this device."), status);
    }
    private void refreshVersions() {
        busy.set(true); cancellation = new Cancellation(); var token = cancellation;
        submit(() -> {
            try {
                JsonObject manifest = installer.metadata().versions(true, token); var available = new ArrayList<String>();
                for (String id : Metadata.TARGETS) try { Metadata.findVersion(manifest, id); available.add(id); } catch (java.io.IOException ignored) { }
                Platform.runLater(() -> { versions.getItems().setAll(available); if (!available.isEmpty()) versions.setValue(available.get(0)); metadataReady.set(true); status.setText("Official versions available: " + String.join(", ", available)); });
            } finally { Platform.runLater(() -> busy.set(false)); }
        });
    }
    private void refreshLibrary() { submit(() -> { var all = installer.instances().list(); Platform.runLater(() -> library.getItems().setAll(all)); }); }
    private void startInstall(Instances.Instance existing) {
        if (busy.get()) return;
        String name = instanceName.getText(); String version = versions.getValue(); var loader = loaders.getValue(); boolean custom = customInstance.isSelected(); String java = customJava.getText().trim();
        busy.set(true); cancellation = new Cancellation(); var token = cancellation; progress.setProgress(-1); lastProgress = 0;
        status.setText("Preparing installation…"); showPage("Downloads");
        submit(() -> {
            try {
                var instance = existing != null ? existing : installer.instances().create(custom ? name : "Aero Default " + version, version, loader, !custom);
                retryInstance = instance;
                installer.install(instance, java.isBlank() ? null : Path.of(java), token, this::updateProgress, text -> Platform.runLater(() -> status.setText(text)));
                Platform.runLater(() -> { progress.setProgress(1); transfer.setText("All required files verified."); });
            } finally {
                Platform.runLater(() -> { busy.set(false); if (progress.getProgress() < 0) progress.setProgress(0); });
                var all = installer.instances().list(); Platform.runLater(() -> library.getItems().setAll(all));
            }
        });
    }
    private synchronized void updateProgress(Downloads.Progress update) {
        long now = System.nanoTime(); if (now - lastProgress < 120_000_000) return; lastProgress = now;
        Platform.runLater(() -> {
            transfer.setText(update.item() + "\n" + String.format(Locale.ROOT, "%.1f MiB", update.completed() / 1048576.0));
            progress.setProgress(update.total() > 0 ? Math.min(1, (double) update.completed() / update.total()) : -1);
        });
    }
    @FunctionalInterface private interface Work { void run() throws Exception; }
    private void submit(Work work) {
        try { worker.execute(() -> {
            try { work.run(); }
            catch (Exception ex) { Platform.runLater(() -> { busy.set(false); progress.setProgress(0); status.setText(ex.getMessage() == null ? "Operation failed; check metadata and retry." : ex.getMessage()); if (!lastPage.equals("Settings")) showPage("Downloads"); }); }
        }); } catch (RejectedExecutionException ex) { status.setText("Aero is busy. Wait for the current operation before retrying."); }
    }
    @Override public void stop() { cancellation.cancel(); worker.shutdownNow(); Thread.ofVirtual().start(installer::close); }
}
