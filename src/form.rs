use std::any::{Any, TypeId};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

use dashmap::DashMap;

pub static FORM_CONTEXT: LazyLock<FormContext> = LazyLock::new(FormContext::new);

pub struct FormContext {
    // 键是 TypeId，值是被 Box 包裹的 Any
    forms: DashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Default for FormContext {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FormContainer<D, S> {
    pub data: D,
    pub state: S,
}
impl<D: Default, S: Default> Default for FormContainer<D, S> {
    fn default() -> Self {
        Self {
            data: D::default(),
            state: S::default(),
        }
    }
}

impl FormContext {
    pub fn new() -> Self {
        Self {
            forms: DashMap::new(),
        }
    }

    /// 获取或初始化特定类型的表单并进行修改
    pub fn modify_or_insert<D, S>(&self, f: impl FnOnce(&mut D, &mut S))
    where
        D: Any + Send + Sync + Default,
        S: Any + Send + Sync + Default,
    {
        // TypeId 是根据容器类型 FormContainer<D, S> 生成的
        let id = TypeId::of::<FormContainer<D, S>>();

        let mut entry = self
            .forms
            .entry(id)
            .or_insert_with(|| Box::new(FormContainer::<D, S>::default()));

        if let Some(container) = entry.value_mut().downcast_mut::<FormContainer<D, S>>() {
            f(&mut container.data, &mut container.state);
        }
    }

    /// 移除指定类型的表单数据
    pub fn remove<T: Any>(&self) {
        self.forms.remove(&TypeId::of::<T>());
    }
}

#[derive(Debug, Default, Clone)]
pub struct Number<T> {
    raw: String,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Number<T>
where
    T: FromStr + Display,
{
    pub fn new(value: T) -> Self {
        Self {
            raw: value.to_string(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn value(&self) -> Result<T, <T as FromStr>::Err> {
        self.raw.parse::<T>()
    }

    pub fn raw_mut(&mut self) -> &mut String {
        &mut self.raw
    }

    pub fn set_raw(&mut self, value: T) {
        self.raw = value.to_string();
    }
}
