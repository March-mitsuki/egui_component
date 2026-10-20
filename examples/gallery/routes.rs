use egui_component_router::{PathPattern, Route};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Component {
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
        Self::Avatar,
        Self::Button,
        Self::Card,
        Self::Code,
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
        Self::Select,
        Self::Separator,
        Self::Switch,
        Self::Text,
        Self::TextEdit,
        Self::Toast,
        Self::Tooltip,
    ];
    pub fn slug(self) -> &'static str {
        match self {
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
    pub fn name(self) -> &'static str {
        match self {
            Self::Avatar => "Avatar",
            Self::Button => "Button",
            Self::Card => "Card",
            Self::Code => "Code",
            Self::Dialog => "Dialog",
            Self::Divider => "Divider",
            Self::Field => "Field",
            Self::Form => "Form",
            Self::Icon => "Icon",
            Self::IconButton => "Icon Button",
            Self::IconMenuButton => "Icon Menu Button",
            Self::MenuButton => "Menu Button",
            Self::Navigation => "Navigation",
            Self::ScrollArea => "Scroll Area",
            Self::Select => "Select",
            Self::Separator => "Separator",
            Self::Switch => "Switch",
            Self::Text => "Text",
            Self::TextEdit => "Text Edit",
            Self::Toast => "Toast",
            Self::Tooltip => "Tooltip",
        }
    }
    pub fn group(self) -> &'static str {
        match self {
            Self::Avatar => "Identity",
            Self::Button => "Actions",
            Self::Card => "Layout",
            Self::Code => "Content",
            Self::Dialog => "Feedback",
            Self::Divider => "Layout",
            Self::Field => "Inputs",
            Self::Form => "Inputs",
            Self::Icon => "Content",
            Self::IconButton => "Actions",
            Self::IconMenuButton => "Actions",
            Self::MenuButton => "Actions",
            Self::Navigation => "Layout",
            Self::ScrollArea => "Layout",
            Self::Select => "Inputs",
            Self::Separator => "Layout",
            Self::Switch => "Inputs",
            Self::Text => "Content",
            Self::TextEdit => "Inputs",
            Self::Toast => "Feedback",
            Self::Tooltip => "Feedback",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Avatar => "A compact visual identity for people, teams, and accounts.",
            Self::Button => "Clear actions, consistent states, and a style for every context.",
            Self::Card => "A quiet surface for grouping related content and controls.",
            Self::Code => "Readable, selectable code with language-aware syntax highlighting.",
            Self::Dialog => "Focused surfaces for decisions that need a little more attention.",
            Self::Divider => "Create structure and breathing room between pieces of content.",
            Self::Field => "Bring labels, controls, and validation into a consistent layout.",
            Self::Form => "Coordinate drafts, validation, submission, and reset behavior.",
            Self::Icon => "Small, expressive symbols that bring clarity to your interface.",
            Self::IconButton => {
                "A compact action with an icon and customizable interaction states."
            }
            Self::IconMenuButton => "Compact icon triggers for contextual menus.",
            Self::MenuButton => "Keep related actions together in a contextual menu.",
            Self::Navigation => {
                "A focused, keyboard-friendly navigation item for sidebars and outlines."
            }
            Self::ScrollArea => "Give overflowing content its own carefully styled viewport.",
            Self::Select => "Let people choose from a compact set of options.",
            Self::Separator => "A subtle visual boundary that follows the surrounding layout.",
            Self::Switch => "A direct control for a setting that is either on or off.",
            Self::Text => "Typography with theme-aware colors, weights, and managed fonts.",
            Self::TextEdit => "Text inputs with flexible visual styles and password support.",
            Self::Toast => "Timely, lightweight feedback without interrupting the workflow.",
            Self::Tooltip => "Helpful context, revealed when it is needed.",
        }
    }
    pub fn source(self) -> &'static str {
        match self {
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
    pub fn title(self) -> &'static str {
        match self {
            Self::GettingStarted => "Getting started",
            Self::Styling => "Styling",
            Self::Theming => "Theming",
            Self::Routing => "Routing",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::GettingStarted => {
                "From your first dependency to your first component. A small API, ready for your next idea."
            }
            Self::Styling => "Start with a preset. Make every detail your own.",
            Self::Theming => "One design language. Light, dark, and everything you customize.",
            Self::Routing => "Typed navigation that feels at home on desktop and in the browser.",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Page {
    Home,
    Docs(Doc),
    Component(Component),
    NotFound(String),
}
impl Route for Page {
    fn from_path(path: &str) -> Self {
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
            Self::Docs(doc) => format!("/docs/{}", doc.slug()),
            Self::Component(component) => format!("/components/{}", component.slug()),
            Self::NotFound(path) => path.clone(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_page_round_trips() {
        for page in std::iter::once(Page::Home)
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
