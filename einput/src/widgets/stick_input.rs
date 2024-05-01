use eframe::egui::{
    Color32, Pos2, Response, Sense, Stroke, TextStyle, Ui, Vec2, Widget, WidgetText,
};
use einput_util::axis::Stick;

pub struct StickInput {
    label: WidgetText,
    stick: Stick,
}

impl StickInput {
    pub fn new(stick: Stick, label: impl Into<WidgetText>) -> Self {
        StickInput {
            label: label.into(),
            stick,
        }
    }
}

impl Widget for StickInput {
    fn ui(self, ui: &mut Ui) -> Response {
        let radius = 25.0;
        let margin = 5.0;

        let label_galley =
            self.label
                .into_galley(ui, Some(false), ui.available_width(), TextStyle::Body);
        let x_galley = WidgetText::from(format!("{:+.02}", self.stick.x)).into_galley(
            ui,
            Some(false),
            ui.available_width(),
            TextStyle::Monospace,
        );
        let y_galley = WidgetText::from(format!("{:+.02}", self.stick.y)).into_galley(
            ui,
            Some(false),
            ui.available_width(),
            TextStyle::Monospace,
        );

        let size = Vec2::new(
            radius * 2.0 + margin * 2.0,
            radius * 2.0
                + margin * 5.0
                + label_galley.rect.height()
                + x_galley.rect.height()
                + y_galley.rect.height(),
        );

        let (rect, response) = ui.allocate_at_least(size, Sense::hover());

        let center = Pos2::new(rect.center().x, rect.bottom() - radius - margin);

        ui.painter()
            .circle(center, 1.0, ui.visuals().error_fg_color, Stroke::NONE);

        ui.painter().circle(
            center + Vec2::new(self.stick.x, self.stick.y) * radius,
            1.0,
            ui.visuals().text_color(),
            Stroke::NONE,
        );

        ui.painter().circle(
            center,
            radius,
            Color32::from_black_alpha(0),
            ui.visuals().widgets.inactive.fg_stroke,
        );

        ui.painter().galley(
            rect.center_top()
                + Vec2::new(
                    -y_galley.rect.width() / 2.0,
                    label_galley.rect.height() + x_galley.rect.height() + margin * 3.0,
                ),
            y_galley,
            ui.visuals().text_color(),
        );

        ui.painter().galley(
            rect.center_top()
                + Vec2::new(
                    -x_galley.rect.width() / 2.0,
                    label_galley.rect.height() + margin * 2.0,
                ),
            x_galley,
            ui.visuals().text_color(),
        );

        ui.painter().galley(
            rect.center_top() + Vec2::new(-label_galley.rect.width() / 2.0, margin),
            label_galley,
            ui.visuals().text_color(),
        );

        response
    }
}
