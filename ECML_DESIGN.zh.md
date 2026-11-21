# ECML 贡献者指南

[English](ECML_DESIGN.md) | [中文](ECML_DESIGN.zh.md)

本文定义可选 `ecml` feature 的设计目标、架构边界与行为约定。修改语言、绑定 API、渲染或热更新流程之前，请先阅读本文。应用集成见 [ECML 用户指南](examples/ecml_hello_world/README.md)，已支持语法见 Gallery 的 ECML 页面（`examples/gallery/ecml_docs/`），仓库通用协作流程见[贡献指南](CONTRIBUTE.zh.md)。

## 目标与边界

ECML（egui Component Markup Language）让开发者在应用运行期间修改受支持的 UI 结构、文字和样式，无需重新编译 Rust，也不丢失应用状态。模板描述界面，Rust 持有状态并实现业务行为。修改 Rust handler、注册项、组件实现或语言能力仍需编译。

运行时必须保持宿主无关：接收 `egui::Ui` 与本库 `Theme`，复用现有组件，将窗口、输入、渲染及重绘调度交给宿主。不得把 eframe、winit 或某个渲染器引入 ECML 运行时依赖。eframe 只是示例启动器。bevy_egui 等宿主需要兼容的 egui 版本并处理重绘请求；目前回调没有 Send/Sync 约束，集成时应考虑主线程存储。当前尚未提供可运行的 Bevy 集成示例。

ECML 必须保持可选；未来新增的解析器或监听依赖应通过 feature 隔离，并按需限制平台。纯 Rust 组件用户不应被要求采用 ECML 或额外的公开布局 API。

ECML 是 Vue 风格的自定义标记语言，不承诺 XML/HTML/Vue 兼容，不实现 CSS 布局、Rust 代码生成、反射或脚本环境。不得通过字符串暴露任意字段或 Rust 函数。

## 模块职责

| 文件 | 职责 |
| --- | --- |
| `src/ecml/mod.rs` | 只声明模块和重新导出，不放实现 |
| `src/ecml/runtime.rs` | 公开值与绑定类型、builder、View 生命周期、渲染、model 写回和事件分发 |
| `src/ecml/parser.rs` | 标记解析、源码偏移、语法与输入限制，不访问应用状态 |
| `src/ecml/template.rs` | 注册项及 schema 校验，编译为类型明确的可渲染节点 |
| `src/ecml/source.rs` | 来源选择、调用方宏展开和原生后台文件轮询 |
| `src/ecml/tests.rs` | 语法、绑定、布局、交互、身份及热更新回归测试 |

流程为：来源 → 解析节点 → schema/绑定校验 → 可渲染节点 → 现有组件调用。仅在加载或替换模板时解析，不逐帧解析。校验时将 binding 和 handler 名称解析为索引，绘制时直接使用。重组内部文件时应保留公开的 `egui_component::ecml::*` 路径。

## 语言与命名约定

- 组件标签必须与公开组件模块名一致，例如 `text_edit`，不添加 `input` 等别名。模块存在不代表自动支持 ECML，必须有显式适配。
- `layout` 和 `fragment` 是内置保留结构：`layout` 直接映射 egui，`fragment` 只分组、不创建布局。不得仅为标记语言添加公开 Row/Column 组件，也不接受 `row`/`column` 别名。
- 保留 egui 字段名和 snake_case。结构化属性使用 `item_spacing.x` 这样的点号路径，不使用 `gap_x`、`item_spacing_x`。
- 解析器保留完整属性名；schema 显式定义每个支持路径的类型、默认值及映射。UI 点号属性不等于嵌套状态绑定或任意 Rust 反射。
- 指令使用 `e-`，动态属性使用 `:`，事件使用 `@`。当前表达式仅包含已注册名称、文本插值、条件（`!`、`&&`、`||`、`$contains_pointer.<id>`、`$contains_focus.<id>`），以及样式属性中的 `cond ? value : value`（见“动态样式”）；计算放在 Rust getter 中。
- 未知标签/属性、重复项、非法值、未注册名称和类型/读写能力不匹配必须报错，不得静默忽略未支持语法或进行有损转换。

自定义语法要求一个根节点，属性值必须带引号；不带值的 `e-else` 是唯一例外。支持自闭合节点、文本、节点内部注释，以及五种具名实体（`amp`、`lt`、`gt`、`quot`、`apos`）。与 HTML 相同，不构成 `&名称;` 的 `&` 按字面处理，所以条件里的 `a && b` 不需要转义。名称段使用 ASCII 字母或下划线开头，后续允许字母、数字、下划线和连字符；属性路径可用点号连接名称段，并可带 `:`/`@` 前缀。模板上限为 256 KiB、64 层节点。错误包含文件、行、列；文本插值错误目前定位到所属节点。

## 布局约定

布局设置直接映射 egui，不另造布局引擎。新增属性时必须同时明确子节点排列和空间分配行为。

| 设置 | 当前行为 |
| --- | --- |
| `direction` | 默认 `top_down`，另支持 `left_to_right` |
| `cross_align` | 默认 `min`，另支持 `center`、`max`，映射 egui 对齐 |
| `item_spacing.x`、`item_spacing.y` | 有限非负逻辑点；未指定的轴分别继承父 Ui |
| 作用域 | 间距修改仅影响当前布局及后代，不影响兄弟布局或外部 Rust Ui |
| 横向空间分配 | 初始高度为 `ui.spacing().interact_size.y`，随内容增长，与 `Ui::horizontal` 一致 |
| 纵向空间分配 | 使用可用宽度，向父级报告实际占用区域，不隐式拉伸全部子节点 |

方向变化不改变物理 x/y 轴含义。主轴/交叉轴 justify 和 wrap 保持关闭。布局不额外增加滚动、裁剪或内容缩小，沿用 egui 溢出和父级裁剪行为。未指定宽度的 text_edit 仍使用可用宽度。不要误将全部剩余高度分给横向布局，否则可能把后续控件推到窗口外。

反向排列、主轴对齐、justify、wrap 和布局容器 width/height 尚未支持。添加尺寸能力前，明确它表示期望、最小、最大还是固定尺寸。滚动区域、网格和面板需要独立约定，不应隐式塞进 `layout`。

## 状态、交互与身份

状态属于调用方。View 保存访问器和 handler，不持有字段的长期可变引用，也不维护整个应用状态的 JSON 副本。`.model()` 注册返回值的 getter 与接收拥有值的 setter；getter 可以 clone 一个字段，也可以返回计算值（如"全部完成"，其 setter 作用于每一行）。`Value` 拥有自己的数据，所以读取最多一次 clone，与改为返回值之前相同；`.bind()` 注册只读 getter；`.on()` 显式注册 handler。类型校验不得依赖执行 getter。

当前值类型为 String、bool、i32、i64；可编辑控件分别要求 text_edit 的 String，以及 switch 和 checkbox 的 bool。getter 按需执行，没有依赖追踪和缓存，不应产生业务副作用。setter 只应同步更新状态；外部副作用放在显式 handler 中，handler 不应执行阻塞绘制的长任务。

一个 View 表示一个 Context/viewport 中的一个挂载实例，每 pass 调用一次。多个挂载位置使用独立 View。必须保留以下更新约定：

1. 新帧首次 show 时采用有效的待替换模板。
2. 绘制组件，立即写回 model 编辑。
3. 收集点击，遍历结束后按组件树顺序执行 handler；handler 能读到本次遍历的 model 更新。
4. 编辑或 handler 执行后请求重绘。后续 pass 展示更新后的状态，不重复提交 model 写回或 handler；这些 pass 中 text_edit 使用只读模式，避免编辑重放。

这是每帧最多一次提交，不是对 egui discarded pass 的事务回滚。同一帧的不同 pass 之间不能替换模板树。

显式 ID 必须非空且在 View 内唯一。身份由实例命名空间、viewport、显式 ID（或自动结构路径）和标签共同组成。兼容的显式标识控件移动后保留焦点/编辑状态；改变标签会改变身份。自动 ID 不保证结构修改后稳定。删除再插入节点不保证清空 egui 内存。未来的数值编辑器需要把 `-`、`1.` 等中间文本与业务值分开保存。应用级表单草稿、保存和取消仍由 Rust 管理。

## 来源、热更新与诊断

`ecml::source!("path")` 相对调用方的 `CARGO_MANIFEST_DIR` 定位，并在调用方编译上下文选择分支：

| 调用方配置 | 来源 |
| --- | --- |
| 原生平台，开启 debug assertions | 文件与热更新 |
| 原生平台，关闭 debug assertions | 编译期 `include_str!` 嵌入 |
| WASM，无论是否开启 debug assertions | 编译期嵌入 |

`Source::inline` 与原生 `Source::file` 可显式覆盖。宏负责资源打包，不负责语义校验；`.load()` 仍在运行时解析和校验。首次失败返回 `Error`，目前没有启动错误占位视图。

原生后台线程每 150ms 轮询、比较内容并解析变更，只保留最新候选，绝不访问 State 或 handler。首次 show 提供用于唤醒的 Context/viewport。UI 线程在下一帧边界校验候选，仅成功时替换模板树。无效修改或读取失败保留旧树，并通过错误窗口和 `last_error()` 暴露诊断；有效替换会清除错误。状态和注册 handler 保留。View drop 时停止并回收线程。`View::replace(text)` 同样暂存替换，但该显式调用的解析是同步的。

将轮询替换为系统监听时，也必须保留有界工作、空闲唤醒、原子保存恢复和最新候选行为。文件与嵌入来源共用校验和渲染路径。浏览器热更新需要宿主传输机制，不能依赖原生文件轮询。

## 循环（e-for）

本节定义 `e-for` 的约定。语法说明和可编辑 demo 见 Gallery 的 ECML 页面；修改下述行为前先更新本文。

### 模板语法

```html
<layout e-for="todo in todos" id="todo_row" direction="left_to_right" item_spacing.x="8">
    <checkbox e-model="todo.done" />
    <text>{{ todo.title }}</text>
    <button variant="ghost" @click="todo.remove">Delete</button>
</layout>
```

- `e-for="<别名> in <列表名>"` 重复所在元素自身，可用于除根节点外的任意元素。别名与列表名遵循名称段规则。
- key 不在模板中声明，由 Rust 注册列表时提供，避免扩展表达式语言，也避免把位置当作 key。
- `别名.名称` 只解析为该列表显式注册的行绑定或行 handler，不是字段反射或嵌套状态路径。未加前缀的名称仍解析为根注册项。
- 行事件写作 `@click="todo.remove"`，不支持 `remove(todo)` 等调用语法；handler 所属行由别名决定。
- 不暴露 index。序号等派生值放在 Rust 行 getter 中计算。

### Rust 注册

```rust
.list(
    "todos",
    |s| Box::new(s.todos.iter().filter(|t| s.filter.matches(t))), // 读：Rust 负责过滤与排序
    |s, id| s.todos.iter_mut().find(|t| t.id == *id),             // 写：按 key 定位
    |t| t.id,                                                     // key：Clone + Eq + Hash + Debug + 'static
    |row| row
        .model("done", |t| t.done, |t, v| t.done = v)
        .bind("title", |_s, t| t.title.clone())
        .bind("selected", |s, t| s.selected == Some(t.id))
        .on("remove", |s, id| s.todos.retain(|t| t.id != *id)),
)
```

- 读取访问器返回借用 `&S` 的迭代器（`Box<dyn Iterator<Item = &T>>`），可在 Rust 中过滤、排序，不要求底层是 Vec。读取必须保持返回迭代器。key 需要实现 `Debug`，以便报告重复 key。
- 写回与 handler 只通过 key 定位行：model 写回调用查找访问器（`for<'a> Fn(&'a mut S, &K) -> Option<&'a mut T>`），返回 `None` 时丢弃该写回并请求重绘；行 handler 为 `FnMut(&mut S, &K)`。不得使用位置定位，因为先执行的 handler 可能已增删行。
- 行 `bind` getter 接收 `(&S, &T)`，用于依赖全局状态的派生值；行 `model` getter 返回 `&T` 内借用值，setter 只接收 `&mut T` 并同步写回。
- 行注册项名称在该列表内唯一；列表名与根绑定、handler 共用命名空间，不得重名。

### 运行时语义

- 遍历期间迭代器借用 `&S`。循环体内的所有 model 写回（行 model 与根 model）先缓冲，在该循环遍历结束后按树顺序提交：行写回经查找访问器，根写回经根 setter。
- 因此同一 pass 中，写回之后、同一循环内的节点仍读到旧值。写回会请求重绘，下一 pass 显示新值；这是已接受的取舍，用于避免可变借用冲突和逐行重新查找。
- 点击仍在整棵树遍历结束后按组件树顺序执行，行事件携带克隆的 key。原有"每帧最多一次提交"和后续 pass 只读的约定不变。
- 重复 key 无法在编译期发现。运行时只渲染首个出现的行，跳过其后重复行，并通过错误窗口和 `last_error()` 报告列表名与重复 key，不得静默。该运行时错误每个 pass 重新计算，key 恢复唯一后自动清除；`last_error()` 优先返回模板错误。内部以 `dyn Any` 擦除行类型，缓冲写回和行事件中的 key 克隆进 `Rc`。
- 不做虚拟化；长列表的滚动由独立约定的 `scroll_area` 负责。

### 身份

- 行身份 = 循环节点身份（显式 `id` 或自动结构路径）+ 行 key 的哈希；行内节点在其下继续使用显式 ID 或行模板内结构路径。
- 因此增删、排序、中间插入行不会影响其他行的焦点和编辑状态。
- 建议在 `e-for` 元素上设置显式 `id`；否则热更新调整外层结构会改变循环身份，行内焦点随之丢失。
- 显式 ID 仍在整个 View 内唯一，行模板中的 ID 只计一次，运行时与行 key 组合。v1 中 `View::response(id)` 不返回循环内节点；如需要，以后另加按列表与 key 查询的 API。

### 编译期错误（v1）

以下情况必须报错并给出位置：未注册的列表；`e-for` 值格式错误；别名与根注册项或外层别名重名；在循环外使用别名；`别名.名称` 或 `别名.列表` 未注册；对只读行绑定使用 `e-model`；值类型不匹配；根节点使用 `e-for`；行注册项重名；列表名与同一层的绑定或 handler 重名；`e-for` 与条件指令位于同一元素（见下文）。

### 嵌套循环

嵌套沿用同样的显式注册，不使用反射：行 builder 注册子列表，`e-for="item in group.items"` 中的 `group.items` 解析到该注册项，与 `group.name` 解析到行绑定的方式相同。

```html
<layout e-for="group in groups" id="group">
    <text size="heading">{{ group.name }}</text>
    <layout e-for="item in group.items" id="item" direction="left_to_right">
        <text>{{ item.label }}</text>
        <button @click="item.remove">Delete</button>
    </layout>
</layout>
```

```rust
.list("groups", |s| Box::new(s.groups.iter()),
      |s, name| s.groups.iter_mut().find(|g| g.name == *name), |g| g.name.clone(),
      |group| group
          .bind("name", |_, g| g.name.clone())
          .list("items",
                |_s, g| Box::new(g.items.iter()),          // 读：(&S, &父行)
                |g, id| g.items.iter_mut().find(|i| i.id == *id), // 查找：在 &mut 父行内
                |i| i.id,
                |item| item
                    .bind("label", |_, i| i.label.clone())
                    .on("remove", |s, (group, id)| { /* key 路径 */ })))
```

- 子列表的读取访问器接收 `(&S, &T)`，`T` 为父行；查找访问器接收 `&mut T` 和子 key，因此写回从根开始逐层定位。子列表行的 key 类型是 key 路径 `(父路径, 子 key)`：两层时行 handler 接收 `(&mut S, &(K1, K2))`，三层时为 `&((K1, K2), K3)`。key 函数仍只返回子行自己的 key。
- 模板必须写明注册名：`{{ item.label }}`，而不是 `{{ item }}`。
- `别名.列表` 可以引用作用域内任意别名，不限于最内层。内层循环仍可访问外层别名（如在 item 循环内使用 `group.name`）；内层别名不得与外层别名重名。
- 最外层循环内任意位置的写回都缓冲到该循环结束。重复 key 只在同一父行的子行之间检查，同一子 key 可出现在不同父行下。行身份逐层叠加：循环身份 + 外层 key 哈希 + 内层循环身份 + 内层 key 哈希。

## 条件分支（e-if）与 fragment

```html
<layout direction="left_to_right" item_spacing.x="8">
    <checkbox e-model="todo.done" />
    <fragment e-if="todo.editing">
        <text_edit id="edit" e-model="todo.draft" />
        <button @click="todo.save">Save</button>
    </fragment>
    <text e-else-if="todo.done">{{ todo.title }}（已完成）</text>
    <text e-else>{{ todo.title }}</text>
</layout>
```

- `e-if` 和 `e-else-if` 接收一个条件，规则与 `:disabled` 相同：`cond := all ("||" all)*`，`all := term ("&&" term)*`，其中 term 是已注册的 bool（根名称或 `别名.名称`）、`$contains_pointer.<id>` 或 `$contains_focus.<id>`，可用 `!` 取反。`&&` 优先级更高，不支持括号；`!` 只作用于一个 term。比较等计算放在 Rust 的 `bind` getter 中。`e-else` 是语法中唯一不带值的属性。
- `e-else-if` 和 `e-else` 必须紧跟在带 `e-if` 或 `e-else-if` 的兄弟元素之后，中间只允许注释。一条分支链最多渲染第一个为真的分支，`e-else` 结束分支链。
- 条件在每个 pass 绘制到分支链时求值一次。循环外的写回是立即的，后面的条件在同一 pass 就能看到；循环内的写回是缓冲的，条件要到下一 pass 才看到。pass 之间模板结构不变，只有选中的分支会变。
- 隐藏的分支不绘制、不占空间，`View::response(id)` 对其中节点返回 `None`。每个分支都是独立的模板元素，有自己的结构路径，因此切换分支不会改变兄弟节点的自动 ID。隐藏与移除一样不保证清除 egui 记忆，获得焦点的控件被隐藏时会失去焦点。显式 ID 仍在 View 内唯一，跨分支也不得重复。
- 根节点可以使用 `e-if`（条件为假时什么都不画）；`e-else` 和 `e-else-if` 不能用于根节点，因为根节点没有兄弟元素。
- 一个元素最多带 `e-if`、`e-else-if`、`e-else` 之一。条件指令与 `e-for` 不能位于同一元素，应拆成内外两层；不需要布局时用 fragment。按行过滤应放在列表的读取访问器中。

`fragment` 与 `layout` 一样是内置保留结构：它只分组，不创建布局。子元素直接绘制到父级 Ui 中，继承父级方向和间距，因此横向 layout 中的 fragment 会让子元素留在同一行。它只接受 `id` 以及 `e-if`、`e-else-if`、`e-else`、`e-for` 指令，不接受文本内容。`id` 用于循环中标识 fragment（行身份 = fragment 身份 + key）；子元素仍使用各自的显式 ID 或结构路径。fragment 可以作为根节点，因此模板可以向宿主 Ui 绘制多个顶层元素。不提供 `<>` 简写。

以下情况必须报错并给出位置：`e-else`/`e-else-if` 前面没有条件兄弟元素，或用于根节点；`e-else` 带值；条件为空、未注册或不是 bool；同一元素上有多个条件指令；条件指令与 `e-for` 同时使用；fragment 上有其他属性或文本。

## 动态样式

```html
<text :color="todo.done ? $theme.colors.gray.l6 : $theme.colors.fg.base"
      :strikethrough="todo.done">{{ todo.title }}</text>
<button :variant="showing_all ? 'outline' : 'ghost'" color="$theme.colors.red">All</button>
<button :variant="!enabled ? 'ghost' : at_limit ? 'surface' : 'solid'">Chained</button>
```

- 样式属性可以是常量，也可以加 `:` 前缀写表达式。常量是一个裸单词（`variant="ghost"`）或一个主题路径（`color="$theme.colors.gray.l6"`）。同一元素不能同时写 `name` 和 `:name`。
- 表达式语法刻意保持很小：`expr := value | cond "?" value ":" expr`。`value` 是带引号的单词（`'ghost'`，因为属性本身用双引号，所以用单引号）或主题路径；`cond` 是与 `e-if` 相同的条件（用 `&&` 和 `||` 连接的 term）。以 `$theme` 之外的 `$` 名字开头的表达式视为以条件开头。条件成立时取的值只能是单个值，连写发生在 else 位置，取第一个为真的条件。bool 属性（`underline`、`strikethrough`、`disabled`）接受 `true`/`false`，写成 `:name` 时直接写条件。不支持算术、比较和函数调用，计算放在 Rust 的 `bind` getter 里。
- `$theme` 是保留关键字，路径与 Rust 的 `Theme` 字段一致：`$theme.colors.<色阶>.<foreground|l0..l9>` 和 `$theme.colors.<bg|fg>.<base|subtle|accent>` 是颜色，`$theme.colors.<色阶>` 是整个色阶（用于 button 的 `color`）。色阶有 primary、gray、red、green、blue、yellow、orange。路径在编译模板时与一张固定的表匹配（不做反射），绘制时在传给 `View::show` 的 Theme 里取值，所以切换明暗主题不需要重新编译。其他 `Theme` 字段暂未开放。
- 每个值都在编译期按属性检查类型：单词必须是该属性的可选值之一，颜色必须是颜色路径，色阶必须是色阶路径。Rust 绑定暂时不能返回颜色或 variant。
- 目前支持：text 的 `size`、`color`、`underline`、`strikethrough`；button 的 `size`、`variant`、`color`；text_edit 的 `size`、`variant`；switch 和 checkbox 的 `size`。text 的线宽为 1 点，颜色取文字颜色，对应 `text::Style::underline`/`strikethrough`。layout 属性和 `width` 仍只接受常量。
- 表达式在每次绘制元素时求值，读取规则与 `:disabled` 相同：在循环内，缓冲的写入在下一个 pass 才可见。样式不会改变身份。

## 文本输入事件与焦点

```html
<text_edit id="new" e-model="draft" autofocus="true" @submit="add" />
<text_edit e-if="todo.editing" id="edit" e-model="todo.draft" autofocus="true"
           @submit="todo.save" @blur="todo.save" @cancel="todo.cancel" />
```

| 事件 | 有焦点的 text_edit 因以下原因失去焦点时触发 |
| --- | --- |
| `@submit` | Enter（仅单行） |
| `@cancel` | Esc |
| `@blur` | 其他原因，如点击别处或按 Tab |

- 每次失去焦点只触发一个 handler：注册了对应的专用事件就触发它，否则触发 `@blur`。因此 `@submit="save" @blur="save"` 只保存一次；注册了 `@cancel` 时，按 Esc 不会触发 `@blur` 的保存。与浏览器不同，egui 中 Enter 和 Esc 都会结束编辑。检测沿用 egui 的惯用法（`lost_focus()` 加上本帧按下的键），与 `Form` 相同；用于确认输入法组合的 Enter 不会触发提交。egui 会让 `lost_focus()` 连续两帧为真，因此在每帧只有一个 pass 时（如 Web），View 会跳过上一帧已报告过的失焦，除非该输入框在此期间重新获得过焦点；请保留单 pass 回归测试。
- handler 与 `@click` 一样无参数，可以是根 handler，也可以是行 handler（如 `todo.save`）。它们与点击一起在遍历结束后按树顺序执行，且只在每帧第一个 pass 收集；同一帧输入的内容（以及循环中缓冲的写回）会先提交。禁用或只读的输入框不触发事件。
- 带 `@submit` 的 text_edit 按 Enter 后保持焦点（重新请求焦点），因此新建输入框可以连续添加多项。如果 handler 隐藏了该输入框，它在下一个 pass 不再存在，焦点随之结束。不带 `@submit` 时保持 egui 默认行为。
- `autofocus="true"` 在控件出现时请求焦点：View 首次显示、`e-if` 切换到该分支或循环中新增该行时。View 记录上一帧绘制过的 autofocus 控件；保持身份不变的热更新不会重新聚焦。同一帧有多个控件出现时，最后绘制的获得焦点。禁用的输入框不会被聚焦。焦点请求会在下一帧绘制该输入框之前再执行一次：在 Web 上，首次绘制时未获得焦点的输入框可能丢失迟到的焦点请求，因为文本代理只为绘制时已获得焦点的输入框接收键盘输入。
- ECML 暂不支持多行输入。以后为 `text_edit` 加入多行模式（映射到 `render_multiline`）时，Enter 用于换行，多行输入框上的 `@submit` 会报错，`@cancel` 和 `@blur` 含义不变。仅单行或仅多行的属性按模式分别校验。

## 指针状态与可见性

```html
<layout e-for="todo in todos" id="todo" direction="left_to_right">
    <text>{{ todo.title }}</text>
    <button :visible="$contains_pointer.todo || $contains_focus.todo"
            aria_label="Delete {{ todo.title }}" @click="todo.remove">Delete</button>
</layout>
```

- `$contains_pointer.<id>` 是内置条件，凡是可以写 bool 名字的地方都能用（`e-if`、`:disabled`、`:visible`、bool 属性以及样式表达式中的 `cond ?`），也可以用 `!` 取反。它对应 egui 的 `Response::contains_pointer()`：鼠标在该元素的区域内时为真，在其中的子控件上也算。egui 的 `hovered()` 在子控件上为假，一行里的按钮会在鼠标移到它上面时立刻消失。
- 即时模式下，元素的区域要画完才知道，所以 View 在每次 show 时记录所有带 id 的元素的区域，条件用上一次记录的区域和本帧的鼠标位置比较（`Context::rect_contains_pointer`）。祖先、前面和后面的元素都按同样方式工作，都晚一帧；鼠标移动时 egui 会重画，所以看不出延迟。上一次 show 没画出的元素（新出现的、被 `e-if` 隐藏的）结果为假。
- id 在使用条件的位置必须对应唯一的元素：要么在循环外，要么在同样包含这个条件的循环里（同一行；嵌套循环的外层行也可以）。在循环外或另一个循环里引用某个循环里的元素，会在编译期报错；未知 id、fragment（没有自己的区域）和其他 `$` 名字也会报错。
- 反馈循环：如果由 `$contains_pointer` 引起的变化会移动或改变它所读取的元素（例如用 `e-if` 在它前面插入元素，或换成更宽的 variant），元素可能离开鼠标，条件变为假，布局恢复，鼠标又落在元素上，于是每帧来回闪烁。这类变化应保持不影响布局：用 `:visible` 代替 `e-if`，改颜色或换尺寸相同的 variant。
- 所有元素（包括 fragment）都支持 `visible`/`:visible`。为假时元素仍占位置，但不绘制也不响应，与 egui 的 `Ui::set_invisible` 相同（CSS 的 `visibility: hidden`）；`e-if` 则移除元素（CSS 的 `display: none`）。不可见的控件同时也是禁用的，不能获得焦点；只在 hover 时显示的控件，除非条件里也用了 `$contains_focus`，否则无法用键盘访问。
- `$contains_focus.<id>` 在键盘焦点位于该元素内的控件上时为真，类似 CSS 的 `:focus-within`，id 规则与 `$contains_pointer` 相同。绘制时，View 会记录每个带 id 的元素内是否有控件 `has_focus()`，条件读取上一次 show 的结果，晚一帧。写成 `:visible="$contains_pointer.row || $contains_focus.row"` 时，Tab 到这一行的第一个控件就会显示隐藏的控件，再按一次 Tab 就能到达它；Tab 离开这一行后它再次隐藏。被隐藏的控件不能是这一行唯一可聚焦的控件，否则焦点永远进不了这一行。
- 不可见的控件仍然有 AccessKit 节点，并标记为禁用：egui 会给每个可聚焦的控件建节点，也没有提供把它移出无障碍树的方法。会列出控件的读屏器可能把隐藏的控件显示为不可用。
- 应用也可以在 Rust 里读取上一次 show 的 response：循环外的元素用 `View::response(id)`，在 `show` 之前复制到 state，再用 `.bind` 暴露。行内元素的 response 暂未公开。

## text 的点击事件

```html
<text id="title" @double_click="todo.edit">{{ todo.title }}</text>
```

- text 上的 `@click` 和 `@double_click` 对应 egui 的 `Response::clicked()` 和 `double_clicked()`，名字跟随 egui，而不是 HTML 的 `dblclick`。handler 不带参数，执行方式与 button 的 `@click` 相同：可以是根或行 handler，在遍历结束后、只在每帧的第一个 pass 执行。
- 带任一事件的 text 用 `text::Style::selectable(false)` 和 `.sense(Sense::click())` 绘制，所以双击不会同时选中单词。没有事件的 text 保持 egui 默认（可选中，`Sense::hover()`）。ECML 不单独提供 `selectable` 或 `sense` 属性。
- egui 把双击的第二次松开也报告为一次点击，所以双击会触发两次 `@click` 和一次 `@double_click`，与浏览器的 click、click、dblclick 一致。同一帧内 `@click` 先执行。单击不会为了区分双击而延迟；需要单击什么都不做时，只绑定 `@double_click`。
- 可点击的 text 可以用 Tab 获得焦点：按 Space 或 Enter 触发 `@click`（egui 的模拟点击），egui 用下划线标示焦点。双击没有对应的键盘操作，所以要配合另一个控件，比如 todo app 里的 Edit 按钮。
- 三击、右键点击和其他事件暂不支持；以后按同样方式、使用 egui 的名字添加。

## 无障碍名称

```html
<checkbox e-model="todo.done" aria_label="{{ todo.title }}" />
<button aria_label="Delete {{ todo.title }}" @click="todo.remove">Delete</button>
<switch e-model="dark" aria_label="Dark mode" />
```

- button、checkbox 和 switch 上的 `aria_label` 映射到组件的 `Style::aria_label`：读屏器读出的名称，替代可见文字，与 HTML 的 `aria-label` 相同，不会被绘制。可见文字不够明确（很多行都是“Delete”）或没有可见文字（没有内容的 checkbox、switch）时使用。
- 它与 `placeholder` 一样支持插值，所以没有 `:aria_label`；名称是文字，不是样式表达式。
- 其他元素不接受它：text 本身就会被读出，layout 和 fragment 不绘制控件，text_edit 在 Rust 中还没有覆盖名称的选项。为 text_edit 命名（Rust 的 `field` 已经能关联可见标签）以及其他 ARIA 属性留作未来工作。

## 扩展 ECML

新增组件或属性时，明确名称、类型、默认值、支持的动态绑定/事件、作用域和身份影响。添加 schema 校验及到现有组件或原生布局操作的渲染映射。提供有效用法测试及有意义的失败/行为回归测试。新语法在 Gallery 的 ECML 页面中配可编辑 demo 说明（模板放在 `examples/gallery/ecml_docs/templates/`，并补齐 en/zh/ja 文案）；约定变化时同步更新中英文设计指南。新语法先在本文设计，实现后语法说明迁到 Gallery，运行时约定与不变量保留在本文。只为完整应用场景新增 ECML 示例，命名为 `ecml_*`。

当前适配项为 layout、fragment、text、button、text_edit、switch、checkbox，以及 `e-for`、`e-if`、`e-else-if`、`e-else` 指令。事件有 `@click`、上述 text_edit 事件和 text 的点击事件。其他事件、多行输入、由 Rust 计算的样式值、其他 `$theme` 字段、嵌套状态路径、自定义 Rust 插槽、更多事件/类型/组件、derive 辅助、延迟 model 和浏览器热更新均属未来工作，不是已接受语法。derive 必须保持可选，并生成与显式注册相同的模型。

## 提交前验证

在仓库根目录运行：

```sh
cargo fmt --all --check
cargo test --features ecml --lib
cargo check --features ecml --example ecml_hello_world
cargo check --release --features ecml --example ecml_hello_world
cargo test --features ecml --example ecml_todo_app
cargo check --features ecml --lib --target wasm32-unknown-unknown
```

按需先安装 WASM target。UI 变更还应运行示例，确认默认窗口内控件可见，不能只验证 Response 存在。保留多 pass 输入/点击、焦点与光标、实例隔离、间距继承与作用域、无效模板恢复和空闲文件唤醒的回归覆盖。还需运行 `cargo test --example gallery`，它会校验 Gallery 中每个 ECML demo。中英文设计文档同时更新；约定与实现细节放在本文，语法参考与 demo 放在 Gallery，应用接入放在用户指南。
