# ECML 贡献者指南

[English](ECML_DESIGN.md) | [中文](ECML_DESIGN.zh.md)

本文定义可选 `ecml` feature 的设计目标、架构边界与行为约定。修改语言、绑定 API、渲染或热更新流程之前，请先阅读本文。应用集成与已支持语法见 [ECML 用户指南](examples/ecml/README.md)，仓库通用协作流程见[贡献指南](CONTRIBUTE.zh.md)。

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
- `layout` 是直接映射 egui 的内置保留结构。不得仅为标记语言添加公开 Row/Column 组件，也不接受 `row`/`column` 别名。
- 保留 egui 字段名和 snake_case。结构化属性使用 `item_spacing.x` 这样的点号路径，不使用 `gap_x`、`item_spacing_x`。
- 解析器保留完整属性名；schema 显式定义每个支持路径的类型、默认值及映射。UI 点号属性不等于嵌套状态绑定或任意 Rust 反射。
- 指令使用 `e-`，动态属性使用 `:`，事件使用 `@`。当前表达式仅包含已注册名称、文本插值以及 `:disabled` 的布尔取反；计算放在 Rust getter 中。
- 未知标签/属性、重复项、非法值、未注册名称和类型/读写能力不匹配必须报错，不得静默忽略未支持语法或进行有损转换。

自定义语法要求一个根节点，属性值必须带引号。支持自闭合节点、文本、节点内部注释，以及五种具名实体（`amp`、`lt`、`gt`、`quot`、`apos`）。名称段使用 ASCII 字母或下划线开头，后续允许字母、数字、下划线和连字符；属性路径可用点号连接名称段，并可带 `:`/`@` 前缀。模板上限为 256 KiB、64 层节点。错误包含文件、行、列；文本插值错误目前定位到所属节点。

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

状态属于调用方。View 保存访问器和 handler，不持有字段的长期可变引用，也不维护整个应用状态的 JSON 副本。`.model()` 注册返回借用的 getter 与接收拥有值的 setter；`.bind()` 注册只读 getter；`.on()` 显式注册 handler。类型校验不得依赖执行 getter。

当前值类型为 String、bool、i32、i64；可编辑控件分别要求 text_edit 的 String 和 switch 的 bool。getter 按需执行，没有依赖追踪和缓存，不应产生业务副作用。setter 只应同步更新状态；外部副作用放在显式 handler 中，handler 不应执行阻塞绘制的长任务。

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

## 扩展 ECML

新增组件或属性时，明确名称、类型、默认值、支持的动态绑定/事件、作用域和身份影响。添加 schema 校验及到现有组件或原生布局操作的渲染映射。提供有效用法测试及有意义的失败/行为回归测试；约定变化时同步更新用户指南和中英文设计指南。

当前适配项为 layout、text、button、text_edit、switch。条件分支、带 key 循环、嵌套状态路径、自定义 Rust 插槽、更多事件/类型/组件、derive 辅助、延迟 model 和浏览器热更新均属未来工作，不是已接受语法。循环需要先设计集合、行作用域、事件与 key。derive 必须保持可选，并生成与显式注册相同的模型。

## 提交前验证

在仓库根目录运行：

```sh
cargo fmt --all --check
cargo test --features ecml --lib
cargo check --features ecml --example ecml
cargo check --release --features ecml --example ecml
cargo check --features ecml --lib --target wasm32-unknown-unknown
```

按需先安装 WASM target。UI 变更还应运行示例，确认默认窗口内控件可见，不能只验证 Response 存在。保留多 pass 输入/点击、焦点与光标、实例隔离、间距继承与作用域、无效模板恢复和空闲文件唤醒的回归覆盖。中英文设计文档同时更新；实现细节放在本文，应用接入与语法参考放在用户指南。
