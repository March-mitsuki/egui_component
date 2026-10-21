use super::*;

fn bytes(name: &str) -> Vec<u8> {
    egui::FontDefinitions::default().font_data[name]
        .as_ref()
        .as_ref()
        .to_vec()
}

fn finish(ctx: &egui::Context) {
    let _ = ctx.end_pass();
}

#[test]
fn latest_update_wins_when_reverting_to_active_fonts_in_the_same_pass() {
    let manager = FontManager::new();
    let ctx = egui::Context::default();
    let regular = bytes("Hack");
    manager.replace_family("main", &[(&regular, 0)]).unwrap();
    manager.apply(&ctx);
    ctx.begin_pass(Default::default());
    manager
        .replace_family("main", &[(&bytes("Ubuntu-Light"), 0)])
        .unwrap();
    assert!(manager.apply(&ctx));
    manager.replace_family("main", &[(&regular, 0)]).unwrap();
    assert!(manager.apply(&ctx));
    finish(&ctx);
    ctx.begin_pass(Default::default());
    let family = manager.get_egui_font_family(&ctx, "main", FontWeight::Regular);
    ctx.fonts(|fonts| {
        let definitions = fonts.definitions();
        let primary = &definitions.families[&family][0];
        assert_eq!(definitions.font_data[primary].as_ref().as_ref(), regular);
    });
    assert!(!manager.apply(&ctx));
    finish(&ctx);
}

fn has_chinese(manager: &FontManager, ctx: &egui::Context) -> bool {
    let family = manager.get_egui_font_family(ctx, "main", FontWeight::Bold);
    ctx.fonts_mut(|fonts| fonts.has_glyph(&egui::FontId::new(16.0, family), '中'))
}

#[test]
fn replaces_faces_without_duplicates_and_skips_unchanged_updates() {
    let manager = FontManager::new();
    let ctx = egui::Context::default();
    let regular = bytes("Hack");
    manager.load_from_bytes("UI", &regular, 0).unwrap();
    assert!(manager.apply(&ctx));
    let original = manager.snapshot();
    manager.load_from_bytes(" ui ", &regular, 0).unwrap();
    assert!(!manager.apply(&ctx));
    assert!(Arc::ptr_eq(&original, &manager.snapshot()));

    manager
        .load_from_bytes("ui", &bytes("Ubuntu-Light"), 0)
        .unwrap();
    manager
        .load_from_bytes("ui", &bytes("NotoEmoji-Regular"), 0)
        .unwrap();
    let state = manager.state.read();
    let faces = &state.families[&FamilyKey::new("ui")];
    assert_eq!(faces.len(), 2); // Light and Regular, with Regular replaced.
    assert_eq!(faces[1].face.weight, FontWeight::Regular);
    assert_eq!(faces[1].face.data.as_ref(), bytes("NotoEmoji-Regular"));
    drop(state);
    assert!(manager.apply(&ctx));
    let updated = manager.snapshot();
    let key = manager.font_key(&FamilyKey::new("ui"), FontWeight::Regular, false);
    assert!(!Arc::ptr_eq(
        &original.definitions.font_data[&key],
        &updated.definitions.font_data[&key]
    ));
}

#[test]
fn batch_replacement_validates_every_face_before_committing() {
    let manager = FontManager::new();
    let ctx = egui::Context::default();
    let regular = bytes("Hack");
    let light = bytes("Ubuntu-Light");
    manager.load_from_bytes("ui", &regular, 0).unwrap();
    manager.apply(&ctx);
    assert!(
        manager
            .replace_family("ui", &[(&light, 0), (b"invalid", 0)])
            .is_err()
    );
    assert!(
        manager
            .replace_family("ui", &[(&regular, 0), (&regular, 0)])
            .is_err()
    );
    assert!(manager.replace_family("ui", &[]).is_err());
    assert!(manager.load_from_bytes("ui", &regular, usize::MAX).is_err());
    assert!(manager.load_from_bytes("  ", &regular, 0).is_err());
    assert!(!manager.apply(&ctx));
    assert_eq!(
        manager.state.read().families[&FamilyKey::new("ui")][0]
            .face
            .data
            .as_ref(),
        regular
    );

    manager.replace_family("ui", &[(&light, 0)]).unwrap();
    assert!(manager.apply(&ctx));
    assert_eq!(
        manager.state.read().families[&FamilyKey::new("ui")].len(),
        1
    );
    assert_eq!(
        manager.state.read().families[&FamilyKey::new("ui")][0]
            .face
            .weight,
        FontWeight::Light
    );
}

#[test]
fn contexts_activate_independently_and_queries_never_return_pending_names() {
    let manager = FontManager::new();
    let first = egui::Context::default();
    let second = egui::Context::default();
    manager
        .load_from_bytes("custom", &bytes("Hack"), 0)
        .unwrap();

    first.begin_pass(Default::default());
    assert!(manager.apply(&first));
    assert!(!manager.apply(&first));
    assert_eq!(
        manager.get_egui_font_family(&first, "custom", FontWeight::Regular),
        egui::FontFamily::Proportional
    );
    finish(&first);
    first.begin_pass(Default::default());
    let family = manager.get_egui_font_family(&first, "custom", FontWeight::Regular);
    assert_ne!(family, egui::FontFamily::Proportional);

    second.begin_pass(Default::default());
    assert_eq!(
        manager.get_egui_font_family(&second, "custom", FontWeight::Regular),
        egui::FontFamily::Proportional
    );
    assert!(manager.apply(&second));
    finish(&second);
    second.begin_pass(Default::default());
    assert_eq!(
        manager.get_egui_font_family(&second, "custom", FontWeight::Regular),
        family
    );

    assert!(manager.remove_family("custom"));
    assert!(manager.apply(&first));
    // First still has the old active snapshot, despite queuing its removal.
    assert_eq!(
        manager.get_egui_font_family(&first, "custom", FontWeight::Regular),
        family
    );
    finish(&first);
    first.begin_pass(Default::default());
    assert_ne!(
        manager.get_egui_font_family(&first, "custom", FontWeight::Regular),
        family
    );
    assert_eq!(
        manager.get_egui_font_family(&second, "custom", FontWeight::Regular),
        family
    );
    assert!(manager.apply(&second));
    finish(&second);
    second.begin_pass(Default::default());
    assert_ne!(
        manager.get_egui_font_family(&second, "custom", FontWeight::Regular),
        family
    );
    finish(&first);
    finish(&second);
}

#[test]
fn late_fallback_load_and_removal_change_actual_glyph_availability() {
    let manager = FontManager::new();
    let ctx = egui::Context::default();
    manager.load_from_bytes("main", &bytes("Hack"), 0).unwrap();
    manager.set_fallback_chain(&["cjk", "CJK", "missing"]);
    manager.apply(&ctx);
    ctx.begin_pass(Default::default());
    assert!(!has_chinese(&manager, &ctx));
    // Use a nonzero collection index to exercise TTC face selection too.
    manager
        .load_from_bytes(
            "cjk",
            include_bytes!("../../examples/assets/NotoSansCJK-Regular.ttc"),
            1,
        )
        .unwrap();
    assert!(manager.apply(&ctx));
    assert!(!has_chinese(&manager, &ctx));
    finish(&ctx);
    ctx.begin_pass(Default::default());
    assert!(has_chinese(&manager, &ctx));
    let family = manager.get_egui_font_family(&ctx, "main", FontWeight::Bold);
    ctx.fonts(|fonts| {
        let definitions = fonts.definitions();
        let chain = &definitions.families[&family];
        assert_eq!(
            chain[0],
            manager.font_key(&FamilyKey::new("main"), FontWeight::Regular, false)
        );
        let cjk = manager.font_key(&FamilyKey::new("cjk"), FontWeight::Regular, false);
        assert_eq!(chain[1], cjk);
        assert_eq!(chain.iter().filter(|key| **key == cjk).count(), 1);
        assert_eq!(definitions.font_data[&cjk].index, 1);
    });
    manager.set_fallback_chain(&[]);
    manager.apply(&ctx);
    finish(&ctx);
    ctx.begin_pass(Default::default());
    assert!(!has_chinese(&manager, &ctx));
    manager.set_fallback_chain(&["cjk"]);
    manager.apply(&ctx);
    finish(&ctx);
    ctx.begin_pass(Default::default());
    assert!(has_chinese(&manager, &ctx));
    assert!(manager.remove_face("cjk", FontWeight::Regular, false));
    assert!(!manager.remove_face("cjk", FontWeight::Regular, false));
    manager.apply(&ctx);
    finish(&ctx);
    ctx.begin_pass(Default::default());
    assert!(!has_chinese(&manager, &ctx));
    finish(&ctx);
}

#[test]
fn regular_and_italic_faces_have_distinct_slots() {
    let regular = bytes("Hack");
    let mut italic = regular.clone();
    let parsed = ttf_parser::Face::parse(&italic, 0).unwrap();
    let os2 = parsed
        .raw_face()
        .table(ttf_parser::Tag::from_bytes(b"OS/2"))
        .unwrap();
    let selection_offset = os2.as_ptr() as usize - italic.as_ptr() as usize + 62;
    let selection = u16::from_be_bytes([italic[selection_offset], italic[selection_offset + 1]]);
    italic[selection_offset..selection_offset + 2]
        .copy_from_slice(&((selection | 1) & !64).to_be_bytes());
    let manager = FontManager::new();
    manager
        .replace_family("ui", &[(&regular, 0), (&italic, 0)])
        .unwrap();
    let snapshot = manager.snapshot();
    let normal_key = manager.font_key(&FamilyKey::new("ui"), FontWeight::Regular, false);
    let italic_key = manager.font_key(&FamilyKey::new("ui"), FontWeight::Regular, true);
    assert!(snapshot.definitions.font_data.contains_key(&normal_key));
    assert!(snapshot.definitions.font_data.contains_key(&italic_key));
    assert_eq!(
        snapshot.definitions.families[&egui::FontFamily::Name(italic_key.clone().into())][0],
        italic_key
    );
    assert!(manager.remove_face("ui", FontWeight::Regular, true));
    assert_eq!(
        manager.state.read().families[&FamilyKey::new("ui")].len(),
        1
    );
}

#[test]
fn preserves_base_families_and_shares_unchanged_font_data_between_revisions() {
    let mut base = egui::FontDefinitions::default();
    base.families.insert(
        egui::FontFamily::Name("app-custom".into()),
        vec!["Hack".into()],
    );
    let manager = FontManager::with_base_fonts(base.clone());
    manager.load_from_bytes("main", &bytes("Hack"), 0).unwrap();
    let before = manager.snapshot();
    let key = manager.font_key(&FamilyKey::new("main"), FontWeight::Regular, false);
    manager.set_fallback_chain(&["future-font"]);
    let after = manager.snapshot();
    assert!(Arc::ptr_eq(
        &before.definitions.font_data[&key],
        &after.definitions.font_data[&key]
    ));
    for (family, fonts) in &base.families {
        assert_eq!(&after.definitions.families[family], fonts);
    }
    let ctx = egui::Context::default();
    manager.apply(&ctx);
    manager.set_base_fonts(base.clone());
    assert!(!manager.apply(&ctx));
    base.families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "Hack".into());
    manager.set_base_fonts(base);
    assert!(manager.apply(&ctx));
}
