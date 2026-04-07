use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(FrameModifier)]
pub fn frame_modifier_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident; // 结构体名称，如 Style

    let fields = if let Data::Struct(data) = input.data {
        if let Fields::Named(fields) = data.fields {
            fields.named
        } else {
            panic!("FrameModifier 仅支持具名字段的结构体");
        }
    } else {
        panic!("FrameModifier 仅支持结构体");
    };

    let has_frame = fields.iter().any(|f| f.ident.as_ref().unwrap() == "frame");
    if !has_frame {
        panic!("结构体必须包含名为 'frame' 的字段才能使用 FrameModifier");
    }

    let expanded = quote! {
        impl #name {
            pub fn shadow(mut self, shadow: egui::Shadow) -> Self {
                self.frame.shadow = shadow;
                self
            }

            pub fn fill(mut self, fill: egui::Color32) -> Self {
                self.frame.fill = fill;
                self
            }

            pub fn stroke(mut self, stroke: egui::Stroke) -> Self {
                self.frame.stroke = stroke;
                self
            }

            pub fn inner_margin(mut self, inner_margin: egui::Margin) -> Self {
                self.frame.inner_margin = inner_margin;
                self
            }

            pub fn outer_margin(mut self, outer_margin: egui::Margin) -> Self {
                self.frame.outer_margin = outer_margin;
                self
            }

            pub fn corner_radius(mut self, corner_radius: egui::CornerRadius) -> Self {
                self.frame.corner_radius = corner_radius;
                self
            }
        }
    };

    TokenStream::from(expanded)
}
