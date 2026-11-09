# 更新日志

在此记录 breaking changes 及对应的迁移方式。

## 0.4.0

### 不兼容变更

- `icon_button::Style` 的公开字段 `size: f32` 拆分为 `width: f32` 和 `height: f32`。直接读取 `style.size` 的代码应改为读取所需的宽度或高度。使用结构体字面量时，应将 `size` 替换为这两个字段。`.size(value)` 仍可同时设置宽高，`.width(value)` 和 `.height(value)` 可分别设置宽度和高度。

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
