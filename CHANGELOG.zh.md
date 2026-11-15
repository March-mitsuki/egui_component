# 更新日志

在此记录 breaking changes 及对应的迁移方式。

## 0.4.0

### 不兼容变更

- 从 `egui_component` 0.4.0 开始，支持的 egui 版本为 0.37。如果项目直接依赖 `egui`、`eframe` 或 `egui_extras`，请将这些依赖同步升级至 0.37。
- 表单和滚动区的 ID 参数现在要求 `egui::AsIdSalt`，`drawer::render` 的 ID 参数现在要求 `Hash + Debug`。如果自定义 ID 类型原先只实现了 `Hash`，请添加 `#[derive(Debug)]` 或手动实现 `Debug`。
- `color_picker::Style` 将 `size: f32` 拆分为 `width: f32` 和 `height: f32`。要保持正方形，请将两者都设为原来的尺寸。逐项列出字段的结构体字面量还需补上 `border_stroke` 和 `corner_radius`。可用 `Style::new(theme).width(...).height(...)` 保留默认边框和圆角，也可通过同名构建方法覆盖。色块默认仍为 32 × 32，透明度棋盘格在圆角轮廓内保持方格。
- `icon_button::Style` 的公开字段 `size: f32` 拆分为 `width: f32` 和 `height: f32`。直接读取 `style.size` 的代码应改为读取所需的宽度或高度。使用结构体字面量时，应将 `size` 替换为这两个字段。`.size(value)` 仍可同时设置宽高，`.width(value)` 和 `.height(value)` 可分别设置宽度和高度。
- `button::Style` 新增公开字段 `content_align: egui::Align`。逐项列出字段的结构体字面量需补上 `content_align: egui::Align::Center` 以保持居中。穷尽解构时需列出该字段或使用 `..`。构造函数默认使用 `Center`。可通过 `.content_align(...)` 选择 `Min`、`Center` 或 `Max`，将整组内容左对齐、居中或右对齐。
- `accordion`、`avatar`、`button`、`icon`、`icon_button`、`navigation`、`select`、`switch` 的 `Style` 新增公开字段 `focus_color: egui::Color32`。逐项列出字段的结构体字面量需补上 `focus_color: theme.colors.primary.normal()`，与构造函数的默认值一致。穷尽解构时需列出 `focus_color` 或使用 `..`。

## 0.3.0

### 不兼容变更

- Toast 状态从进程级全局管理器迁移至 egui Context 的临时数据。不同 Context 的通知队列、位置、ID、自定义内容和动画状态相互独立；同一个 Context 的克隆句柄共享状态。Toast 不参与序列化持久化。
- `toast::create`、`custom`、`dismiss`、`set_position`、`info`、`success`、`warning` 和 `error` 的第一个参数现在必须是 `&egui::Context`。创建、关闭和渲染通知时应使用同一个 Context；通知 ID 仅在创建它的 Context 内有效。`toast::renderer(ui, theme)` 用法不变，使用 `ui.ctx()` 中的状态。

```rust
// 修改前
toast::success("保存成功");
toast::dismiss(id);

// 修改后
toast::success(ui.ctx(), "保存成功");
toast::dismiss(ui.ctx(), id);
```

后台任务可克隆应用的 Context 并传入其引用。修改 Toast 时会自动请求重绘。自定义 UI 回调仍可创建或关闭通知。
