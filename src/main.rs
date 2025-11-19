//#![windows_subsystem = "windows"]
use eframe::egui;

use egui_notify::{Toast, Toasts};
use std::time::Duration;
use std::os::windows::ffi::OsStrExt;
use winreg::RegKey;
use winreg::enums::*;
use std::ffi::OsStr;
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, ShowWindow, SW_MINIMIZE};


include!("hotkeyreg.rs");
include!("iconhandler.rs");
include!("spotifyfunctions.rs");


// helper: create/remove registry run key (module-level so main() can call it)
fn set_startup(enabled: bool, app_name: &str, exe_path: &str, args: Option<&str>) -> std::io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let (key, _disp) = hkcu.create_subkey(run_path)?;
    if enabled {
        let value = if let Some(a) = args { format!("\"{}\" {}", exe_path, a) } else { format!("\"{}\"", exe_path) };
        key.set_value(app_name, &value)?;
    } else {
        let _ = key.delete_value(app_name);
    }
    Ok(())
}

// helper: attempt to minimize a window by title after a short delay (module-level)
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




fn main() -> eframe::Result {
    
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // If the process was started with --minimized we'll spawn a helper thread later
    let start_minimized_arg = std::env::args().any(|a| a == "--minimized");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 480.0]), //window dimensions
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

            

            // load settings into app
            if let Ok(s) = AppSettings::load() {
                app.settings = s;
            }
            // load saved binds (if any) and populate UI fields
            if let Ok(b) = AppSettings::load() {
                app.settings = b.clone();
                if !b.toggle.is_empty() { app.toggleplayback = b.toggle.clone(); }
                if !b.play.is_empty() { app.play = b.play.clone(); }
                if !b.pause.is_empty() { app.pause = b.pause.clone(); }
                if !b.next.is_empty() { app.next = b.next.clone(); }
                if !b.previous.is_empty() { app.previous = b.previous.clone(); }
            }
            let tray = icon();
            let ctx = _cc.egui_ctx.clone();
            
            std::thread::spawn(move || {
                
                let menu_channel = MenuEvent::receiver();
                
                loop {
                    if let Ok(event) = menu_channel.recv() {
                        match event.id.0.as_str() {
                            "Show" => {
                                
                                println!("Show button clicked!");
                                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                                
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                                ctx.request_repaint();
                                
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
    }

    #[derive(Serialize, Deserialize, Default, Clone)]
    struct AppSettings {
        start_on_login: bool,
        start_minimized: bool,
        toggle: String,
        play: String,
        pause: String,
        next: String,
        previous: String,
        
    }


    impl AppSettings {
        fn path() -> PathBuf {
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
        spotify: Option<AuthCodeSpotify>,
        settings: AppSettings,
        tray_icon: Option<TrayIcon>,
        spotifyinitialized: bool,
        
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
                    spotify: None,
                    settings: AppSettings::default(),
                    tray_icon: None,
                    spotifyinitialized: false,
                    
                }
        }
    }

    

    impl eframe::App for Appinfo {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

            if ctx.input(|i| i.viewport().close_requested()) {
                    
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
            
            


            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("SpotifyBinds");

                // helper: create/remove registry run key
                fn set_startup(enabled: bool, app_name: &str, exe_path: &str, args: Option<&str>) -> std::io::Result<()> {
                    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                    let run_path = r"Software\Microsoft\Windows\CurrentVersion\Run";
                    let (key, _disp) = hkcu.create_subkey(run_path)?;
                    if enabled {
                        let value = if let Some(a) = args { format!("\"{}\" {}", exe_path, a) } else { format!("\"{}\"", exe_path) };
                        key.set_value(app_name, &value)?;
                    } else {
                        let _ = key.delete_value(app_name);
                    }
                    Ok(())
                }

                // helper: attempt to minimize a window by title after a short delay
                fn minimize_window_by_title(title: &str) {
                    let title_w: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(300));
                        unsafe {
                            let hwnd = FindWindowW(PCWSTR(title_w.as_ptr()), PCWSTR::null());
                            // HWND::0 is 0 if invalid
                            if hwnd.0 != 0 {
                                let _ = ShowWindow(hwnd, SW_MINIMIZE);
                            }
                        }
                    });
                }

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
                        instance.save_to_json().unwrap();
                        (self.toasts.success("Saved!"));
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

                    if changed {
                        // persist settings
                        let _ = self.settings.save();

                        // update registry entry if requested
                        if let Ok(exe) = std::env::current_exe() {
                            let exe_path = exe.display().to_string();
                            let args = if self.settings.start_minimized { Some("--minimized") } else { None };
                            let _ = set_startup(self.settings.start_on_login, "SpotifyBinds", &exe_path, args);
                        }
                    }
                });

                ui.horizontal(|ui|{
                    if ui.button("Start").clicked() {
                        // Move the spotify client into a background async worker so
                        // key events are handled even when the UI is minimized.
                        if let Some(spotify) = self.spotify.take() {
                            let toggle_key = self.toggleplayback.clone();
                            let play_key = self.play.clone();
                            let pause_key = self.pause.clone();
                            let next_key = self.next.clone();
                            let previous_key = self.previous.clone();
                            

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
                                listenforkey_send(tx_clone, toggle, play, pause, next, previous);
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
                                    }
                                }
                            });

                            (self.toasts.success("Started! Running in background."));
                        } else {
                            (self.toasts.info("Spotify client not initialized."));
                        }
                    }
                });

                ui.add_space(10.0);
                ui.separator();
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
                });

            });
        }
    }

