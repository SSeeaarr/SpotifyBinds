#![windows_subsystem = "windows"]
use eframe::egui;

use egui_notify::Toasts;
use std::time::Duration;
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::{HWND, BOOL, LPARAM};
use auto_launch::AutoLaunch;
use std::env;

mod hotkeyreg;
use hotkeyreg::*;
include!("iconhandler.rs");
include!("spotifyfunctions.rs");

fn main() -> eframe::Result {
    
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    

    // If the process was started with --minimized we'll spawn a helper thread later
    let start_minimized_arg = std::env::args().any(|a| a == "--minimized");
    let start_bg_arg = std::env::args().any(|a| a == "--background");
    
    let icon_data = load_eframe_icon(include_bytes!("mash.png"));

    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 480.0])
            .with_visible(!start_bg_arg)
            .with_icon(icon_data),
        vsync: true,
        ..Default::default()
    };
    
    // Create a Tokio runtime for the entire app
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    
    // If launched with --minimized, spawn a short-lived thread that will find our
    // window by title and minimize it after creation.
    if start_minimized_arg {
        // title must match the string passed to run_native below
        minimize_window_by_title("SpotifyBinds");
    }
    if start_bg_arg {
        options.viewport = options.viewport.with_visible(false);
    }

    eframe::run_native("SpotifyBinds", options, Box::new(|_cc| {
            let mut app = Appinfo::default();

            // Load token data
            if let Ok(instance) = MyToken::from_json() {
                app.clientId = instance.RSPOTIFY_CLIENT_ID.clone();
                app.clientSecret = instance.RSPOTIFY_CLIENT_SECRET.clone();
                app.redirectUri = instance.RSPOTIFY_REDIRECT_URI.clone();
            }

            println!("Attempting initializiation");
            if !app.clientId.is_empty() && !app.clientSecret.is_empty() {
                if let Some(spotify) = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(spotifyinit())
                }) {
                    app.spotify = Some(spotify); //do we have a working spotify connection
                    app.spotifyinitialized = true;
                    
                    println!("Spotify client initialized!");
                } else { //if not, continue with blank template
                    app.spotify=None; //this is redundant since default handles this anyway, but its helpful for debug
                }
            }


            
            // load saved binds (if any) and populate UI fields
            if let Ok(b) = AppSettings::load() {
                app.settings = b.clone();
                if !b.toggle.is_empty() { app.toggleplayback = b.toggle; }
                if !b.play.is_empty() { app.play = b.play; }
                if !b.pause.is_empty() { app.pause = b.pause; }
                if !b.next.is_empty() { app.next = b.next; }
                if !b.previous.is_empty() { app.previous = b.previous; }
                if !b.volup.is_empty() { app.volup = b.volup; }
                if !b.voldown.is_empty() { app.voldown = b.voldown; }
                if !b.mute.is_empty() { app.mute = b.mute; }
                if b.volstepup != 0 { app.volstepup = b.volstepup; }
                if b.volstepdown != 0 { app.volstepdown = b.volstepdown; }
            }

            let exe_path = env::current_exe()
                .expect("Failed to get current executable path")
                .to_string_lossy()
                .to_string();

            //wrap in quotes in case spaces in user folder name
            let quoted_exe_path = format!("\"{}\"", exe_path);

            let mut args = Vec::new();
            if app.settings.start_minimized {
                args.push("--minimized");
            }
            if app.settings.start_in_bg {
                args.push("--background");
            }
            let autolaunch = AutoLaunch::new("SpotifyBinds", &quoted_exe_path, &args);
            // Force update registry by disabling first
            let _ = autolaunch.disable();
            if app.settings.start_on_login{
                let _ = autolaunch.enable();
            }

            if (app.settings.start_on_login || app.settings.start_in_bg || autolaunch.is_enabled().unwrap_or(false)) && app.spotifyinitialized {
                app.alreadystarted = true;
                if let Some(ref spotify) = app.spotify {
                            let spotify_clone = spotify.clone();
                            let toggle_key = app.toggleplayback.clone();
                            let play_key = app.play.clone();
                            let pause_key = app.pause.clone();
                            let next_key = app.next.clone();
                            let previous_key = app.previous.clone();
                            let volup_key = app.volup.clone();
                            let voldown_key = app.voldown.clone();
                            let mute_key = app.mute.clone();
                            let incamt = app.volstepup;
                            let decamt = app.volstepdown;
                            

                            // Create a tokio unbounded channel for the async spotify worker
                            let (tx_tokio, mut rx_tokio) = tokio::sync::mpsc::unbounded_channel::<KeyEvent>();

                            // Start the blocking rdev listener on its own OS thread and pass the
                            // tokio sender directly so it can forward events without a bridge.
                            let toggle = toggle_key.clone();
                            let play = play_key.clone();
                            let pause = pause_key.clone();
                            let next = next_key.clone();
                            let previous = previous_key.clone();
                            let volup = volup_key.clone();
                            let voldown = voldown_key.clone();
                            let mute = mute_key.clone();
                            
                            let tx_clone = tx_tokio.clone();
                            std::thread::spawn(move || {
                                listenforkey_send(tx_clone, toggle, play, pause, next, previous, volup, voldown, mute);
                            });

                            // Spawn the spotify worker on the tokio runtime. It owns the AuthCodeSpotify.
                            tokio::spawn(async move {
                                let client = SpotifyClient { spotify: spotify_clone };
                                while let Some(ev) = rx_tokio.recv().await {
                                    match ev {
                                        KeyEvent::Toggle => { let _ = client.toggle_playback(None).await; }
                                        KeyEvent::Play => { let _ = client.play(None).await; }
                                        KeyEvent::Pause => { let _ = client.pause(None).await; }
                                        KeyEvent::Next => { let _ = client.next_track(None).await; }
                                        KeyEvent::Previous => { let _ = client.previous_track(None).await; }
                                        KeyEvent::Volup => { let _ = client.volup(None, incamt).await; }
                                        KeyEvent::Voldown => { let _ = client.voldown(None, decamt).await; }
                                        KeyEvent::Mute => { let _ = client.mute(None).await; }
                                    }
                                }
                            });

                            (app.toasts.success("Started! Running in background."));
                        } else {
                            (app.toasts.info("Spotify client not initialized."));
                        }
            }

            let tray = icon();
            
            std::thread::spawn(move || {
                
                let menu_channel = MenuEvent::receiver();
                
                loop {
                    if let Ok(event) = menu_channel.recv() {
                        match event.id.0.as_str() {
                           "Show" => {
                                // Use helper to restore styles & show window.
                                restore_and_show_window("SpotifyBinds");
                            }
                            "Quit" => {
                                std::process::exit(0);
                            }
                            _ => {}
                        }
                    }
                }
            });
            app.tray_icon = Some(tray);
            
            Ok(Box::new(app))
        }),
    )

    

}


    // Which bind (if any) the UI is currently recording
    #[derive(PartialEq, Eq, Clone, Copy)]
    enum RecordingTarget {
        Toggle,
        Next,
        Previous,
        Play,
        Pause,
        Volup,
        Voldown,
        Mute,
    }

    #[derive(Serialize, Deserialize, Default, Clone)]
    struct AppSettings {
        start_on_login: bool,
        start_minimized: bool,
        start_in_bg: bool,
        toggle: String,
        play: String,
        pause: String,
        next: String,
        previous: String,
        volup: String,
        voldown: String,
        mute: String,
        volstepup: u32,
        volstepdown: u32,
        
    }


    impl AppSettings {
        fn path() -> PathBuf {
            // Use executable directory for persistence so autostart with a
            // different working directory still finds the settings file.
            if let Ok(exe) = std::env::current_exe() {
                if let Some(dir) = exe.parent() {
                    return dir.join(".spotify_settings.json");
                }
            }
            PathBuf::from(".spotify_settings.json")
        }

        fn load() -> Result<Self, std::io::Error> {
            let p = Self::path();
            if p.exists() {
                let s = std::fs::read_to_string(p)?;
                let cfg: Self = serde_json::from_str(&s)?;
                Ok(cfg)
            } else {
                Ok(Self::default())
            }
        }

        fn save(&self) -> Result<(), std::io::Error> {
            let s = serde_json::to_string_pretty(self).unwrap();
            std::fs::write(Self::path(), s)
        }
    }

    

    // Our application initial state:
    struct Appinfo {
        toasts: Toasts, //notifications
        recording_target: Option<RecordingTarget>,
        clientId: String,
        clientSecret: String,
        redirectUri: String,

        toggleplayback: String,
        play: String,
        pause: String,
        next: String,
        previous: String,
        volup: String,
        voldown: String,
        mute: String,

        volstepup: u32,
        volstepdown: u32,

        spotify: Option<AuthCodeSpotify>,
        settings: AppSettings,
        tray_icon: Option<TrayIcon>,
        spotifyinitialized: bool,
        alreadystarted: bool,
        firstframe: bool,
    }

    impl Default for Appinfo {
        fn default() -> Self {
                Self {
                    toasts: Toasts::default(),
                    recording_target: None,
                    clientId: "".to_owned(),
                    clientSecret: "".to_owned(),
                    redirectUri: "".to_owned(),

                    toggleplayback: "           ".to_owned(),
                    play: "           ".to_owned(),
                    pause: "           ".to_owned(),
                    next: "           ".to_owned(),
                    previous: "           ".to_owned(),
                    volup: "           ".to_owned(),
                    voldown: "           ".to_owned(),
                    mute: "           ".to_owned(),

                    volstepup: 0,
                    volstepdown: 0,

                    spotify: None,
                    settings: AppSettings::default(),
                    tray_icon: None,
                    spotifyinitialized: false,
                    alreadystarted: false,
                    firstframe: true,
                }
        }
    }

    

    impl eframe::App for Appinfo {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

            if self.firstframe && self.settings.start_in_bg {
                self.firstframe = false;
                // Minimize AND remove from taskbar by adjusting extended window style.
                minimize_and_hide_from_taskbar("SpotifyBinds");
            }

            if ctx.input(|i| i.viewport().close_requested()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                // Minimize AND remove from taskbar by adjusting extended window style.
                minimize_and_hide_from_taskbar("SpotifyBinds");
            }
            

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("SpotifyBinds");

                //notifications
                self.toasts.show(ctx);


                ui.horizontal(|ui|{
                    let clientid = ui.label("Client id: ");
                    ui.text_edit_singleline(&mut self.clientId)
                        .labelled_by(clientid.id);

                });
                ui.horizontal(|ui|{
                    let clientsecret = ui.label("Client secret: ");
                    ui.text_edit_singleline(&mut self.clientSecret)
                        .labelled_by(clientsecret.id);
                });
                ui.horizontal(|ui|{
                    let redirecturi = ui.label("Redirtect URI: ");
                    ui.text_edit_singleline(&mut self.redirectUri)
                        .labelled_by(redirecturi.id);
                });

                ui.horizontal(|ui|{
                    if ui.button("Save credentials").clicked() {
                        let instance = MyToken{
                            RSPOTIFY_CLIENT_ID: self.clientId.clone(),
                            RSPOTIFY_CLIENT_SECRET: self.clientSecret.clone(),
                            RSPOTIFY_REDIRECT_URI: self.redirectUri.clone(),
                        };
                        if let Err(e) = instance.save_to_json() {
                            (self.toasts.error(format!("Failed to save: {}", e)));
                        } else {
                            (self.toasts.success("Saved!"));
                        }
                    }
                });

                
                if !self.spotifyinitialized {
                    ui.horizontal(|ui|{
                        if ui.button("Initialize spotify client").clicked() {
                            if let Some(spotify) = tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(spotifyinit())
                            }) {
                                self.spotify = Some(spotify); //do we have a working spotify connection
                                self.spotifyinitialized = true;
                                (self.toasts.success("Spotify client initialized!"));
                            } else {
                                self.spotify=None;
                                self.spotifyinitialized = false;
                                (self.toasts.error("Failed to initialize spotify client. Check your credentials."));
                            }
                        }
                    });
                }
                

                ui.horizontal(|ui|{
                
                    if self.alreadystarted {
                        ui.add_enabled(false, egui::Button::new("Running..."));
                    } else if ui.button("Start").clicked() {
                        self.alreadystarted = true;
                        
                        // Move the spotify client into a background async worker so
                        // key events are handled even when the UI is minimized.
                        if let Some(spotify) = self.spotify.take() {
                            let toggle_key = self.toggleplayback.clone();
                            let play_key = self.play.clone();
                            let pause_key = self.pause.clone();
                            let next_key = self.next.clone();
                            let previous_key = self.previous.clone();
                            let volup_key = self.volup.clone();
                            let voldown_key = self.voldown.clone();
                            let mute_key = self.mute.clone();
                            let incamt = self.volstepup;
                            let decamt = self.volstepdown;
                            

                            // Create a tokio unbounded channel for the async spotify worker
                            let (tx_tokio, mut rx_tokio) = tokio::sync::mpsc::unbounded_channel::<KeyEvent>();

                            // Start the blocking rdev listener on its own OS thread and pass the
                            // tokio sender directly so it can forward events without a bridge.
                            let toggle = toggle_key.clone();
                            let play = play_key.clone();
                            let pause = pause_key.clone();
                            let next = next_key.clone();
                            let previous = previous_key.clone();
                            let tx_clone = tx_tokio.clone();
                            std::thread::spawn(move || {
                                listenforkey_send(tx_clone, toggle, play, pause, next, previous, volup_key, voldown_key, mute_key);
                            });

                            // Spawn the spotify worker on the tokio runtime. It owns the AuthCodeSpotify.
                            tokio::spawn(async move {
                                let client = SpotifyClient { spotify };
                                while let Some(ev) = rx_tokio.recv().await {
                                    match ev {
                                        KeyEvent::Toggle => { let _ = client.toggle_playback(None).await; }
                                        KeyEvent::Play => { let _ = client.play(None).await; }
                                        KeyEvent::Pause => { let _ = client.pause(None).await; }
                                        KeyEvent::Next => { let _ = client.next_track(None).await; }
                                        KeyEvent::Previous => { let _ = client.previous_track(None).await; }
                                        KeyEvent::Volup => { let _ = client.volup(None, incamt).await; }
                                        KeyEvent::Voldown => { let _ = client.voldown(None, decamt).await; }
                                        KeyEvent::Mute => { let _ = client.mute(None).await; }
                                    }
                                }
                            });

                            (self.toasts.success("Started! Running in background."));

                            if ui.button("Stop").clicked(){

                            }
                        } else {
                            (self.toasts.info("Spotify client not initialized."));
                        }
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                

                ui.horizontal(|ui| {
                    // Settings toggles
                    let mut changed = false;
                    let mut start_on_login = self.settings.start_on_login;
                    if ui.checkbox(&mut start_on_login, "Start on login").changed() {
                        self.settings.start_on_login = start_on_login;
                        changed = true;
                    }

                    let mut start_minimized = self.settings.start_minimized;
                    if ui.checkbox(&mut start_minimized, "Start minimized").changed() {
                        self.settings.start_minimized = start_minimized;
                        changed = true;
                    }

                    let mut start_in_background = self.settings.start_in_bg;
                    if ui.checkbox(&mut start_in_background, "Start in background").changed() {
                        self.settings.start_in_bg = start_in_background;
                        changed = true;
                    }


                    if changed {
                        // persist settings
                        let _ = self.settings.save();

                        // Update AutoLaunch registry
                        if let Ok(exe_path) = std::env::current_exe() {
                             let exe_path_str = exe_path.to_string_lossy().to_string();
                             let quoted_exe_path = format!("\"{}\"", exe_path_str);
                             
                             let mut args = Vec::new();
                             if self.settings.start_minimized {
                                 args.push("--minimized");
                             }
                             if self.settings.start_in_bg {
                                 args.push("--background");
                             }
                             
                             let autolaunch = AutoLaunch::new("SpotifyBinds", &quoted_exe_path, &args);
                             
                             
                             if self.settings.start_on_login {
                                 let _ = autolaunch.enable();
                             } else {
                                let _ = autolaunch.disable();
                             }
                        }
                    }
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Toggle playback: ");
                    
                    if self.recording_target == Some(RecordingTarget::Toggle) {
                        ui.label("Press a key or key combination...");

                        if let Some(key_combo) = capture_key_input(ctx) {
                            self.toggleplayback = key_combo.clone();
                            self.settings.toggle = key_combo;
                            let _ = self.settings.save();
                            self.recording_target = None;
                        }

                        if ui.button("Cancel").clicked() {
                            self.recording_target = None;
                        }
                    } else {
                        if ui.button(&self.toggleplayback).clicked() {
                            println!("Recording key...");
                            self.recording_target = Some(RecordingTarget::Toggle);
                            (self.toasts.info("Key recording..."));
                        }
                    }
                    ui.add_space(15.0);
                    if ui.button("Clear").clicked() {
                        self.toggleplayback = "           ".to_owned();
                        self.settings.toggle = "           ".to_owned();
                        let _ = self.settings.save();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Skip: ");
                    
                    if self.recording_target == Some(RecordingTarget::Next) {
                        ui.label("Press a key or key combination...");

                        if let Some(key_combo) = capture_key_input(ctx) {
                            self.next = key_combo.clone();
                            self.settings.next = key_combo;
                            let _ = self.settings.save();
                            self.recording_target = None;
                        }

                        if ui.button("Cancel").clicked() {
                            self.recording_target = None;
                        }
                    } else {
                        if ui.button(&self.next).clicked() {
                            println!("Recording key...");
                            self.recording_target = Some(RecordingTarget::Next);
                            (self.toasts.info("Key recording..."));
                        }
                    }
                    ui.add_space(15.0);
                    if ui.button("Clear").clicked() {
                        self.next = "           ".to_owned();
                        self.settings.next = "           ".to_owned();
                        let _ = self.settings.save();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Previous: ");
                    
                    if self.recording_target == Some(RecordingTarget::Previous) {
                        ui.label("Press a key or key combination...");

                        if let Some(key_combo) = capture_key_input(ctx) {
                            self.previous = key_combo.clone();
                            self.settings.previous = key_combo;
                            let _ = self.settings.save();
                            self.recording_target = None;
                        }

                        if ui.button("Cancel").clicked() {
                            self.recording_target = None;
                        }
                    } else {
                        if ui.button(&self.previous).clicked() {
                            println!("Recording key...");
                            self.recording_target = Some(RecordingTarget::Previous);
                            (self.toasts.info("Key recording..."));
                        }
                    } 
                    ui.add_space(15.0);
                    if ui.button("Clear").clicked() {
                        self.previous = "           ".to_owned();
                        self.settings.previous = "           ".to_owned();
                        let _ = self.settings.save();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Play: ");
                    
                    if self.recording_target == Some(RecordingTarget::Play) {
                        ui.label("Press a key or key combination...");

                        if let Some(key_combo) = capture_key_input(ctx) {
                            self.play = key_combo.clone();
                            self.settings.play = key_combo;
                            let _ = self.settings.save();
                            self.recording_target = None;
                        }

                        if ui.button("Cancel").clicked() {
                            self.recording_target = None;
                        }
                    } else {
                        if ui.button(&self.play).clicked() {
                            println!("Recording key...");
                            self.recording_target = Some(RecordingTarget::Play);
                            (self.toasts.info("Key recording..."));
                        }
                    } 
                    ui.add_space(15.0);
                    if ui.button("Clear").clicked() {
                        self.play = "           ".to_owned();
                        self.settings.play = "           ".to_owned();
                        let _ = self.settings.save();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Pause: ");
                    
                    if self.recording_target == Some(RecordingTarget::Pause) {
                        ui.label("Press a key or key combination...");

                        if let Some(key_combo) = capture_key_input(ctx) {
                            self.pause = key_combo.clone();
                            self.settings.pause = key_combo;
                            let _ = self.settings.save();
                            self.recording_target = None;
                        }

                        if ui.button("Cancel").clicked() {
                            self.recording_target = None;
                        }
                    } else {
                        if ui.button(&self.pause).clicked() {
                            println!("Recording key...");
                            self.recording_target = Some(RecordingTarget::Pause);
                            (self.toasts.info("Key recording..."));
                        }
                    } 
                    ui.add_space(15.0);
                    if ui.button("Clear").clicked() {
                        self.pause = "           ".to_owned();
                        self.settings.pause = "           ".to_owned();
                        let _ = self.settings.save();
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Volume up: ");
                    
                    if self.recording_target == Some(RecordingTarget::Volup) {
                        ui.label("Press a key or key combination...");

                        if let Some(key_combo) = capture_key_input(ctx) {
                            self.volup = key_combo.clone();
                            self.settings.volup = key_combo;
                            let _ = self.settings.save();
                            self.recording_target = None;
                        }

                        if ui.button("Cancel").clicked() {
                            self.recording_target = None;
                        }
                    } else {
                        if ui.button(&self.volup).clicked() {
                            println!("Recording key...");
                            self.recording_target = Some(RecordingTarget::Volup);
                            (self.toasts.info("Key recording..."));
                        }
                    } 
                    ui.add_space(15.0);
                    if ui.button("Clear").clicked() {
                        self.volup = "           ".to_owned();
                        self.settings.volup = "           ".to_owned();
                        let _ = self.settings.save();
                    }
                    
                    if ui.add(egui::Slider::new(&mut self.volstepup, 0..=100).text("Increase amount")).changed() {
                        self.settings.volstepup = self.volstepup;
                        let _ = self.settings.save();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Volume down: ");
                    
                    if self.recording_target == Some(RecordingTarget::Voldown) {
                        ui.label("Press a key or key combination...");

                        if let Some(key_combo) = capture_key_input(ctx) {
                            self.voldown = key_combo.clone();
                            self.settings.voldown = key_combo;
                            let _ = self.settings.save();
                            self.recording_target = None;
                        }

                        if ui.button("Cancel").clicked() {
                            self.recording_target = None;
                        }
                    } else {
                        if ui.button(&self.voldown).clicked() {
                            println!("Recording key...");
                            self.recording_target = Some(RecordingTarget::Voldown);
                            (self.toasts.info("Key recording..."));
                        }
                    } 
                    ui.add_space(15.0);
                    if ui.button("Clear").clicked() {
                        self.voldown = "           ".to_owned();
                        self.settings.voldown = "           ".to_owned();
                        let _ = self.settings.save();
                    }

                    if ui.add(egui::Slider::new(&mut self.volstepdown, 0..=100).text("Decrease amount")).changed() {
                        self.settings.volstepdown = self.volstepdown;
                        let _ = self.settings.save();
                    }
                    
                });

                ui.horizontal(|ui| {
                    ui.label("Mute: ");
                    
                    if self.recording_target == Some(RecordingTarget::Mute) {
                        ui.label("Press a key or key combination...");

                        if let Some(key_combo) = capture_key_input(ctx) {
                            self.mute = key_combo.clone();
                            self.settings.mute = key_combo;
                            let _ = self.settings.save();
                            self.recording_target = None;
                        }

                        if ui.button("Cancel").clicked() {
                            self.recording_target = None;
                        }
                    } else {
                        if ui.button(&self.mute).clicked() {
                            println!("Recording key...");
                            self.recording_target = Some(RecordingTarget::Mute);
                            (self.toasts.info("Key recording..."));
                        }
                    } 
                    ui.add_space(15.0);
                    if ui.button("Clear").clicked() {
                        self.mute = "           ".to_owned();
                        self.settings.mute = "           ".to_owned();
                        let _ = self.settings.save();
                    }

                    
                });

                
            });
            
            
            //ctx.request_repaint_after(Duration::from_millis(33)); // ~30 FPS
        }
    }

fn minimize_window_by_title(title: &str) {
    let title_w: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        unsafe {
            let hwnd = FindWindowW(PCWSTR(title_w.as_ptr()), PCWSTR::null());
            if hwnd.0 != 0 {
                let _ = ShowWindow(hwnd, SW_MINIMIZE);
            }
        }
    });
}


#[cfg(windows)] //HUGE thanks to phoglund on github, this is the only thing that worked.
pub unsafe fn force_window_wakeup() { //Do I know whats going on here? nope. I think its poking the window
    use windows::Win32::UI::WindowsAndMessaging::*; // because it lies in a dormant state after calling
    use windows::Win32::System::Threading::GetCurrentProcessId; //visible(false) on the viewport

    unsafe extern "system" fn enum_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
        let mut pid: u32 = 0;

        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
        }

        let current_pid = unsafe { GetCurrentProcessId() };

        if pid == current_pid {
            if unsafe { IsIconic(hwnd).as_bool() } {
                unsafe {
                    ShowWindow(hwnd, SW_RESTORE);
                }
            } else {
                unsafe {
                    ShowWindow(hwnd, SW_SHOW);
                }
            }

            return BOOL(0); // stop enum
        }

        BOOL(1)
    }

    unsafe {
        EnumWindows(Some(enum_proc), LPARAM(0));
    }
}

// Minimize the window (stops wgpu rendering) and adjust extended styles so it
// does not appear on the taskbar. This avoids the higher CPU cost seen when
// only hiding the window with SW_HIDE while still rendering.
fn minimize_and_hide_from_taskbar(title: &str) {
    let wide: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
    unsafe {
        let hwnd = FindWindowW(PCWSTR::null(), PCWSTR(wide.as_ptr()));
        if hwnd.0 != 0 {
            // Minimize first – this reduces GPU usage.
            ShowWindow(hwnd, SW_MINIMIZE);
            // Modify extended window style: remove APPWINDOW, add TOOLWINDOW so it
            // is not shown on the taskbar but can have a tray icon.
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let mut new_style = ex_style | WS_EX_TOOLWINDOW.0 as i32;
            new_style &= !(WS_EX_APPWINDOW.0 as i32);
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
        }
    }
}

// Restore window: undo style change and bring window back (Show button)
fn restore_and_show_window(title: &str) {
    let wide: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
    unsafe {
        let hwnd = FindWindowW(PCWSTR::null(), PCWSTR(wide.as_ptr()));
        if hwnd.0 != 0 {
            // Restore extended style so it appears again on taskbar.
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let mut new_style = ex_style | WS_EX_APPWINDOW.0 as i32; // add taskbar presence
            new_style &= !(WS_EX_TOOLWINDOW.0 as i32); // remove toolwindow flag
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
            // Restore window (if minimized) and show.
            ShowWindow(hwnd, SW_RESTORE);
            ShowWindow(hwnd, SW_SHOW); // ensure visibility
            SetForegroundWindow(hwnd);
        }
    }
}
