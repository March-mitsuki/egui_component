# 更新日志

在此记录 breaking changes 及对应的迁移方式。

## 0.4.0

### 不兼容变更

- 从 `egui_component` 0.4.0 开始，支持的 egui 版本为 0.37。如果项目直接依赖 `egui`、`eframe` 或 `egui_extras`，请将这些依赖同步升级至 0.37。
- 表单和滚动区的 ID 参数现在要求 `egui::AsIdSalt`，`drawer::render` 的 ID 参数现在要求 `Hash + Debug`。如果自定义 ID 类型原先只实现了 `Hash`，请添加 `#[derive(Debug)]` 或手动实现 `Debug`。
- `color_picker::Style` 将 `size: f32` 拆分为 `width: f32` 和 `height: f32`。要保持正方形，请将两者都设为原来的尺寸。逐项列出字段的结构体字面量还需补上 `border_stroke` 和 `corner_radius`。可用 `Style::new(theme).width(...).height(...)` 保留默认边框和圆角，也可通过同名构建方法覆盖。色块默认仍为 32 × 32，透明度棋盘格在圆角轮廓内保持方格。
- `icon_button::Style` 的公开字段 `size: f32` 拆分为 `width: f32` 和 `height: f32`。直接读取 `style.size` 的代码应改为读取所需的宽度或高度。使用结构体字面量时，应将 `size` 替换为这两个字段。`.size(value)` 仍可同时设置宽高，`.width(value)` 和 `.height(value)` 可分别设置宽度和高度。
- `button::Style` 新增公开字段 `content_align: egui::Align`。逐项列出字段的结构体字面量需补上 `content_align: egui::Align::Center` 以保持居中。穷尽解构时需列出该字段或使用 `..`。构造函数默认使用 `Center`。可通过 `.content_align(...)` 选择 `Min`、`Center` 或 `Max`，将整组内容左对齐、居中或右对齐。
- `accordion`、`avatar`、`button`、`icon`、`icon_button`、`navigation`、`select`、`slider`、`switch` 的 `Style` 新增公开字段 `focus_ring: theme::FocusRing`（`slider::Style` 由原来的 `focus_color: Color32` 替换而来）。逐项列出字段的结构体字面量需补上 `focus_ring: theme.focus_ring`，与构造函数的默认值一致。穷尽解构时需列出 `focus_ring` 或使用 `..`。
- 主题 JSON 的 `light` 和 `dark` 现在都必须包含 `focus_ring` 对象，含 `color` 和 `width`。所有组件的键盘焦点指示器都由它绘制，`text_edit` 聚焦时的边框也使用它的宽度。在每个模式中加入例如 `"focus_ring": { "color": <颜色>, "width": 2.0 }`，颜色也可以用 `theme_gen::auto_focus_color(&theme)` 计算。颜色与背景的对比度应不低于 3:1。`theme_gen::generate` 默认由生成的 primary 推导颜色（设置了 `Options::focus_ring_color` 时使用该值），宽度取自 `Options::focus_ring_width`。
- `theme_gen::Options` 新增 `focus_ring_color_light: Option<Color32>`、`focus_ring_color_dark: Option<Color32>`（默认 `None`，按模式分别自动推导）和 `focus_ring_width: f32`（默认 `2.0`，两个模式共用）。逐项列出字段的结构体字面量需补上这些字段，或使用 `..Default::default()`。`theme_gen::apply_focus_ring(theme, color, width)` 用于设置单个模式的焦点环。
- `theme::Color.foreground` 由 `Option<Color32>` 改为必填的 `Color32`。主题 JSON 在两个模式下的每个色阶（`primary`、`gray`、`red`、`green`、`blue`、`yellow`、`orange`）都必须提供 `foreground`。以前缺省时，实心按钮文字回退到 `colors.fg.base`，开关圆钮回退到白色；如需保持原外观，可设为 `fg.base`，或像 gallery 主题现在这样，在 `fg.base` 和 `bg.base` 中选与 `l5`–`l7` 对比度更高的一个。`theme_gen::generate` 已为每个色阶生成该值。Rust 代码中把 `Some(color)` 改为 `color`，并去掉 `unwrap_or(...)`。
- `Color::to_button_palette` 不再接收 `&Theme`，改为 `color.to_button_palette(variant)`。
- 移除仅在 debug 构建中可用的 `UiTheme::dummy`。请用 `theme_gen::generate` 或已有的主题文件创建主题。
- `text_edit::Style` 新增公开字段 `disabled: bool` 和 `read_only: bool`。逐项列出字段的结构体字面量需补上这两个字段，并将两者设为 `false` 以保持可编辑。穷尽解构时需列出这两个字段或使用 `..`。构造函数默认将两者设为 `false`，单行和多行输入框均支持 `.disabled(...)` 和 `.read_only(...)`。
- `accordion::Style` 新增公开字段 `show_body_guide: bool`。逐项列出字段的结构体字面量需补上 `show_body_guide: false`。穷尽解构时需列出该字段或使用 `..`。正文竖线默认关闭，可通过 `.show_body_guide(true)` 开启。
- `accordion::Style` 将 `padding: i8` 拆为 `header_padding_y: i8` 和 `body_indent: i8`，分别控制标题上下各自的内边距，以及开启竖线时的正文左缩进。两者默认均为 `12`，负值按 `0` 处理。逐项列出字段的结构体字面量和穷尽解构需用这两个字段替换 `padding`。折叠组件的标题现在接受可转换为 `egui::WidgetText` 的值。可传入 `egui::RichText` 自定义标题样式，显式设置的标题颜色也会用于悬停下划线。标题栏默认透明、无描边，悬停时标题文字显示下划线。自定义 `hover_background` 时，透明值现在会沿用 `background`。
- `egui_component` 不再启用 `egui_extras` 的 `all_loaders` feature，只启用内置图标所需的 `svg`。需要加载其他图片的应用，例如 `avatar` 通过 `http(s)://` 或 `file://` 加载的头像，以及 PNG/JPEG、GIF、WebP，请在自己的 `egui_extras` 依赖中启用对应的加载器，例如 `egui_extras = { version = "0.37", features = ["all_loaders"] }`，并继续调用 `egui_extras::install_image_loaders(ctx)`。未启用时，`avatar` 会回退为显示名字首字母。
- `field::render` 的 `input_content` 闭包现在必须返回输入控件的 `egui::Response`，返回值也从 `InnerResponse<R>` 改为 `InnerResponse<Response>`。点击 label 会聚焦该控件，`switch` 等开关类控件还会同时切换。大多数情况下，只需去掉闭包末尾的 `;`，把控件的 Response 返回即可；`form.field` 和 `form.field_with_error` 本身已返回该值。如需从闭包中带出其他数据，请赋值给闭包外声明的变量。无论 `label_style.selectable` 如何设置，field 的 label 都不再可选中文本，也不会成为 Tab 停留点。
- `icon_button::render` 新增无障碍名称参数：`render(ui, icon, aria_label, style)`。屏幕阅读器会读出 `aria_label`（相当于 `aria-label`），它不会被绘制。请传入简短、已本地化的操作名称，例如 `"关闭"`。在整个 crate 中，`label` 指显示在界面上的文字，`aria_label` 指只给屏幕阅读器读的名称。
- `icon::render` 新增参数 `options: icon::Options`：`render(ui, icon, style, options)`。装饰性图标传 `Default::default()`。设置 `Options::aria_label` 可以为图标命名：可交互的图标（例如用 `.sense(egui::Sense::drag())` 做拖拽把手）会作为按钮，只能悬停的图标会作为图片。可交互但没有设置名称的图标，在 debug 构建下会记录一次警告。
- 删除 `calendar::render_with_options`、`date_picker::render_with_options` 和 `color_picker::render_with_options`，`render` 改为把原来的 `options` 作为最后一个参数。把 `render_with_options` 调用改名为 `render`，原有的 `render` 调用末尾补上 `Default::default()`。
- `dialog::render_header` 和 `drawer::render_header` 末尾新增参数 `options: HeaderOptions`，`window::render` 新增 `options: window::Options`。传 `Default::default()` 即可；设置 `close_aria_label`（`window` 还有 `collapse_aria_label`）可以本地化关闭按钮和折叠按钮的屏幕阅读器名称，默认值为 `"Close"` 和 `"Collapse"`。
- `text_edit::Style` 新增公开字段 `password_toggle_aria_label: String`，`icon_menu_button::Style` 新增 `menu_handle_aria_label: String`，分别是密码显示切换按钮和菜单把手的无障碍名称，默认值为 `"Show password"` 和 `"More options"`。逐项列出字段的结构体字面量需补上这两个字段，也可以用 `.password_toggle_aria_label(...)` 和 `.menu_handle_aria_label(...)` 本地化。
- `toast::custom` 在 `content` 前新增参数 `announcement: Option<&str>`：`custom(ctx, kind, duration, announcement, content)`。内容自己绘制了文字时传 `None`，屏幕阅读器会读出这些文字；内容没有文字（例如只有进度条）时，传 `Some("...")` 作为 toast 的播报名称。
- `navigation::Style` 新增公开字段 `current_kind: navigation::Current`。逐项列出字段的结构体字面量需补上 `current_kind: navigation::Current::Page`，穷尽解构模式需包含该字段或使用 `..`。`navigation::render` 的第三个参数由 `selected` 改名为 `current`，类型和含义不变。
- 移除 `FontManager::set_egui_fonts`，请改用它原本转发到的 `FontManager::apply`。`get_egui_font_family_with_style` 合并进 `get_egui_font_family`，后者新增最后一个参数 `italic: bool`：现有调用补上 `false`，`get_egui_font_family_with_style` 的调用改名为 `get_egui_font_family`。
- `text::Style` 新增公开字段 `underline: egui::Stroke` 和 `strikethrough: egui::Stroke`，名称与 egui `TextFormat` 的字段一致。逐项列出字段的结构体字面量需补上 `underline: egui::Stroke::NONE` 和 `strikethrough: egui::Stroke::NONE` 以保持无装饰线。穷尽解构时需列出这两个字段或使用 `..`。构造函数默认两者均为 `Stroke::NONE`；用 `.underline(Stroke::new(1.0, color))` 或 `.strikethrough(...)` 绘制线条，宽度和颜色各自独立。
- `text::Style` 新增公开字段 `sense: egui::Sense`，与 `icon::Style` 和 egui 的 `Label::sense` 一致。逐项列出字段的结构体字面量需补上 `sense: egui::Sense::hover()` 以保持文字不响应交互；穷尽解构时需列出该字段或使用 `..`。构造函数默认为 `Sense::hover()`。用 `.selectable(false).sense(egui::Sense::click())` 让文字可点击，并从返回的 `Response` 读取 `clicked()` 或 `double_clicked()`；不设置 `.selectable(false)` 时，双击还会选中一个单词。
- `button::Style`、`checkbox::Style` 和 `switch::Style` 新增公开字段 `aria_label: Option<String>`，作为屏幕阅读器读出的名称，替代按钮文字或复选框的 label（switch 本身没有文字）。逐项列出字段的结构体字面量需补上 `aria_label: None`，以继续使用可见文字作为名称；穷尽解构时需列出该字段或使用 `..`。构造函数默认为 `None`。可见文字不够明确（例如很多行都有 Delete 按钮）或没有可见文字（没有 label 的复选框、switch）时，用 `.aria_label("删除 买牛奶")` 设置。

### 颜色选择器行为变更

- 颜色选择器弹窗改为饱和度／明度色板、色相条、可选透明度条，以及用于切换 HEX 和 RGB(A) 输入的 ghost button。透明度输入使用 `0–1`，替代原先透明度滑块的 `0–255` 刻度；RGB 输入仍为 `0–255` 整数。依赖旧弹窗布局或透明度刻度的 UI 自动化和使用说明需要相应更新。
- HEX 输入允许省略 `#`，合法输入在按 Enter 或失焦时规范化。关闭透明度时显示 `#rrggbb`，开启时显示 `#rrggbbaa`。合法输入仍实时更新颜色，非法草稿保留最后一个合法颜色，并在 Enter 或失焦时显示错误边框。错误文字需通过 `Options::error_message` 回调启用，由应用将 `ColorInputError` 转换为本地化的 `String`。
- 现有 `Style` 字段和构建方法保持不变，无损转换函数 `hex_to_rgba` / `rgba_to_hex` 为新增。`render` 现在也接收 `Options`（见破坏性变更）。

### 表单行为变更

- 在通过 `form.field` 或 `form.field_with_error` 注册的单行输入框中按回车，现在会请求提交；表单校验通过时，`submitted` 为 `true`。提交后焦点仍留在该输入框中。多行输入框按回车仍然换行，用输入法确认候选词时按下的回车也不会触发提交。如需保持原有行为，请使用 `Form::new(id).submit_on_enter(false)`。

### 文本输入框行为变更

- `render_multiline` 现在最多增长到 `Style::max_rows` 行（默认 `5`），超出后在框内滚动，边框和焦点环保持不动；以前会随内容无限增高。`rows` 仍表示最少显示的行数。设置 `.max_rows(None)` 可恢复无限增高；把 `.max_rows(Some(n))` 的 `n` 设为与 `rows` 相同，可得到类似 HTML textarea 的固定高度。逐项列出字段的 `text_edit::Style` 结构体字面量需补上 `max_rows` 字段。
- 密码输入框的 Tab 顺序改为先到输入框，再到显示/隐藏按钮，然后到下一个控件；之前是先到按钮。用鼠标点击该按钮时，焦点仍留在输入框中；用键盘切换时，焦点留在按钮上。

### 无障碍与键盘行为变更

以下变更遵循 WAI-ARIA Authoring Practices 和 WCAG。除特别说明外无需修改代码，但可能影响 UI 自动化测试。

- `icon::Style` 和 `avatar::Style` 的默认值改为 `Sense::hover()`，装饰用的图标和头像不再是 Tab 停留点。需要可点击时请传入 `.sense(egui::Sense::click())`；可点击的图标建议改用 `icon_button`。
- `tooltip::render` 显示的提示在目标获得键盘焦点时也会出现，按 Esc 可关闭，直到指针移开且焦点离开后才会再次显示。
- 处于 `Loading` 状态的 `button` 仍可聚焦但无法触发，按下后焦点不会丢失。禁用的按钮仍不可聚焦。
- `segmented_control` 现在整组只有一个 Tab 停留点，位于选中项上。←/→（以及 ↑/↓）选择上一个或下一个选项，到两端时循环。各选项以单选按钮的语义提供给屏幕阅读器。
- `dialog::render` 和 `drawer::render` 打开时把焦点移到第一个控件，Tab 和 Shift+Tab 在其中循环，关闭后焦点回到打开它们的控件。
- `select` 以及 `menu_button`/`icon_menu_button` 的弹窗支持键盘操作：回车、空格、↓ 或 ↑ 打开弹窗，并聚焦选中项或第一个选项；↓/↑ 在选项间移动（循环），Home/End 跳到首尾，回车或空格触发选项并关闭弹窗，Tab 或 Esc 关闭弹窗。关闭后焦点回到触发按钮。方向键只在本库的按钮和 select 选项之间移动，其他控件仍可用 Tab 访问。
- 键盘焦点环现在统一画在控件内侧，不会被裁剪，也不额外占用空间。焦点环与控件填充之间有一条 1px 的黑色或白色细线（取与焦点环对比度更高的一个，见 `FocusRing::separator_color`），因此在实心填充上也清晰可见，例如以前看不到焦点环的 calendar 选中日期。以前大多数焦点环画在控件外侧，并且会压住控件边缘。
- 按钮、图标按钮、开关、select 的触发按钮与选项、头像、窗口折叠按钮和菜单触发按钮，现在会通过 AccessKit 向屏幕阅读器提供角色、名称和状态（按下、选中、展开）。
- `field::render` 将标签与控件关联（相当于 `<label for>`），屏幕阅读器会把标签读作控件名称。`Form::field` 和 `field_with_error` 在显示错误时把字段标记为无效，并把错误信息关联为字段的描述（相当于 `aria-invalid` 和 `aria-describedby`）。
- `dialog::render` 和 `drawer::render` 以模态对话框提供给屏幕阅读器（`role="dialog"`、`aria-modal`）。`dialog::render_header` 和 `drawer::render_header` 用其标题为所在对话框命名（相当于 `aria-labelledby`）。
- `tooltip::render` 把显示中的提示关联为目标控件的描述。`date_picker` 的触发按钮声明弹出对话框，弹出的日历是非模态对话框。日历中的日期会报告是否选中，今天以 `aria-current="date"` 的方式标记。
- `slider` 会报告最小值、最大值和步长。在聚焦的 slider 上按方向键时，焦点不再同时跳到相邻控件。
- 颜色选择器的饱和度/亮度平面和色相/透明度滑条现在会显示焦点环，并可用方向键调整（每次 1%，按住 Shift 为 10%）。它们以滑块的角色提供给屏幕阅读器，名称默认为英文，可通过 `color_picker::Options::aria_labels` 本地化。
- `switch` 不再响应 Enter 切换，与 `checkbox` 一致：空格、鼠标点击或点击其 `field` 标签可以切换，Enter 留给表单提交。
- `navigation` 的各项改为以链接（而不是切换按钮）提供给屏幕阅读器，当前项按 HTML `aria-current` 的方式标记（默认是 `Page`；页内目录可设置 `.current_kind(navigation::Current::Location)`）。用 `navigation::group(ui, label, ...)` 包住一组导航项，可将其作为带名称的导航地标。
- 新增 `theme_gen::audit(&theme)`，列出不满足 WCAG 对比度的颜色组合（`theme_gen::contrast_checks` 列出全部组合），分为错误（文字低于 4.5:1、焦点环对背景低于 3:1）和警告（surface 按钮文字对 `l0`；焦点环对其分隔线低于 9:1，此时按 WCAG 技巧 C40 无法保证在所有填充色上都可见）。`theme_gen::generate` 生成的主题没有错误。gallery 的主题（`examples/theme.json`）改用默认种子色重新生成并通过检测，因此配色略有变化，例如亮色模式的 primary 从 `#6B65F5` 变为 `#584EDC`。gallery 的主题生成器可以检测粘贴的 JSON，而不导入它。
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
