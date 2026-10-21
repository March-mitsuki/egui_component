use egui_component_router::{PathPattern, Route};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentGroup {
    Actions,
    Inputs,
    Layout,
    Content,
    Identity,
    Feedback,
}
impl ComponentGroup {
    pub const ALL: [Self; 6] = [
        Self::Actions,
        Self::Inputs,
        Self::Layout,
        Self::Content,
        Self::Identity,
        Self::Feedback,
    ];
    pub fn label(self, locale: crate::i18n::Locale) -> &'static str {
        locale.text(match self {
            Self::Actions => "groups:actions",
            Self::Inputs => "groups:inputs",
            Self::Layout => "groups:layout",
            Self::Content => "groups:content",
            Self::Identity => "groups:identity",
            Self::Feedback => "groups:feedback",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Component {
    SegmentedControl,
    ColorPicker,
    Slider,
    Progress,
    Accordion,
    Avatar,
    Button,
    Card,
    Code,
    Dialog,
    Divider,
    Field,
    Form,
    Icon,
    IconButton,
    IconMenuButton,
    MenuButton,
    Navigation,
    ScrollArea,
    Select,
    Separator,
    Switch,
    Text,
    TextEdit,
    Toast,
    Tooltip,
}
impl Component {
    pub const ALL: &'static [Self] = &[
        Self::Accordion,
        Self::Avatar,
        Self::Button,
        Self::Card,
        Self::Code,
        Self::ColorPicker,
        Self::Dialog,
        Self::Divider,
        Self::Field,
        Self::Form,
        Self::Icon,
        Self::IconButton,
        Self::IconMenuButton,
        Self::MenuButton,
        Self::Navigation,
        Self::ScrollArea,
        Self::SegmentedControl,
        Self::Select,
        Self::Separator,
        Self::Slider,
        Self::Progress,
        Self::Switch,
        Self::Text,
        Self::TextEdit,
        Self::Toast,
        Self::Tooltip,
    ];
    pub fn slug(self) -> &'static str {
        match self {
            Self::Progress => "progress",
            Self::Slider => "slider",
            Self::ColorPicker => "color-picker",
            Self::SegmentedControl => "segmented-control",
            Self::Accordion => "accordion",
            Self::Avatar => "avatar",
            Self::Button => "button",
            Self::Card => "card",
            Self::Code => "code",
            Self::Dialog => "dialog",
            Self::Divider => "divider",
            Self::Field => "field",
            Self::Form => "form",
            Self::Icon => "icon",
            Self::IconButton => "icon-button",
            Self::IconMenuButton => "icon-menu-button",
            Self::MenuButton => "menu-button",
            Self::Navigation => "navigation",
            Self::ScrollArea => "scroll-area",
            Self::Select => "select",
            Self::Separator => "separator",
            Self::Switch => "switch",
            Self::Text => "text",
            Self::TextEdit => "text-edit",
            Self::Toast => "toast",
            Self::Tooltip => "tooltip",
        }
    }
    pub fn name(self, locale: crate::i18n::Locale) -> &'static str {
        match self {
            Self::Progress => locale.text("component.progress:name"),
            Self::Slider => locale.text("component.slider:name"),
            Self::ColorPicker => locale.text("component.colorPicker:name"),
            Self::SegmentedControl => locale.text("component.segmentedControl:name"),
            Self::Accordion => locale.text("component.accordion:name"),
            Self::Avatar => locale.text("component.avatar:name"),
            Self::Button => locale.text("component.button:name"),
            Self::Card => locale.text("component.card:name"),
            Self::Code => locale.text("component.code:name"),
            Self::Dialog => locale.text("component.dialog:name"),
            Self::Divider => locale.text("component.divider:name"),
            Self::Field => locale.text("component.field:name"),
            Self::Form => locale.text("component.form:name"),
            Self::Icon => locale.text("component.icon:name"),
            Self::IconButton => locale.text("component.iconButton:name"),
            Self::IconMenuButton => locale.text("component.iconMenuButton:name"),
            Self::MenuButton => locale.text("component.menuButton:name"),
            Self::Navigation => locale.text("component.navigation:name"),
            Self::ScrollArea => locale.text("component.scrollArea:name"),
            Self::Select => locale.text("component.select:name"),
            Self::Separator => locale.text("component.separator:name"),
            Self::Switch => locale.text("component.switch:name"),
            Self::Text => locale.text("component.text:name"),
            Self::TextEdit => locale.text("component.textEdit:name"),
            Self::Toast => locale.text("component.toast:name"),
            Self::Tooltip => locale.text("component.tooltip:name"),
        }
    }
    pub fn group(self) -> ComponentGroup {
        match self {
            Self::Progress => ComponentGroup::Feedback,
            Self::Slider => ComponentGroup::Inputs,
            Self::ColorPicker => ComponentGroup::Inputs,
            Self::SegmentedControl => ComponentGroup::Inputs,
            Self::Accordion => ComponentGroup::Layout,
            Self::Avatar => ComponentGroup::Identity,
            Self::Button => ComponentGroup::Actions,
            Self::Card => ComponentGroup::Layout,
            Self::Code => ComponentGroup::Content,
            Self::Dialog => ComponentGroup::Feedback,
            Self::Divider => ComponentGroup::Layout,
            Self::Field => ComponentGroup::Inputs,
            Self::Form => ComponentGroup::Inputs,
            Self::Icon => ComponentGroup::Content,
            Self::IconButton => ComponentGroup::Actions,
            Self::IconMenuButton => ComponentGroup::Actions,
            Self::MenuButton => ComponentGroup::Actions,
            Self::Navigation => ComponentGroup::Layout,
            Self::ScrollArea => ComponentGroup::Layout,
            Self::Select => ComponentGroup::Inputs,
            Self::Separator => ComponentGroup::Layout,
            Self::Switch => ComponentGroup::Inputs,
            Self::Text => ComponentGroup::Content,
            Self::TextEdit => ComponentGroup::Inputs,
            Self::Toast => ComponentGroup::Feedback,
            Self::Tooltip => ComponentGroup::Feedback,
        }
    }
    pub fn description(self, locale: crate::i18n::Locale) -> &'static str {
        match self {
            Self::Progress => locale.text("component.progress:description"),
            Self::Slider => locale.text("component.slider:description"),
            Self::ColorPicker => locale.text("component.colorPicker:description"),
            Self::SegmentedControl => locale.text("component.segmentedControl:description"),
            Self::Accordion => locale.text("component.accordion:description"),
            Self::Avatar => locale.text("component.avatar:description"),
            Self::Button => locale.text("component.button:description"),
            Self::Card => locale.text("component.card:description"),
            Self::Code => locale.text("component.code:description"),
            Self::Dialog => locale.text("component.dialog:description"),
            Self::Divider => locale.text("component.divider:description"),
            Self::Field => locale.text("component.field:description"),
            Self::Form => locale.text("component.form:description"),
            Self::Icon => locale.text("component.icon:description"),
            Self::IconButton => locale.text("component.iconButton:description"),
            Self::IconMenuButton => locale.text("component.iconMenuButton:description"),
            Self::MenuButton => locale.text("component.menuButton:description"),
            Self::Navigation => locale.text("component.navigation:description"),
            Self::ScrollArea => locale.text("component.scrollArea:description"),
            Self::Select => locale.text("component.select:description"),
            Self::Separator => locale.text("component.separator:description"),
            Self::Switch => locale.text("component.switch:description"),
            Self::Text => locale.text("component.text:description"),
            Self::TextEdit => locale.text("component.textEdit:description"),
            Self::Toast => locale.text("component.toast:description"),
            Self::Tooltip => locale.text("component.tooltip:description"),
        }
    }
    pub fn source(self) -> &'static str {
        match self {
            Self::Progress => include_str!("pages/progress.rs"),
            Self::Slider => include_str!("pages/slider.rs"),
            Self::ColorPicker => include_str!("pages/color_picker.rs"),
            Self::SegmentedControl => include_str!("pages/segmented_control.rs"),
            Self::Accordion => include_str!("pages/accordion.rs"),
            Self::Avatar => include_str!("pages/avatar.rs"),
            Self::Button => include_str!("pages/button.rs"),
            Self::Card => include_str!("pages/card.rs"),
            Self::Code => include_str!("pages/code.rs"),
            Self::Dialog => include_str!("pages/dialog.rs"),
            Self::Divider => include_str!("pages/divider.rs"),
            Self::Field => include_str!("pages/field.rs"),
            Self::Form => include_str!("pages/form.rs"),
            Self::Icon => include_str!("pages/icon.rs"),
            Self::IconButton => include_str!("pages/icon_button.rs"),
            Self::IconMenuButton => include_str!("pages/icon_menu_button.rs"),
            Self::MenuButton => include_str!("pages/menu_button.rs"),
            Self::Navigation => include_str!("pages/navigation.rs"),
            Self::ScrollArea => include_str!("pages/scroll_area.rs"),
            Self::Select => include_str!("pages/select.rs"),
            Self::Separator => include_str!("pages/separator.rs"),
            Self::Switch => include_str!("pages/switch.rs"),
            Self::Text => include_str!("pages/text.rs"),
            Self::TextEdit => include_str!("pages/text_edit.rs"),
            Self::Toast => include_str!("pages/toast.rs"),
            Self::Tooltip => include_str!("pages/tooltip.rs"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Doc {
    GettingStarted,
    Styling,
    Theming,
    Routing,
}
impl Doc {
    pub const ALL: &'static [Self] = &[
        Self::GettingStarted,
        Self::Styling,
        Self::Theming,
        Self::Routing,
    ];
    pub fn slug(self) -> &'static str {
        match self {
            Self::GettingStarted => "getting-started",
            Self::Styling => "styling",
            Self::Theming => "theming",
            Self::Routing => "routing",
        }
    }
    pub fn title(self, locale: crate::i18n::Locale) -> &'static str {
        match self {
            Self::GettingStarted => locale.text("doc.gettingStarted:title"),
            Self::Styling => locale.text("doc.styling:title"),
            Self::Theming => locale.text("doc.theming:title"),
            Self::Routing => locale.text("doc.routing:title"),
        }
    }
    pub fn description(self, locale: crate::i18n::Locale) -> &'static str {
        match self {
            Self::GettingStarted => locale.text("doc.gettingStarted:description"),
            Self::Styling => locale.text("doc.styling:description"),
            Self::Theming => locale.text("doc.theming:description"),
            Self::Routing => locale.text("doc.routing:description"),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Page {
    Home,
    ThemeGenerator,
    Docs(Doc),
    Component(Component),
    NotFound(String),
}
impl Route for Page {
    fn from_path(path: &str) -> Self {
        if path == "/theme-generator" {
            return Self::ThemeGenerator;
        }
        if path == "/" {
            return Self::Home;
        }
        if path == "/docs" {
            return Self::Docs(Doc::GettingStarted);
        }
        if path == "/components" {
            return Self::Component(Component::Button);
        }
        if let Some(params) = PathPattern::new("/docs/:page").unwrap().match_path(path) {
            if let Some(doc) = Doc::ALL
                .iter()
                .find(|doc| Some(doc.slug()) == params.get("page"))
            {
                return Self::Docs(*doc);
            }
        }
        if let Some(params) = PathPattern::new("/components/:name")
            .unwrap()
            .match_path(path)
        {
            if let Some(component) = Component::ALL
                .iter()
                .find(|component| Some(component.slug()) == params.get("name"))
            {
                return Self::Component(*component);
            }
        }
        Self::NotFound(path.into())
    }
    fn to_path(&self) -> String {
        match self {
            Self::Home => "/".into(),
            Self::ThemeGenerator => "/theme-generator".into(),
            Self::Docs(doc) => format!("/docs/{}", doc.slug()),
            Self::Component(component) => format!("/components/{}", component.slug()),
            Self::NotFound(path) => path.clone(),
        }
    }
}
/// Language is part of the route; Page remains independent of presentation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GalleryRoute {
    pub locale: crate::i18n::Locale,
    pub page: Page,
}
impl GalleryRoute {
    pub fn new(locale: crate::i18n::Locale, page: Page) -> Self {
        Self { locale, page }
    }
}
impl Route for GalleryRoute {
    fn from_path(path: &str) -> Self {
        let (prefix, tail) = path
            .strip_prefix('/')
            .unwrap_or(path)
            .split_once('/')
            .unwrap_or((path.trim_start_matches('/'), ""));
        if let Some(locale) = crate::i18n::Locale::parse(prefix) {
            Self::new(locale, Page::from_path(&format!("/{tail}")))
        } else {
            Self::new(crate::i18n::Locale::En, Page::from_path(path))
        }
    }
    fn to_path(&self) -> String {
        let page = self.page.to_path();
        if page == "/" {
            format!("/{}", self.locale.code())
        } else {
            format!("/{}{page}", self.locale.code())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn localized_pages_round_trip_and_legacy_urls_default_to_english() {
        use crate::i18n::Locale;
        for locale in Locale::ALL {
            for page in [Page::Home, Page::ThemeGenerator]
                .into_iter()
                .chain(Doc::ALL.iter().copied().map(Page::Docs))
                .chain(Component::ALL.iter().copied().map(Page::Component))
                .chain([Page::NotFound("/missing".into())])
            {
                let route = GalleryRoute::new(locale, page);
                assert_eq!(GalleryRoute::from_path(&route.to_path()), route);
            }
            assert_eq!(
                GalleryRoute::from_path(&format!("/{}/components", locale.code())).page,
                Page::Component(Component::Button)
            );
            assert_eq!(
                GalleryRoute::from_path(&format!("/{}/docs", locale.code())).page,
                Page::Docs(Doc::GettingStarted)
            );
        }
        for path in ["/", "/components/button", "/docs/styling"] {
            let route = GalleryRoute::from_path(path);
            assert_eq!(route.locale, Locale::En);
            assert_eq!(route.page, Page::from_path(path));
            assert!(route.to_path().starts_with("/en"));
        }
        assert!(matches!(
            GalleryRoute::from_path("/fr/components/button").page,
            Page::NotFound(_)
        ));
    }
    #[test]
    fn language_navigation_preserves_search_and_browser_history() {
        use crate::i18n::Locale;
        use egui_component_router::{Location, Router, SearchParams};
        let original = GalleryRoute::new(Locale::En, Page::Component(Component::Select));
        let search = SearchParams::parse("section=usage&tag=a&tag=b");
        let mut router =
            Router::with_location(Location::new(original.clone(), search.clone()), || {}).unwrap();
        let translated = GalleryRoute::new(Locale::Ja, original.page.clone());
        router
            .navigate_with_search(translated.clone(), router.search_params())
            .unwrap();
        assert_eq!(
            router.location().to_url(),
            "/ja/components/select?section=usage&tag=a&tag=b"
        );
        router.back().unwrap();
        assert_eq!(router.current(), original);
        assert_eq!(router.search_params(), search);
        router.forward().unwrap();
        assert_eq!(router.current(), translated);
    }
    #[test]
    fn every_page_round_trips() {
        for page in [Page::Home, Page::ThemeGenerator]
            .into_iter()
            .chain(Doc::ALL.iter().copied().map(Page::Docs))
            .chain(Component::ALL.iter().copied().map(Page::Component))
        {
            assert_eq!(Page::from_path(&page.to_path()), page);
        }
    }
    #[test]
    fn fallback_and_section_roots() {
        assert_eq!(Page::from_path("/docs"), Page::Docs(Doc::GettingStarted));
        assert_eq!(
            Page::from_path("/components"),
            Page::Component(Component::Button)
        );
        for path in [
            "/missing",
            "/docs/missing",
            "/components/missing",
            "/components/button/extra",
        ] {
            assert_eq!(Page::from_path(path), Page::NotFound(path.into()));
        }
    }
}
