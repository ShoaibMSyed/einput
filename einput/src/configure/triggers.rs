use eframe::egui::{
    Color32, CursorIcon, Rect, Rounding, Sense, Stroke, TextStyle, Ui, Vec2, WidgetText,
};
use einput_config::input::TriggerConfig;
use einput_device::input::triggers::TriggerId;
use einput_util::axis::{Trigger, TriggerAxis};

use super::Configure;

impl Configure {
    pub fn tab_triggers(&mut self, ui: &mut Ui) {
        let Some(&triggers) = self.get_raw_input().and_then(|input| input.triggers()) else {
            return;
        };

        ui.horizontal_wrapped(|ui| {
            for id in TriggerId::ALL {
                let config = &mut self.config.input.triggers[id as usize];

                if draw_trigger(ui, id, *triggers.get(id), config) {
                    self.update_config();
                }
            }
        });
    }
}

/// Returns true if config was changed
fn draw_trigger(ui: &mut Ui, id: TriggerId, trigger: Trigger, config: &mut TriggerConfig) -> bool {
    let margin = 2.0;

    let mut changed = false;

    let percent = TriggerAxis::to_f32(trigger.0);

    let mut min: f32 = config.min as f32 / 255.0;
    let mut max = config.max as f32 / 255.0;

    let name_galley = WidgetText::from(format!("{id:?}")).into_galley(
        ui,
        Some(false),
        ui.available_width(),
        TextStyle::Body,
    );

    let value_galley = WidgetText::from(format!("{percent:.02}")).into_galley(
        ui,
        Some(false),
        ui.available_width(),
        TextStyle::Monospace,
    );

    let min_galley = WidgetText::from(format!("{:.02}", min)).into_galley(
        ui,
        Some(false),
        ui.available_width(),
        TextStyle::Monospace,
    );

    let max_galley = WidgetText::from(format!("{:.02}", max)).into_galley(
        ui,
        Some(false),
        ui.available_width(),
        TextStyle::Monospace,
    );

    let size = Vec2::new(
        100.0,
        25.0 + name_galley.rect.height().max(value_galley.rect.height())
            + min_galley.rect.height()
            + margin * 2.0,
    );
    let (rect, _) = ui.allocate_at_least(size, Sense::hover());

    let trigger_rect = Rect::from_min_max(
        rect.min
            + Vec2::new(
                0.0,
                name_galley.rect.height().max(value_galley.rect.height()) + margin,
            ),
        rect.max - Vec2::new(0.0, min_galley.rect.height() + margin),
    );

    // Trigger
    ui.painter().rect(
        Rect::from_min_size(
            trigger_rect.min,
            Vec2::new(trigger_rect.width() * percent, trigger_rect.height()),
        ),
        Rounding::ZERO,
        ui.visuals().widgets.inactive.bg_fill,
        Stroke::NONE,
    );

    // Min Rect
    ui.painter().rect(
        Rect::from_min_size(
            trigger_rect.min + Vec2::X * (min * trigger_rect.width() - 1.0),
            Vec2::new(2.0, trigger_rect.height()),
        ),
        Rounding::ZERO,
        ui.visuals().error_fg_color,
        Stroke::NONE,
    );

    // Max Rect
    ui.painter().rect(
        Rect::from_min_size(
            trigger_rect.min + Vec2::X * (max * trigger_rect.width() - 1.0),
            Vec2::new(2.0, trigger_rect.height()),
        ),
        Rounding::ZERO,
        ui.visuals().hyperlink_color,
        Stroke::NONE,
    );

    // Rect
    ui.painter().rect(
        trigger_rect,
        Rounding::ZERO,
        Color32::TRANSPARENT,
        ui.visuals().widgets.inactive.fg_stroke,
    );

    // Name
    ui.painter()
        .galley(rect.min, name_galley, ui.visuals().text_color());

    // Value
    ui.painter().galley(
        rect.right_top() + Vec2::new(-value_galley.rect.width(), 0.0),
        value_galley,
        ui.visuals().text_color(),
    );

    let min_pos = rect.left_bottom() + Vec2::new(0.0, -min_galley.rect.height());
    let min_rect = Rect::from_min_size(min_pos, min_galley.size());

    // Min
    ui.painter()
        .galley(min_pos, min_galley, ui.visuals().error_fg_color);

    let max_pos =
        rect.right_bottom() + Vec2::new(-max_galley.rect.width(), -max_galley.rect.height());
    let max_rect = Rect::from_min_size(max_pos, max_galley.size());

    // Max
    ui.painter()
        .galley(max_pos, max_galley, ui.visuals().hyperlink_color);

    let min_delta = ui
        .allocate_rect(min_rect, Sense::drag())
        .on_hover_cursor(CursorIcon::ResizeHorizontal)
        .drag_delta()
        .x;
    if min_delta != 0.0 {
        min += min_delta / 250.0;
        config.min = (min * 255.0) as u8;
        changed = true;
    }

    let max_delta = ui
        .allocate_rect(max_rect, Sense::drag())
        .on_hover_cursor(CursorIcon::ResizeHorizontal)
        .drag_delta()
        .x;
    if max_delta != 0.0 {
        max += max_delta / 250.0;
        config.max = (max * 255.0) as u8;
        changed = true;
    }

    ui.add_space(10.0);

    ui.advance_cursor_after_rect(rect);

    changed
}
