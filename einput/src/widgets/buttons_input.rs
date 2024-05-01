use eframe::egui::{
    Frame, Response, Rounding, Sense, Stroke, TextStyle, Ui, Vec2, Widget, WidgetText,
};
use einput_device::input::buttons::{Button, Buttons};

pub struct ButtonsInput {
    available: Buttons,
    buttons: Buttons,
}

impl ButtonsInput {
    pub fn new(buttons: Buttons) -> Self {
        Self {
            available: Buttons::ALL,
            buttons,
        }
    }

    pub fn available(mut self, buttons: Buttons) -> Self {
        self.available = buttons;
        self
    }
}

impl Widget for ButtonsInput {
    fn ui(self, ui: &mut Ui) -> Response {
        let galleys = Button::ALL.map(|button| {
            (
                button,
                WidgetText::from(format!("{button:?}")).into_galley(
                    ui,
                    Some(false),
                    100.0,
                    TextStyle::Body,
                ),
            )
        });

        let button_width = galleys
            .iter()
            .map(|(_, galley)| galley.rect.width())
            .reduce(f32::max)
            .unwrap()
            + 10.0;
        let button_height = galleys
            .iter()
            .map(|(_, galley)| galley.rect.height())
            .reduce(f32::max)
            .unwrap()
            + 5.0;

        Frame::none()
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(5.0);

                    for (button, galley) in galleys {
                        let (rect, _) = ui.allocate_at_least(
                            Vec2::new(button_width, button_height),
                            Sense::hover(),
                        );

                        let stroke = if self.buttons.is_pressed(button) {
                            Stroke::new(1.0, ui.visuals().widgets.active.bg_stroke.color)
                        } else {
                            Stroke::new(1.0, ui.visuals().widgets.inactive.bg_stroke.color)
                        };

                        ui.painter().rect(
                            rect,
                            Rounding::same(1.0),
                            ui.visuals().widgets.inactive.bg_fill,
                            stroke,
                        );

                        let text_color = if self.available.is_pressed(button) {
                            ui.visuals().strong_text_color()
                        } else {
                            ui.visuals().text_color()
                        };

                        ui.painter().galley(
                            rect.center_top() + Vec2::new(-galley.rect.width() / 2.0, 2.5),
                            galley,
                            text_color,
                        );
                    }
                });
            })
            .response
    }
}
