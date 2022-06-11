#![windows_subsystem = "windows"]

mod audio;
mod effect;
mod graph;
mod settings;

use anyhow::Result;
use audio::{Audio, AudioSession, Message};
use effect::{Effect, EffectMessage};
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

                match effect {
                    Effect::Gain { volume } => {
                        egui::CollapsingHeader::new("Gain")
                            .id_source(id)
                            .default_open(true)
                            .show(ui, |ui| {
                                if ui.add(egui::Slider::new(volume, 0.0..=4.0)).changed() {
                                    if let Some(session) = session {
                                        session.dispatch(Message::Update {
                                            id,
                                            message: EffectMessage::UpdateGain { volume: *volume },
                                        });
                                    }
                                }
                            });
                    }
                    _ => {}
                }
            }

            if ui.button("Add").clicked() {
                let effect = Effect::Gain { volume: 1.0 };
                effects.push(effect.clone());

                if let Some(session) = session {
                    session.dispatch(Message::Add { effect });
                }
            }

            if ui.button("Remove").clicked() {
                let id = effects.len() - 1;
                effects.remove(id);

                if let Some(session) = session {
                    session.dispatch(Message::Remove { id });
                }
            }
        });

        match (session, selected_input, selected_output) {
            (None, Some(input), Some(output)) => {
                self.session = Some(self.audio.session(input, output, &self.effects).unwrap());
            }
            (Some(session), Some(input), Some(output))
                if &session.input().unwrap() != input || &session.output().unwrap() != output =>
            {
                self.session = Some(self.audio.session(input, output, &self.effects).unwrap());
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let settings = settings::Settings::read(SETTINGS_PATH).unwrap_or_default();
    let effects = vec![Effect::Gain { volume: 1.0 }];

    let audio = Audio::new(settings.latency);
    let outputs = audio.outputs()?;
    let inputs = audio.inputs()?;

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
