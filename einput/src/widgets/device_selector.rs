use eframe::egui::{
    Align2, Color32, Rect, Rounding, Sense, Stroke, TextStyle, Vec2, Widget, WidgetText,
};
use einput_core::device::Device;

pub struct DeviceSelector<'a> {
    text: String,
    pick_state: &'a mut PickState,
}

impl<'a> DeviceSelector<'a> {
    pub fn new(text: impl ToString, pick_state: &'a mut PickState) -> Self {
        Self {
            text: text.to_string(),
            pick_state,
        }
    }
}

impl Widget for DeviceSelector<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let galley = WidgetText::from(self.text).into_galley(
            ui,
            Some(false),
            ui.available_width(),
            TextStyle::Body,
        );

        let width = f32::max(galley.rect.width() + 20.0, 200.0);
        let (rect, response) = ui.allocate_at_least(Vec2::new(width, 30.0), Sense::click());

        let stroke = if response.dnd_hover_payload::<Device>().is_some() {
            ui.visuals().widgets.hovered.fg_stroke.color
        } else {
            ui.visuals().weak_text_color()
        };

        if let Some(dev) = response.dnd_release_payload::<Device>() {
            *self.pick_state = PickState::Pick(dev.as_ref().clone());
        }

        ui.painter().rect(
            rect.shrink(4.0),
            Rounding::same(5.0),
            ui.visuals().extreme_bg_color,
            Stroke::new(1.0, stroke),
        );

        ui.painter().galley(
            rect.left_center() + Vec2::new(10.0, -galley.rect.height() / 2.0),
            galley,
            ui.visuals().text_color(),
        );

        if matches!(&self.pick_state, PickState::Picked) {
            let right_rect = Rect::from_min_max(
                rect.shrink(4.0).right_top() - Vec2::X * 25.0,
                rect.shrink(4.0).right_bottom(),
            );

            let hovered = response.hovered()
                && response
                    .hover_pos()
                    .map(|p| right_rect.contains(p))
                    .unwrap_or(false);

            if hovered {
                ui.painter().rect(
                    right_rect.shrink(1.0),
                    Rounding::same(5.0),
                    Color32::from_black_alpha(0),
                    Stroke::new(1.0, ui.visuals().text_color()),
                );
            }

            let clicked = response.clicked()
                && response
                    .interact_pointer_pos()
                    .map(|p| right_rect.contains(p))
                    .unwrap_or(false);

            if clicked {
                *self.pick_state = PickState::Remove;
            }

            ui.painter().text(
                rect.right_center() - Vec2::X * 10.0,
                Align2::RIGHT_CENTER,
                "✖",
                TextStyle::Monospace.resolve(ui.style()),
                ui.visuals().strong_text_color(),
            );
        }

        response
    }
}

pub enum PickState {
    None,
    Picked,
    Pick(Device),
    Remove,
}
