use egui::Ui;

#[derive(Clone)]
pub struct Style {
    pub ratio: f32,
    pub content_padding: f32,
}

impl Style {
    pub fn new() -> Self {
        Self {
            ratio: 1.0,
            content_padding: 8.0,
        }
    }

    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio;
        self
    }

    pub fn content_padding(mut self, content_padding: f32) -> Self {
        self.content_padding = content_padding;
        self
    }
}

pub fn render(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> egui::Response, style: Style) {
    let available_size = ui.available_size_before_wrap();
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    let is_vertical = ui.layout().main_dir() == egui::Direction::TopDown
        || ui.layout().main_dir() == egui::Direction::BottomUp;

    if is_vertical {
        // 父级是纵向布局：绘制水平分隔线，包含文字
        let total_width = available_size.x;
        let line_width = total_width * style.ratio;

        let response =
            ui.allocate_ui_with_layout(egui::vec2(total_width, 0.0), ui.layout().clone(), |ui| {
                ui.set_min_width(total_width);
                ui.allocate_ui_with_layout(
                    egui::vec2(line_width, 0.0),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.set_min_width(line_width);
                        content(ui)
                    },
                )
            });

        let inner_ui_rect = response.inner.response.rect;
        let text_rect = response.inner.inner.rect;

        let x_start = inner_ui_rect.left();
        let x_end = inner_ui_rect.right();
        let y = text_rect.center().y;

        let left_line_end = text_rect.left() - style.content_padding;
        if x_start < left_line_end {
            ui.painter().line_segment(
                [egui::pos2(x_start, y), egui::pos2(left_line_end, y)],
                stroke,
            );
        }

        let right_line_start = text_rect.right() + style.content_padding;
        if right_line_start < x_end {
            ui.painter().line_segment(
                [egui::pos2(right_line_start, y), egui::pos2(x_end, y)],
                stroke,
            );
        }
    } else {
        // 父级是横向布局：绘制垂直分隔线，包含文字
        let total_height = available_size.y;
        let line_height = total_height * style.ratio;

        let response =
            ui.allocate_ui_with_layout(egui::vec2(0.0, total_height), ui.layout().clone(), |ui| {
                ui.set_min_height(total_height);
                ui.allocate_ui_with_layout(
                    egui::vec2(0.0, line_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.set_min_height(line_height);
                        content(ui)
                    },
                )
            });

        let inner_ui_rect = response.inner.response.rect;
        let text_rect = response.inner.inner.rect;

        let y_start = inner_ui_rect.top();
        let y_end = inner_ui_rect.bottom();
        let x = text_rect.center().x;

        let top_line_end = text_rect.top() - style.content_padding;
        if y_start < top_line_end {
            ui.painter().line_segment(
                [egui::pos2(x, y_start), egui::pos2(x, top_line_end)],
                stroke,
            );
        }

        let bottom_line_start = text_rect.bottom() + style.content_padding;
        if bottom_line_start < y_end {
            ui.painter().line_segment(
                [egui::pos2(x, bottom_line_start), egui::pos2(x, y_end)],
                stroke,
            );
        }
    }
}
