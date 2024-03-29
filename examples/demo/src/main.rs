use microui_femtovg::{App, Shell, run, microui::{*, const_vec::ConstStr}};
use microui_theme::catppuccin;

#[derive(Debug)]
struct Demo {
    dropdown_state: dropdown::State,
    themes: dropdown::State,
    checkboxes: [bool; 3],
    background: Color,
    textbox_state: ConstStr<128>,
    log: ConstStr<32000>,
    log_updated: bool
}

fn main() {    
    run(Box::new(Demo {
        dropdown_state: dropdown::State::default(),
        themes: dropdown::State::with_selection(0),
        checkboxes: Default::default(),
        background: Color::rgb(90, 95, 100),
        textbox_state: ConstStr::new(), 
        log: ConstStr::new(),
        log_updated: false
    }));
}

impl App for Demo {
    fn frame(&mut self, ctx: &mut Context, shell: &mut Shell) {
        self.test_window(ctx, shell);
        self.log_window(ctx);
        self.style_window(ctx);
    }
}

impl Demo {
    fn test_window(&mut self, ctx: &mut Context, shell: &mut Shell) {
        Window::new("Demo Window", rect(40, 40, 335, 450))
            .min_size(vec2(335, 300))
            .show(ctx, |ctx| 
        {
            if ctx.header("Window Info", false) {
                let rect = ctx.current_container().rect;
    
                ctx.layout_row(&[66, -1], 0);
    
                ctx.label("Position:");
                ctx.label(format!("{}, {}", rect.x, rect.y));
    
                ctx.label("Size:");
                ctx.label(format!("{}, {}", rect.w, rect.h));
            }
    
            if ctx.header("Test Buttons", true) {
                ctx.layout_row(&[108, -100, -1], 0);
    
                ctx.label("Test buttons:");
                if ctx.button("Button 1") {
                    self.write_log("Pressed button 1");
                }
    
                if ctx.button("Button 2") {
                    self.write_log("Pressed button 2");
                }
    
                ctx.label("Popup widgets:");
                
                const CHOICES: &[&str] = &["Option 1", "Option 2", "Option 3", "Option 4", "Option 5", "Option 6"];

                if ctx.w(
                    Dropdown::new(&mut self.dropdown_state, CHOICES)
                        .placeholder_text("Select option", true)
                ).submit {
                    let selected = self.dropdown_state.index.unwrap();
                    self.write_log(format!("Selected {}", CHOICES[selected]));
                }
    
                let popup = Popup::new("Test Popup");
                
                if ctx.button("Popup") {
                    popup.open(ctx);
                }
    
                popup.show(ctx, |ctx| {
                    if ctx.button("Hello") {
                        self.write_log("Hello");
                    }

                    if ctx.button("World") {
                        self.write_log("World");
                    }
                });
            }
    
            if ctx.header("Tree and Text", true) {
                ctx.layout_row(&[140, -1], 0);
                ctx.layout_begin_column();
    
                Treenode::new("Test 1").show(ctx, |ctx| {
                    Treenode::new("Test 1a").show(ctx, |ctx| {
                        if ctx.clickable_label("Click me!") {
                            self.write_log("Clicked on label 1");
                        }

                        if ctx.clickable_label("Click me 2!") {
                            self.write_log("Clicked on label 2");
                        }
                    });

                    Treenode::new("Test 1b").show(ctx, |ctx| {
                        if ctx.button("Button 1") {
                            self.write_log("Pressed button 1");
                        }
            
                        if ctx.button("Button 2") {
                            self.write_log("Pressed button 2");
                        }
                    });
                });
    
                Treenode::new("Test 2").show(ctx, |ctx| {
                    ctx.layout_row(&[58, 54], 0);
    
                    if ctx.button("Button 3") {
                        self.write_log("Pressed button 3");
                    }
        
                    if ctx.button("Button 4") {
                        self.write_log("Pressed button 4");
                    }
    
                    if ctx.button("Button 5") {
                        self.write_log("Pressed button 5");
                    }
        
                    if ctx.button("Button 6") {
                        self.write_log("Pressed button 6");
                    }
                });

                Treenode::new("Test 3").show(ctx, |ctx| {
                    ctx.checkbox("Checkbox 1", &mut self.checkboxes[0]);
                    ctx.checkbox("Checkbox 2", &mut self.checkboxes[1]);
                    ctx.checkbox("Checkbox 3", &mut self.checkboxes[2]);
                });
    
                ctx.layout_end_column();

                ctx.layout_begin_column();
                ctx.layout_row(&[-1], 0);
                ctx.text("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas lacinia, sem eu lacinia molestie, mi risus faucibus ipsum, eu varius magna felis a nulla.");

                ctx.layout_end_column();
            }

            if ctx.header("Background Color", true) {
                ctx.layout_row(&[-78, -1], 74);

                ctx.layout_begin_column();
                ctx.layout_row(&[46, -1], 0);

                let mut value_changed = false;
                let mut r = self.background.r as f64;
                let mut g = self.background.g as f64;
                let mut b = self.background.b as f64;
                let mut a = self.background.a as f64;

                ctx.label("Red:");
                if ctx.slider(&mut r, 0.0..255.) {
                    value_changed = true;
                }

                ctx.label("Green:");
                if ctx.slider(&mut g, 0.0..255.) {
                    value_changed = true;
                }

                ctx.label("Blue:");
                if ctx.slider(&mut b, 0.0..255.) {
                    value_changed = true;
                }

                ctx.label("Alpha:");
                if ctx.drag_value(&mut a, 1.) {
                    value_changed = true
                }

                if value_changed {
                    self.background = Color::rgba(r as u8, g as u8, b as u8, a as u8);
                    shell.set_clear_color(self.background);
                }

                ctx.layout_end_column();

                let rect = ctx.layout_next();

                ctx.draw_rect(rect, self.background);

                let color_label = format!(
                    "#{:#04x}{:#04x}{:#04x}",
                    self.background.r,
                    self.background.g,
                    self.background.b
                );

                let color_label = color_label.replace("0x", "");

                let mut opts = ContainerOptions::default();
                opts.set(ContainerOption::AlignCenter);

                ctx.draw_widget_text(color_label, rect, WidgetColor::Text, opts);
            }
        });
    }

    fn log_window(&mut self, ctx: &mut Context) {
        let rect = rect(380, 40, 390, 200);

        Window::new("Log Window", rect).show(ctx, |ctx| {
            ctx.layout_row(&[-1], -25);

            let mut index = 0;
            Panel::new("Log Output").show(ctx, |ctx| {
                index = ctx.current_container_index().unwrap();
                ctx.layout_row(&[-1], -1);
                
                ctx.text(self.log.as_str());
            });

            if self.log_updated {
                // Scroll to bottom
                let panel = ctx.container_mut(index);
                panel.scroll.y = panel.content_size.y;

                self.log_updated = false;
            }

            let mut textbox_submitted = false;

            ctx.layout_row(&[-70, -1], 0);

            if ctx.textbox(&mut self.textbox_state).submit {
                ctx.set_focus(ctx.last_id());
                textbox_submitted = true;
            }

            if ctx.button("Submit") {
                textbox_submitted = true;
            }

            if textbox_submitted {
                let text = self.textbox_state.as_str().to_string();
                self.write_log(text);

                self.textbox_state.clear();
            }
        });
    }

    fn style_window(&mut self, ctx: &mut Context) {
        const LABELS: &[&'static str] = &[
            "text:",
            "border:",
            "window bg:",
            "title bg:",
            "title text:",
            "panel bg:",
            "button:",
            "button hover:",
            "button focus:",
            "base:",
            "base hover:",
            "base focus:",
            "scroll base:",
            "scroll thumb:"
        ];

        let rect = rect(380, 250, 390, 240);

        Window::new("Style Editor", rect)
            .min_size(vec2(390, 240))
            .show(ctx, |ctx|
        {
            ctx.layout_row(&[55, -1], 0);
            ctx.label("Theme:");

            const THEMES: &[&str] = &["Default", "Catppuccin Latte", "Catppuccin Frappe", "Catppuccin Macchiato", "Catppuccin Mocha"];

            if ctx.w(Dropdown::new(&mut self.themes, THEMES).visible_items(5)).submit {
                let colors = match self.themes.index.unwrap() {
                    1 => catppuccin::LATTE.widget_colors(),
                    2 => catppuccin::FRAPPE.widget_colors(),
                    3 => catppuccin::MACCHIATO.widget_colors(),
                    4 => catppuccin::MOCHA.widget_colors(),
                    _ => WidgetColors::default()
                };

                ctx.style.colors = colors;
            }

            ctx.layout_row(&[-1], -1);

            Panel::new("Theme color editor").show(ctx, |ctx| {
                let width = ctx.current_container().body.w as f64 * 0.14;
                let width = width as i32;
    
                ctx.layout_row(&[96, width, width, width, width, -1], 0);
    
                for i in 0..ctx.style.colors.0.len() {
                    let mut color = ctx.style.colors.0[i];
    
                    ctx.label(LABELS[i]);
                    self.style_slider(ctx, &mut color.r, i);
                    self.style_slider(ctx, &mut color.g, i);
                    self.style_slider(ctx, &mut color.b, i);
                    self.style_slider(ctx, &mut color.a, i);
                    ctx.style.colors.0[i] = color;
                    
                    let next = ctx.layout_next();
                    ctx.draw_rect(next, color);
                }
            });
        });
    }

    #[inline]
    fn style_slider(
        &self,
        ctx: &mut Context,
        value: &mut u8,
        entropy: usize
    ) {
        let mut float = *value as f64;
        let addr = value as *mut u8;

        ctx.push_id(&[addr as usize, entropy]);
        ctx.slider(&mut float, 0f64..255.);
        ctx.pop_id();

        *value = float as u8;
    }

    #[inline]
    fn write_log(&mut self, text: impl Into<String>) {
        let mut text = text.into();

        if text.is_empty() {
            return;
        }

        text.push('\n');

        self.log.push_str(&text);
        self.log_updated = true;
    }
}
