pub fn render(ui: &mut egui::Ui, ratio: f32) {
    let available_size = ui.available_size_before_wrap();
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    let is_vertical =
        ui.layout().main_dir() == egui::Direction::TopDown || ui.layout().main_dir() == egui::Direction::BottomUp;

    if is_vertical {
        // 父级是纵向布局：绘制水平分隔线，限制宽度，按 cross_align 对齐
        let total_width = available_size.x;
        let line_width = total_width * ratio;

        // 先分配一个只有高度（stroke 粗细）的矩形，让布局正常推进
        let (rect, _) = ui.allocate_exact_size(egui::vec2(total_width, stroke.width.max(1.0)), egui::Sense::hover());

        // 根据 cross_align 计算线的 x 起始位置
        let x_start = match ui.layout().cross_align() {
            egui::Align::Min => rect.left(),
            egui::Align::Center => rect.center().x - line_width / 2.0,
            egui::Align::Max => rect.right() - line_width,
        };
        let y = rect.center().y;
        ui.painter().line_segment([egui::pos2(x_start, y), egui::pos2(x_start + line_width, y)], stroke);
    } else {
        // 父级是横向布局：绘制垂直分隔线，限制高度，按 cross_align 对齐
        let total_height = available_size.y;
        let line_height = total_height * ratio;

        // 先分配一个只有宽度（stroke 粗细）的矩形
        let (rect, _) = ui.allocate_exact_size(egui::vec2(stroke.width.max(1.0), total_height), egui::Sense::hover());

        // 根据 cross_align 计算线的 y 起始位置
        let y_start = match ui.layout().cross_align() {
            egui::Align::Min => rect.top(),
            egui::Align::Center => rect.center().y - line_height / 2.0,
            egui::Align::Max => rect.bottom() - line_height,
        };
        let x = rect.center().x;
        ui.painter().line_segment([egui::pos2(x, y_start), egui::pos2(x, y_start + line_height)], stroke);
    }
}
