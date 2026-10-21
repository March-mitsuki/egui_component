use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{select, text},
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let title_style = text::Style::new_heading(theme).size(16.0);

    ui.vertical(|ui| {
        // 1. Interactive Demo
        text::render(
            ui,
            locale.text("demo.select:interactive"),
            title_style.clone(),
        );
        ui.add_space(8.0);

        let select_id = ui.make_persistent_id("interactive_select");
        let mut selected = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(select_id)
                .unwrap_or_else(|| "Option 1".to_string())
        });

        let current_selected = selected.clone();
        let default_style = select::Style::new(theme, None, select::SelectSize::Md);

        ui.horizontal(|ui| {
            text::render(
                ui,
                locale.text("demo.select:choose"),
                text::Style::new(theme),
            );
            select::render(
                ui,
                select_id,
                option_label(locale, &current_selected),
                default_style.clone().width(Some(200.0)),
                |ui| {
                    let opts = vec!["Option 1", "Option 2", "Option 3", "Option 4", "Option 5"];
                    for opt in opts {
                        if select::render_item(
                            ui,
                            option_label(locale, opt),
                            selected == opt,
                            &default_style,
                            theme,
                        )
                        .clicked()
                        {
                            selected = opt.to_string();
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(select_id, selected.clone());
                            });
                            egui::Popup::close_id(ui.ctx(), select_id);
                        }
                    }
                },
            );
            ui.add_space(12.0);
            text::render(
                ui,
                &locale.format(
                    "demo.select:selected",
                    &[("selected", option_label(locale, &selected))],
                ),
                text::Style::new(theme),
            );
        });

        ui.add_space(24.0);

        // 2. Sizes
        text::render(ui, locale.text("common:sizes"), title_style.clone());
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            // XS Select
            let xs_id = ui.make_persistent_id("xs_select");
            let mut xs_val =
                ui.memory(|mem| mem.data.get_temp::<String>(xs_id).unwrap_or_default());
            let current_xs = xs_val.clone();
            let xs_style = select::Style::new(theme, None, select::SelectSize::Xs);
            select::render(
                ui,
                xs_id,
                option_label(locale, &current_xs),
                xs_style
                    .clone()
                    .placeholder(locale.text("demo.select:xs"))
                    .width(Some(120.0)),
                |ui| {
                    let opts = vec!["Apple", "Banana", "Orange"];
                    for opt in opts {
                        if select::render_item(
                            ui,
                            option_label(locale, opt),
                            xs_val == opt,
                            &xs_style,
                            theme,
                        )
                        .clicked()
                        {
                            xs_val = opt.to_string();
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(xs_id, xs_val.clone());
                            });
                            egui::Popup::close_id(ui.ctx(), xs_id);
                        }
                    }
                },
            );

            ui.add_space(12.0);

            // SM Select
            let sm_id = ui.make_persistent_id("sm_select");
            let mut sm_val =
                ui.memory(|mem| mem.data.get_temp::<String>(sm_id).unwrap_or_default());
            let current_sm = sm_val.clone();
            let sm_style = select::Style::new(theme, None, select::SelectSize::Sm);
            select::render(
                ui,
                sm_id,
                option_label(locale, &current_sm),
                sm_style
                    .clone()
                    .placeholder(locale.text("demo.select:sm"))
                    .width(Some(140.0)),
                |ui| {
                    let opts = vec!["Rust", "Python", "TypeScript"];
                    for opt in opts {
                        if select::render_item(
                            ui,
                            option_label(locale, opt),
                            sm_val == opt,
                            &sm_style,
                            theme,
                        )
                        .clicked()
                        {
                            sm_val = opt.to_string();
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(sm_id, sm_val.clone());
                            });
                            egui::Popup::close_id(ui.ctx(), sm_id);
                        }
                    }
                },
            );

            ui.add_space(12.0);

            // MD Select
            let md_id = ui.make_persistent_id("md_select");
            let mut md_val =
                ui.memory(|mem| mem.data.get_temp::<String>(md_id).unwrap_or_default());
            let current_md = md_val.clone();
            let md_style = select::Style::new(theme, None, select::SelectSize::Md);
            select::render(
                ui,
                md_id,
                option_label(locale, &current_md),
                md_style
                    .clone()
                    .placeholder(locale.text("demo.select:md"))
                    .width(Some(160.0)),
                |ui| {
                    let opts = vec!["VS Code", "Neovim", "Emacs"];
                    for opt in opts {
                        if select::render_item(
                            ui,
                            option_label(locale, opt),
                            md_val == opt,
                            &md_style,
                            theme,
                        )
                        .clicked()
                        {
                            md_val = opt.to_string();
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(md_id, md_val.clone());
                            });
                            egui::Popup::close_id(ui.ctx(), md_id);
                        }
                    }
                },
            );

            ui.add_space(12.0);

            // LG Select
            let lg_id = ui.make_persistent_id("lg_select");
            let mut lg_val =
                ui.memory(|mem| mem.data.get_temp::<String>(lg_id).unwrap_or_default());
            let current_lg = lg_val.clone();
            let lg_style = select::Style::new(theme, None, select::SelectSize::Lg);
            select::render(
                ui,
                lg_id,
                option_label(locale, &current_lg),
                lg_style
                    .clone()
                    .placeholder(locale.text("demo.select:lg"))
                    .width(Some(180.0)),
                |ui| {
                    let opts = vec!["Linux", "macOS", "Windows"];
                    for opt in opts {
                        if select::render_item(
                            ui,
                            option_label(locale, opt),
                            lg_val == opt,
                            &lg_style,
                            theme,
                        )
                        .clicked()
                        {
                            lg_val = opt.to_string();
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(lg_id, lg_val.clone());
                            });
                            egui::Popup::close_id(ui.ctx(), lg_id);
                        }
                    }
                },
            );
        });

        ui.add_space(24.0);

        // 3. Colors
        text::render(ui, locale.text("common:colors"), title_style.clone());
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            // Green
            let green_id = ui.make_persistent_id("green_select");
            let mut green_val =
                ui.memory(|mem| mem.data.get_temp::<String>(green_id).unwrap_or_default());
            let current_green = green_val.clone();
            let green_style =
                select::Style::new(theme, Some(theme.colors.green), select::SelectSize::Md);
            select::render(
                ui,
                green_id,
                option_label(locale, &current_green),
                green_style
                    .clone()
                    .placeholder(locale.text("demo.select:green"))
                    .width(Some(150.0)),
                |ui| {
                    let opts = vec!["Low", "Medium", "High"];
                    for opt in opts {
                        if select::render_item(
                            ui,
                            option_label(locale, opt),
                            green_val == opt,
                            &green_style,
                            theme,
                        )
                        .clicked()
                        {
                            green_val = opt.to_string();
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(green_id, green_val.clone());
                            });
                            egui::Popup::close_id(ui.ctx(), green_id);
                        }
                    }
                },
            );

            ui.add_space(12.0);

            // Blue
            let blue_id = ui.make_persistent_id("blue_select");
            let mut blue_val =
                ui.memory(|mem| mem.data.get_temp::<String>(blue_id).unwrap_or_default());
            let current_blue = blue_val.clone();
            let blue_style =
                select::Style::new(theme, Some(theme.colors.blue), select::SelectSize::Md);
            select::render(
                ui,
                blue_id,
                option_label(locale, &current_blue),
                blue_style
                    .clone()
                    .placeholder(locale.text("demo.select:blue"))
                    .width(Some(150.0)),
                |ui| {
                    let opts = vec!["Primary", "Secondary", "Success"];
                    for opt in opts {
                        if select::render_item(
                            ui,
                            option_label(locale, opt),
                            blue_val == opt,
                            &blue_style,
                            theme,
                        )
                        .clicked()
                        {
                            blue_val = opt.to_string();
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(blue_id, blue_val.clone());
                            });
                            egui::Popup::close_id(ui.ctx(), blue_id);
                        }
                    }
                },
            );

            ui.add_space(12.0);

            // Red
            let red_id = ui.make_persistent_id("red_select");
            let mut red_val =
                ui.memory(|mem| mem.data.get_temp::<String>(red_id).unwrap_or_default());
            let current_red = red_val.clone();
            let red_style =
                select::Style::new(theme, Some(theme.colors.red), select::SelectSize::Md);
            select::render(
                ui,
                red_id,
                option_label(locale, &current_red),
                red_style
                    .clone()
                    .placeholder(locale.text("demo.select:red"))
                    .width(Some(150.0)),
                |ui| {
                    let opts = vec!["Fatal", "Error", "Warning"];
                    for opt in opts {
                        if select::render_item(
                            ui,
                            option_label(locale, opt),
                            red_val == opt,
                            &red_style,
                            theme,
                        )
                        .clicked()
                        {
                            red_val = opt.to_string();
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(red_id, red_val.clone());
                            });
                            egui::Popup::close_id(ui.ctx(), red_id);
                        }
                    }
                },
            );
        });

        ui.add_space(24.0);

        // 4. Disabled
        text::render(
            ui,
            locale.text("common:disabledHeading"),
            title_style.clone(),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            // Disabled with placeholder
            let d_id1 = ui.make_persistent_id("disabled_select1");
            let md_style = select::Style::new(theme, None, select::SelectSize::Md);
            select::render(
                ui,
                d_id1,
                "",
                md_style
                    .clone()
                    .placeholder(locale.text("demo.select:disabled"))
                    .disabled(true)
                    .width(Some(160.0)),
                |_| {},
            );

            ui.add_space(12.0);

            // Disabled with value
            let d_id2 = ui.make_persistent_id("disabled_select2");
            select::render(
                ui,
                d_id2,
                locale.text("demo.select:preselected"),
                md_style.clone().disabled(true).width(Some(160.0)),
                |_| {},
            );
        });

        ui.add_space(24.0);

        // 5. Invalid
        text::render(ui, locale.text("demo.select:invalidHeading"), title_style);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            // Invalid empty (placeholder)
            let inv_id1 = ui.make_persistent_id("invalid_select1");
            let inv_style = select::Style::new(theme, None, select::SelectSize::Md);
            select::render(
                ui,
                inv_id1,
                "",
                inv_style
                    .clone()
                    .placeholder(locale.text("demo.select:required"))
                    .is_invalid(true)
                    .width(Some(160.0)),
                |_| {},
            );

            ui.add_space(12.0);

            // Invalid with selected value
            let inv_id2 = ui.make_persistent_id("invalid_select2");
            let mut inv_val2 = ui.memory(|mem| {
                mem.data
                    .get_temp::<String>(inv_id2)
                    .unwrap_or_else(|| "Invalid Input".to_string())
            });
            let current_inv2 = inv_val2.clone();
            select::render(
                ui,
                inv_id2,
                option_label(locale, &current_inv2),
                inv_style.clone().is_invalid(true).width(Some(160.0)),
                |ui| {
                    let opts = vec!["Valid Option 1", "Valid Option 2", "Invalid Input"];
                    for opt in opts {
                        if select::render_item(
                            ui,
                            option_label(locale, opt),
                            inv_val2 == opt,
                            &inv_style,
                            theme,
                        )
                        .clicked()
                        {
                            inv_val2 = opt.to_string();
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(inv_id2, inv_val2.clone());
                            });
                            egui::Popup::close_id(ui.ctx(), inv_id2);
                        }
                    }
                },
            );
        });
    });
}

fn option_label(locale: Locale, value: &str) -> &str {
    match value {
        "Option 1" => locale.text("demo.select:option1"),
        "Option 2" => locale.text("demo.select:option2"),
        "Option 3" => locale.text("demo.select:option3"),
        "Option 4" => locale.text("demo.select:option4"),
        "Option 5" => locale.text("demo.select:option5"),
        "Apple" => locale.text("demo.select:apple"),
        "Banana" => locale.text("demo.select:banana"),
        "Orange" => locale.text("demo.select:orange"),
        "Low" => locale.text("demo.select:low"),
        "Medium" => locale.text("demo.select:medium"),
        "High" => locale.text("demo.select:high"),
        "Primary" => locale.text("common:primary"),
        "Secondary" => locale.text("demo.select:secondary"),
        "Success" => locale.text("common:success"),
        "Fatal" => locale.text("demo.select:fatal"),
        "Error" => locale.text("common:error"),
        "Warning" => locale.text("common:warning"),
        "Valid Option 1" => locale.text("demo.select:validOption1"),
        "Valid Option 2" => locale.text("demo.select:validOption2"),
        "Invalid Input" => locale.text("demo.select:invalidInput"),
        _ => value,
    }
}
