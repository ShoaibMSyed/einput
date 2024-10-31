use std::f32::consts::PI;

use eframe::egui::{self, Color32, RichText, Sense, Slider, Stroke, Ui, Vec2};
use einput_config::input::{StickSampler, StickConfig};
use einput_device::input:: stick::StickId;
use einput_util::axis::Stick;

use super::Configure;

#[derive(Default)]
pub struct SticksTab {
    calibrating: Option<Calibration>,
}

impl Configure {
    pub fn tab_sticks(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for id in StickId::ALL {
                let Some(&raw_stick) = self.get_raw_input().and_then(|input| input.stick(id))
                else {
                    continue;
                };

                let Some(&stick) = self.get_input().and_then(|input| input.stick(id)) else {
                    continue;
                };

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        let config = &mut self.config.input.sticks[id as usize];

                        if let Some(calibration) = &mut self.tab_sticks.calibrating {
                            if calibration.id == id {
                                config.samples = Some(calibration.sampler.samples);
                                if calibration.id == id {
                                    calibration.sampler.add(raw_stick);
                                }
                            }
                        }

                        ui.label(format!("{id:?}"));

                        ui.label(
                            RichText::new(format!(
                                "Raw:        X: {:+.02}, Y: {:+.02}",
                                raw_stick.x, raw_stick.y
                            ))
                            .monospace()
                            .color(ui.visuals().strong_text_color()),
                        );
                        ui.label(
                            RichText::new(format!(
                                "Configured: X: {:+.02}, Y: {:+.02}",
                                stick.x, stick.y
                            ))
                            .monospace()
                            .color(ui.visuals().hyperlink_color),
                        );

                        ui.add_space(2.0);

                        ui.shrink_width_to_current();

                        draw_stick(ui, raw_stick, stick, &config);

                        ui.add_space(2.0);

                        if ui
                            .add(Slider::new(&mut config.deadzone, 0.0..=1.0).text("Deadzone"))
                            .changed()
                        {
                            self.update_config();
                        }

                        if self.tab_sticks.calibrating.is_none() {
                            if ui.button("Calibrate").clicked() {
                                self.tab_sticks.calibrating = Some(Calibration::new(id));
                            }
                        } else if self.tab_sticks.calibrating.as_ref().unwrap().id == id {
                            if ui.button("Finish Calibration").clicked() {
                                let samples =
                                    self.tab_sticks.calibrating.take().unwrap().sampler.samples;
                                self.config.input.sticks[id as usize].samples = Some(samples);
                                self.update_config();
                            }
                        } else {
                            ui.add_enabled(false, egui::Button::new("Calibrate"));
                        }
                    });
                });
            }
        });
    }
}

fn draw_stick(ui: &mut Ui, raw_stick: Stick, stick: Stick, config: &StickConfig) {
    let radius = 50.0;

    let (rect, _) = ui.allocate_at_least(
        Vec2::new(ui.available_width(), radius * 2.0),
        Sense::hover(),
    );

    // Outer Circle
    ui.painter().circle(
        rect.center(),
        radius,
        Color32::from_black_alpha(0),
        ui.visuals().widgets.inactive.fg_stroke,
    );

    // Calibrated
    if let Some(samples) = &config.samples {
        let divisor = samples.len() as f32;

        let mut points = samples.iter().enumerate().map(|(i, scalar)| {
            let angle = i as f32 * PI * 2.0 / divisor;
            let scalar = scalar * radius;
            let x = scalar * angle.cos();
            let y = scalar * angle.sin();
            (x, y)
        });

        let mut first = None;
        let mut prev = points.next();

        for (x, y) in points {
            if first.is_none() {
                first = Some((x, y));
            }

            if let Some((prev_x, prev_y)) = prev {
                prev = Some((x, y));

                ui.painter().line_segment(
                    [
                        rect.center() + Vec2::new(prev_x, prev_y),
                        rect.center() + Vec2::new(x, y),
                    ],
                    Stroke::new(2.0, ui.visuals().hyperlink_color),
                );
            }
        }

        if let (Some(a), Some(b)) = (first, prev) {
            ui.painter().line_segment(
                [
                    rect.center() + Vec2::new(a.0, a.1),
                    rect.center() + Vec2::new(b.0, b.1),
                ],
                Stroke::new(2.0, ui.visuals().hyperlink_color),
            );
        }
    }

    // Deadzone
    ui.painter().circle(
        rect.center(),
        radius * config.deadzone,
        ui.visuals().faint_bg_color,
        Stroke::new(1.0, ui.visuals().error_fg_color),
    );

    // Middle Dot
    ui.painter().circle(
        rect.center(),
        1.0,
        ui.visuals().error_fg_color,
        Stroke::NONE,
    );

    // Raw Stick
    ui.painter().circle(
        rect.center() + Vec2::new(raw_stick.x, raw_stick.y) * radius,
        1.0,
        ui.visuals().strong_text_color(),
        Stroke::NONE,
    );

    // Raw Stick
    ui.painter().circle(
        rect.center() + Vec2::new(stick.x, stick.y) * radius,
        1.0,
        ui.visuals().hyperlink_color,
        Stroke::NONE,
    );
}

struct Calibration {
    id: StickId,
    sampler: StickSampler,
}

impl Calibration {
    fn new(id: StickId) -> Self {
        Calibration {
            id,
            sampler: StickSampler::new(),
        }
    }
}
