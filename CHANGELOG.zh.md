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
- `text_edit::Style` 新增公开字段 `disabled: bool` 和 `read_only: bool`。逐项列出字段的结构体字面量需补上这两个字段，并将两者设为 `false` 以保持可编辑。穷尽解构时需列出这两个字段或使用 `..`。构造函数默认将两者设为 `false`，单行和多行输入框均支持 `.disabled(...)` 和 `.read_only(...)`。
- `accordion::Style` 新增公开字段 `show_body_guide: bool`。逐项列出字段的结构体字面量需补上 `show_body_guide: false`。穷尽解构时需列出该字段或使用 `..`。正文竖线默认关闭，可通过 `.show_body_guide(true)` 开启。
- `accordion::Style` 将 `padding: i8` 拆为 `header_padding_y: i8` 和 `body_indent: i8`，分别控制标题上下各自的内边距，以及开启竖线时的正文左缩进。两者默认均为 `12`，负值按 `0` 处理。逐项列出字段的结构体字面量和穷尽解构需用这两个字段替换 `padding`。折叠组件的标题现在接受可转换为 `egui::WidgetText` 的值。可传入 `egui::RichText` 自定义标题样式，显式设置的标题颜色也会用于悬停下划线。标题栏默认透明、无描边，悬停时标题文字显示下划线。自定义 `hover_background` 时，透明值现在会沿用 `background`。
- `egui_component` 不再启用 `egui_extras` 的 `all_loaders` feature，只启用内置图标所需的 `svg`。需要加载其他图片的应用，例如 `avatar` 通过 `http(s)://` 或 `file://` 加载的头像，以及 PNG/JPEG、GIF、WebP，请在自己的 `egui_extras` 依赖中启用对应的加载器，例如 `egui_extras = { version = "0.37", features = ["all_loaders"] }`，并继续调用 `egui_extras::install_image_loaders(ctx)`。未启用时，`avatar` 会回退为显示名字首字母。
- `field::render` 的 `input_content` 闭包现在必须返回输入控件的 `egui::Response`，返回值也从 `InnerResponse<R>` 改为 `InnerResponse<Response>`。点击 label 会聚焦该控件，`switch` 等开关类控件还会同时切换。大多数情况下，只需去掉闭包末尾的 `;`，把控件的 Response 返回即可；`form.field` 和 `form.field_with_error` 本身已返回该值。如需从闭包中带出其他数据，请赋值给闭包外声明的变量。无论 `label_style.selectable` 如何设置，field 的 label 都不再可选中文本，也不会成为 Tab 停留点。
- `icon_button::render` 新增无障碍名称参数：`render(ui, icon, label, style)`。屏幕阅读器会读出 `label`（相当于 `aria-label`），它不会被绘制。请传入简短、已本地化的操作名称，例如 `"关闭"`。
- `text_edit::Style` 新增公开字段 `password_toggle_label: String`，`icon_menu_button::Style` 新增 `menu_handle_label: String`，分别是密码显示切换按钮和菜单把手的无障碍名称，默认值为 `"Show password"` 和 `"More options"`。逐项列出字段的结构体字面量需补上这两个字段，也可以用 `.password_toggle_label(...)` 和 `.menu_handle_label(...)` 本地化。`dialog::render_header`、`drawer::render_header` 和 `window` 内置的关闭按钮使用英文名称 `"Close"`。
- `toast::custom` 在 `content` 前新增参数 `announcement: Option<&str>`：`custom(ctx, kind, duration, announcement, content)`。内容自己绘制了文字时传 `None`，屏幕阅读器会读出这些文字；内容没有文字（例如只有进度条）时，传 `Some("...")` 作为 toast 的播报名称。
- `navigation::Style` 新增公开字段 `current_kind: navigation::Current`。逐项列出字段的结构体字面量需补上 `current_kind: navigation::Current::Page`，穷尽解构模式需包含该字段或使用 `..`。`navigation::render` 的第三个参数由 `selected` 改名为 `current`，类型和含义不变。
- 移除 `FontManager::set_egui_fonts`，请改用它原本转发到的 `FontManager::apply`。`get_egui_font_family_with_style` 合并进 `get_egui_font_family`，后者新增最后一个参数 `italic: bool`：现有调用补上 `false`，`get_egui_font_family_with_style` 的调用改名为 `get_egui_font_family`。

### 颜色选择器行为变更

- 颜色选择器弹窗改为饱和度／明度色板、色相条、可选透明度条，以及用于切换 HEX 和 RGB(A) 输入的 ghost button。透明度输入使用 `0–1`，替代原先透明度滑块的 `0–255` 刻度；RGB 输入仍为 `0–255` 整数。依赖旧弹窗布局或透明度刻度的 UI 自动化和使用说明需要相应更新。
- HEX 输入允许省略 `#`，合法输入在按 Enter 或失焦时规范化。关闭透明度时显示 `#rrggbb`，开启时显示 `#rrggbbaa`。合法输入仍实时更新颜色，非法草稿保留最后一个合法颜色，并在 Enter 或失焦时显示错误边框。错误文字需通过 `render_with_options` 的 `Options::error_message` 回调启用，由应用将 `ColorInputError` 转换为本地化的 `String`。
- 本次重设计没有源码层面的破坏性 API 变更：`render(ui, id, &mut Color32, Style)` 及现有 `Style` 字段和构建方法均保持不变。选项 API 和无损转换函数 `hex_to_rgba` / `rgba_to_hex` 均为新增，现有调用代码无需迁移。

### 表单行为变更

- 在通过 `form.field` 或 `form.field_with_error` 注册的单行输入框中按回车，现在会请求提交；表单校验通过时，`submitted` 为 `true`。提交后焦点仍留在该输入框中。多行输入框按回车仍然换行，用输入法确认候选词时按下的回车也不会触发提交。如需保持原有行为，请使用 `Form::new(id).submit_on_enter(false)`。

### 文本输入框行为变更

- 密码输入框的 Tab 顺序改为先到输入框，再到显示/隐藏按钮，然后到下一个控件；之前是先到按钮。用鼠标点击该按钮时，焦点仍留在输入框中；用键盘切换时，焦点留在按钮上。

### 无障碍与键盘行为变更

以下变更遵循 WAI-ARIA Authoring Practices 和 WCAG。除特别说明外无需修改代码，但可能影响 UI 自动化测试。

- `icon::Style` 和 `avatar::Style` 的默认值改为 `Sense::hover()`，装饰用的图标和头像不再是 Tab 停留点。需要可点击时请传入 `.sense(egui::Sense::click())`；可点击的图标建议改用 `icon_button`。
- `tooltip::render` 显示的提示在目标获得键盘焦点时也会出现，按 Esc 可关闭，直到指针移开且焦点离开后才会再次显示。
- 处于 `Loading` 状态的 `button` 仍可聚焦但无法触发，按下后焦点不会丢失。禁用的按钮仍不可聚焦。
- `segmented_control` 现在整组只有一个 Tab 停留点，位于选中项上。←/→（以及 ↑/↓）选择上一个或下一个选项，到两端时循环。各选项以单选按钮的语义提供给屏幕阅读器。
- `dialog::render` 和 `drawer::render` 打开时把焦点移到第一个控件，Tab 和 Shift+Tab 在其中循环，关闭后焦点回到打开它们的控件。
- `select` 以及 `menu_button`/`icon_menu_button` 的弹窗支持键盘操作：回车、空格、↓ 或 ↑ 打开弹窗，并聚焦选中项或第一个选项；↓/↑ 在选项间移动（循环），Home/End 跳到首尾，回车或空格触发选项并关闭弹窗，Tab 或 Esc 关闭弹窗。关闭后焦点回到触发按钮。方向键只在本库的按钮和 select 选项之间移动，其他控件仍可用 Tab 访问。
- 按钮、图标按钮、开关、select 的触发按钮与选项、头像、窗口折叠按钮和菜单触发按钮，现在会通过 AccessKit 向屏幕阅读器提供角色、名称和状态（按下、选中、展开）。
- `switch` 不再响应 Enter 切换，与 `checkbox` 一致：空格、鼠标点击或点击其 `field` 标签可以切换，Enter 留给表单提交。
- `navigation` 的各项改为以链接（而不是切换按钮）提供给屏幕阅读器，当前项按 HTML `aria-current` 的方式标记（默认是 `Page`；页内目录可设置 `.current_kind(navigation::Current::Location)`）。用 `navigation::group(ui, label, ...)` 包住一组导航项，可将其作为带名称的导航地标。
- toast 现在是 live 区域：info 和 success 会在屏幕阅读器空闲时播报（`role="status"`），warning 和 error 会立即播报（`role="alert"`）。自动消失的计时除了鼠标悬停时，键盘焦点位于 toast 内时也会暂停。播报效果取决于平台的 AccessKit 适配器，eframe 的 web 版没有提供。

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
