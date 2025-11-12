use eframe::egui;


fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 480.0]), //window dimensions
        ..Default::default()
    };
    
    eframe::run_native("SpotifyBinds", options, Box::new(|cc| {
            Ok(Box::<Appinfo>::default())
        }),
    )



}
    // Our application initial state:
    struct Appinfo {
        age: u32,
        clientId: String,
        clientSecret: String,
        redirectUri: String,
        toggleplayback: String,
    }

    impl Default for Appinfo {
        fn default() -> Self {
                Self {
                    age: 42,
                    clientId: "".to_owned(),
                    clientSecret: "".to_owned(),
                    redirectUri: "".to_owned(),
                    toggleplayback: "".to_owned(),
                }
        }
    }

    impl eframe::App for Appinfo {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("SpotifyBinds");


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
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Toggle playback: ");
                    if ui.button(&self.toggleplayback).clicked() {
                        let recordkey = todo!();
                        self.toggleplayback = recordkey;
                    }
                });
            });
        }
    }


