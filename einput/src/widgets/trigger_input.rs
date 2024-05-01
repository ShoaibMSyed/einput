use eframe::egui::{
    Color32, Rect, Response, Rounding, Sense, Stroke, TextStyle, Ui, Vec2, Widget, WidgetText,
};
use einput_util::axis::Trigger;

pub struct TriggerInput {
    trigger: Trigger,
    label: WidgetText,
}

impl TriggerInput {
    pub fn new(trigger: Trigger, label: impl Into<WidgetText>) -> Self {
        TriggerInput {
            trigger,
            label: label.into(),
        }
    }
}

impl Widget for TriggerInput {
    fn ui(self, ui: &mut Ui) -> Response {
        let margin = 5.0;

        let percent = self.trigger.0 as f32 / 255.0;

        let name_galley =
            self.label
                .into_galley(ui, Some(false), ui.available_width(), TextStyle::Body);

        let percent_galley = WidgetText::from(format!("{percent:.02}")).into_galley(
            ui,
            Some(false),
            ui.available_width(),
            TextStyle::Monospace,
        );

        let text_height = f32::max(name_galley.rect.height(), percent_galley.rect.height());

        let size = Vec2::new(
            f32::max(
                name_galley.rect.width() + percent_galley.rect.width(),
                100.0,
            ),
            text_height + margin + 25.0,
        );

        let (rect, response) = ui.allocate_at_least(size, Sense::hover());

        let trig_rect =
            Rect::from_min_max(rect.min + Vec2::new(0.0, margin + text_height), rect.max);

        ui.painter().rect(
            Rect::from_min_size(
                trig_rect.min,
                Vec2::new(trig_rect.width() * percent, trig_rect.height()),
            ),
            Rounding::ZERO,
            ui.visuals().widgets.inactive.bg_fill,
            Stroke::NONE,
        );

        ui.painter().rect(
            trig_rect,
            Rounding::ZERO,
            Color32::from_black_alpha(0),
            ui.visuals().widgets.inactive.fg_stroke,
        );

        ui.painter()
            .galley(rect.left_top(), name_galley, ui.visuals().text_color());

        ui.painter().galley(
            rect.right_top() - Vec2::new(percent_galley.rect.width(), 0.0),
            percent_galley,
            ui.visuals().text_color(),
        );

        response
    }
}
