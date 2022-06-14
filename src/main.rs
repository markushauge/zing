#![windows_subsystem = "windows"]

mod audio;
mod effect;
mod graph;
mod settings;

use anyhow::Result;
use audio::{Audio, AudioSession, Message};
use effect::{create_gain, Band, Effect};
use eframe::egui;

const SETTINGS_PATH: &str = "settings.json";

struct App {
    audio: Audio,
    inputs: Vec<String>,
    outputs: Vec<String>,
    effects: Vec<Effect>,
    selected_input: Option<String>,
    selected_output: Option<String>,
    session: Option<AudioSession>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let effects = &mut self.effects;
        let inputs = &self.inputs;
        let outputs = &self.outputs;
        let selected_input = &mut self.selected_input;
        let selected_output = &mut self.selected_output;
        let session = &mut self.session;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ComboBox::from_label("Input")
                .selected_text(selected_input.as_deref().unwrap_or(""))
                .show_ui(ui, |ui| {
                    for input in inputs {
                        if ui
                            .selectable_label(
                                Some(input.as_str()) == selected_input.as_deref(),
                                input,
                            )
                            .clicked()
                        {
                            *selected_input = Some(input.clone())
                        };
                    }
                });

            egui::ComboBox::from_label("Output")
                .selected_text(selected_output.as_deref().unwrap_or(""))
                .show_ui(ui, |ui| {
                    for output in outputs {
                        if ui
                            .selectable_label(
                                Some(output.as_str()) == selected_output.as_deref(),
                                output,
                            )
                            .clicked()
                        {
                            *selected_output = Some(output.clone())
                        };
                    }
                });

            ui.separator();

            for (id, effect) in effects.iter_mut().enumerate() {
                if id > 0 {
                    ui.separator();
                }

                egui::CollapsingHeader::new(effect.name())
                    .id_source(id)
                    .default_open(true)
                    .show(ui, |ui| {
                        effect.update(ui);
                    });
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Add").clicked() {
                    let mut gain = create_gain(1.0);

                    if let Some(session) = session {
                        session.dispatch(Message::Add {
                            effect: gain.node(),
                        });
                    }

                    effects.push(gain);
                }

                if ui.button("Remove").clicked() {
                    let id = effects.len() - 1;

                    if let Some(session) = session {
                        session.dispatch(Message::Remove { id });
                    }

                    effects.remove(id);
                }
            });
        });

        match (session, selected_input, selected_output) {
            (None, Some(input), Some(output)) => {
                self.session = Some(
                    self.audio
                        .session(input, output, &mut self.effects)
                        .unwrap(),
                );
            }
            (Some(session), Some(input), Some(output))
                if &session.input().unwrap() != input || &session.output().unwrap() != output =>
            {
                self.session = Some(
                    self.audio
                        .session(input, output, &mut self.effects)
                        .unwrap(),
                );
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let settings = settings::Settings::read(SETTINGS_PATH).unwrap_or_default();
    let audio = Audio::new(settings.latency);
    let outputs = audio.outputs()?;
    let inputs = audio.inputs()?;

    let effects = vec![
        effect::create_gain(1.0),
        effect::create_equalizer(vec![
            Band::Peaking {
                frequency: 90.0,
                q: 1.0,
                gain: 0.0,
            },
            Band::Peaking {
                frequency: 250.0,
                q: 1.0,
                gain: 0.0,
            },
            Band::Peaking {
                frequency: 500.0,
                q: 1.0,
                gain: 0.0,
            },
            Band::Peaking {
                frequency: 1500.0,
                q: 1.0,
                gain: 0.0,
            },
            Band::Peaking {
                frequency: 3000.0,
                q: 1.0,
                gain: 0.0,
            },
            Band::Peaking {
                frequency: 5000.0,
                q: 1.0,
                gain: 0.0,
            },
            Band::Peaking {
                frequency: 8000.0,
                q: 1.0,
                gain: 0.0,
            },
        ]),
    ];

    let app = App {
        audio,
        outputs,
        inputs,
        effects,
        selected_input: None,
        selected_output: None,
        session: None,
    };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Zing", native_options, Box::new(|_| Box::new(app)));
}
