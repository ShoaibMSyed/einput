use eframe::egui::{
    Pos2, Rect, Response, Rounding, Sense, Stroke, TextStyle, Vec2, Widget, WidgetText,
};
use einput_device::{
    input::{sticks::StickId, triggers::TriggerId},
    DeviceInfo, DeviceInput, DeviceKind,
};
use einput_util::axis::TriggerAxis;

const MARGIN: f32 = 10.0;

pub struct DevicePreview<'a> {
    info: &'a DeviceInfo,
    input: &'a DeviceInput,
}

impl<'a> DevicePreview<'a> {
    pub fn new(info: &'a DeviceInfo, input: &'a DeviceInput) -> Self {
        Self { info, input }
    }
}

impl DevicePreview<'_> {
    fn gamepad(&self, rect: Rect, ui: &mut eframe::egui::Ui) {
        const BUTTON_PADDING: f32 = 5.0;

        let mut leftover = rect.width();

        if let Some(buttons) = self.input.buttons() {
            for button in buttons.get_pressed() {
                let galley = WidgetText::from(button.name()).into_galley(
                    ui,
                    Some(false),
                    leftover,
                    TextStyle::Small,
                );

                let taken_y = galley.rect.height() + BUTTON_PADDING * 2.0;
                let taken_x = f32::max(taken_y, galley.rect.width() + BUTTON_PADDING * 2.0);

                if leftover < taken_y {
                    return;
                }

                ui.painter().rect(
                    Rect::from_min_size(
                        Pos2::new(
                            rect.right() - leftover,
                            rect.center().y - galley.rect.height() / 2.0 - BUTTON_PADDING,
                        ),
                        Vec2::new(taken_x, taken_y),
                    ),
                    Rounding::same(10.0),
                    ui.visuals().extreme_bg_color,
                    Stroke::new(1.0, ui.visuals().weak_text_color()),
                );

                ui.painter().galley(
                    Pos2::new(
                        rect.right() - leftover + taken_x / 2.0 - galley.rect.width() / 2.0,
                        rect.center().y - galley.rect.height() / 2.0,
                    ),
                    galley,
                    ui.visuals().strong_text_color(),
                );

                leftover -= taken_x + MARGIN / 2.0;
            }
        }

        for id in TriggerId::ALL {
            let Some(trigger) = self.input.get(id) else {
                continue;
            };

            let percent: f32 = TriggerAxis::to_f32(trigger.0);
            if percent < 0.2 {
                continue;
            }

            let galley = WidgetText::from(id.name()).into_galley(
                ui,
                Some(false),
                leftover,
                TextStyle::Small,
            );

            let taken = galley.rect.width() + MARGIN * 2.0;

            if leftover < taken {
                continue;
            }

            let trigger_rect = Rect::from_min_size(
                Pos2::new(
                    rect.right() - leftover + MARGIN / 2.0,
                    rect.center().y - galley.rect.height() / 2.0 - MARGIN / 2.0,
                ),
                Vec2::new(galley.rect.width() + MARGIN, galley.rect.height() + MARGIN),
            );

            ui.painter().rect(
                trigger_rect,
                Rounding::ZERO,
                ui.visuals().extreme_bg_color,
                Stroke::new(1.0, ui.visuals().strong_text_color()),
            );

            let reduce_to = trigger_rect.height() * percent;
            let reduce_by = trigger_rect.height() - reduce_to;

            let trigger_rect =
                Rect::from_min_max(trigger_rect.min + Vec2::Y * reduce_by, trigger_rect.max);

            ui.painter().rect(
                trigger_rect.shrink(1.0),
                Rounding::ZERO,
                ui.visuals().weak_text_color(),
                Stroke::NONE,
            );

            ui.painter().galley(
                Pos2::new(
                    rect.right() - leftover + MARGIN,
                    rect.center().y - galley.rect.height() / 2.0,
                ),
                galley,
                ui.visuals().strong_text_color(),
            );

            leftover -= taken;
        }

        for id in StickId::ALL {
            let Some(stick) = self.input.get(id) else {
                continue;
            };

            if stick.length() <= 0.2 {
                continue;
            }

            let radius = rect.height() / 2.0;

            if leftover < radius * 2.0 + MARGIN {
                continue;
            }

            let center = rect.right_center() + Vec2::X * (-leftover + radius + MARGIN / 2.0);

            ui.painter().circle(
                center,
                radius,
                ui.visuals().extreme_bg_color,
                Stroke::new(1.0, ui.visuals().weak_text_color()),
            );

            ui.painter().circle_filled(
                center + Vec2::new(stick.x, stick.y) * radius,
                1.0,
                ui.visuals().strong_text_color(),
            );

            leftover -= radius * 2.0 + MARGIN;
        }
    }
}

impl Widget for DevicePreview<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> Response {
        let rect = ui.available_rect_before_wrap();

        let response = ui.allocate_rect(rect, Sense::click());

        match self.info.kind {
            DeviceKind::Gamepad => self.gamepad(rect, ui),
            _ => {}
        }

        response
    }
}
