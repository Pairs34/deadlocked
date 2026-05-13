use std::{num::NonZeroU32, sync::Arc};

use egui::{Color32, FontData, FontDefinitions, Stroke, Style};
use egui_glow::glow::{self, HasContext as _};
use glutin::prelude::PossiblyCurrentGlContext;
use winit::platform::x11::{WindowAttributesExtX11, WindowType};

use crate::ui::color::Colors;

pub struct WindowContext {
    window: winit::window::Window,
    gl_context: glutin::context::PossiblyCurrentContext,
    _gl_display: glutin::display::Display,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    glow: Arc<glow::Context>,
    egui_glow: egui_glow::EguiGlow,
    clear_color: Color32,
}

impl WindowContext {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        overlay: bool,
        accent_color: egui::Color32,
    ) -> Self {
        use glutin::context::NotCurrentGlContext as _;
        use glutin::display::GetGlDisplay as _;
        use glutin::display::GlDisplay as _;
        use glutin::prelude::GlSurface as _;
        use winit::raw_window_handle::HasWindowHandle as _;

        let winit_window_builder = if overlay {
            winit::window::WindowAttributes::default()
                .with_decorations(false)
                .with_inner_size(winit::dpi::PhysicalSize::new(1, 1))
                .with_position(winit::dpi::PhysicalPosition::new(0, 0))
                .with_resizable(false)
                .with_transparent(true)
                .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
                .with_override_redirect(true)
                .with_x11_window_type(vec![WindowType::Tooltip])
                .with_title("deadlocked_overlay")
        } else {
            winit::window::WindowAttributes::default()
                .with_inner_size(winit::dpi::LogicalSize::new(940, 580))
                .with_min_inner_size(winit::dpi::LogicalSize::new(800u32, 520u32))
                .with_title("Pairs Cheat")
        };

        let config_template_builder = if overlay {
            glutin::config::ConfigTemplateBuilder::new()
                .prefer_hardware_accelerated(Some(true))
                .with_transparency(true)
        } else {
            glutin::config::ConfigTemplateBuilder::new()
                .prefer_hardware_accelerated(Some(true))
                .with_transparency(false)
        };

        let (mut window, gl_config) =
            glutin_winit::DisplayBuilder::new() // let glutin-winit helper crate handle the complex parts of opengl context creation
                .with_preference(glutin_winit::ApiPreference::FallbackEgl) // https://github.com/emilk/egui/issues/2520#issuecomment-1367841150
                .with_window_attributes(Some(winit_window_builder.clone()))
                .build(
                    event_loop,
                    config_template_builder,
                    |mut config_iterator| {
                        config_iterator.next().expect(
                            "failed to find a matching configuration for creating glutin config",
                        )
                    },
                )
                .expect("failed to create gl_config");
        let gl_display = gl_config.display();

        let raw_window_handle = window.as_ref().map(|w| {
            w.window_handle()
                .expect("failed to get window handle")
                .as_raw()
        });
        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(raw_window_handle);
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(raw_window_handle);
        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_config
                        .display()
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context even with fallback attributes")
                })
        };

        // this is where the window is created, if it has not been created while searching for suitable gl_config
        let window = window.take().unwrap_or_else(|| {
            glutin_winit::finalize_window(event_loop, winit_window_builder.clone(), &gl_config)
                .expect("failed to finalize glutin window")
        });
        let (width, height): (u32, u32) = window.inner_size().into();
        let width = NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN);
        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window
                        .window_handle()
                        .expect("failed to get window handle")
                        .as_raw(),
                    width,
                    height,
                );
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .unwrap()
        };
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        gl_surface
            .set_swap_interval(&gl_context, glutin::surface::SwapInterval::DontWait)
            .unwrap();

        if overlay {
            window.set_cursor_hittest(false).unwrap();
            window.set_outer_position(winit::dpi::PhysicalPosition::new(0, 0));
        }

        let glow = unsafe {
            glow::Context::from_loader_function(|s| {
                let s = std::ffi::CString::new(s)
                    .expect("failed to construct C string from string for gl proc address");

                gl_display.get_proc_address(&s)
            })
        };

        let glow = Arc::new(glow);
        let mut egui_glow = egui_glow::EguiGlow::new(event_loop, glow.clone(), None, None, true);
        prep_ctx(&mut egui_glow.egui_ctx, accent_color);

        let clear_color = if overlay {
            Color32::TRANSPARENT
        } else {
            Color32::BLACK
        };

        Self {
            window,
            gl_context,
            _gl_display: gl_display,
            gl_surface,
            glow,
            egui_glow,
            clear_color,
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        use glutin::surface::GlSurface as _;
        let width = NonZeroU32::new(physical_size.width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(physical_size.height).unwrap_or(NonZeroU32::MIN);
        self.gl_surface.resize(&self.gl_context, width, height);
    }

    pub fn swap_buffers(&self) -> glutin::error::Result<()> {
        use glutin::surface::GlSurface as _;
        self.gl_surface.swap_buffers(&self.gl_context)
    }

    pub fn make_current(&self) -> glutin::error::Result<()> {
        self.gl_context.make_current(&self.gl_surface)
    }

    pub fn process_event(&mut self, event: &winit::event::WindowEvent) -> egui_glow::EventResponse {
        self.egui_glow.on_window_event(&self.window, event)
    }

    pub fn process_modifier(&mut self, modifiers: egui::Modifiers, pressed: bool, repeat: bool) {
        self.egui_glow.egui_ctx.input_mut(|i| {
            i.events.push(egui::Event::Key {
                key: egui::Key::F35,
                physical_key: None,
                pressed,
                repeat,
                modifiers,
            });
        });
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn run(&mut self, func: impl FnMut(&mut egui::Ui)) {
        self.egui_glow.run(&self.window, func);
    }

    pub fn clear(&self) {
        let [r, g, b, a] = self.clear_color.to_normalized_gamma_f32();
        unsafe {
            self.glow.clear_color(r, g, b, a);
            self.glow.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    pub fn paint(&mut self) {
        self.egui_glow.paint(&self.window);
    }
}

impl Drop for WindowContext {
    fn drop(&mut self) {
        self.egui_glow.destroy();
    }
}

fn prep_ctx(ctx: &mut egui::Context, accent_color: egui::Color32) {
    // add font
    let fira_sans = include_bytes!("../../resources/FiraSansIcons.ttf");
    let cs2_icons = include_bytes!("../../resources/CS2EquipmentIcons.ttf");
    let mut font_definitions = FontDefinitions::default();
    font_definitions.font_data.insert(
        String::from("fira_sans"),
        Arc::new(FontData::from_static(fira_sans)),
    );
    font_definitions.font_data.insert(
        String::from("cs2_icons"),
        Arc::new(FontData::from_static(cs2_icons)),
    );

    // insert into font definitions, so it gets chosen as default
    font_definitions
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, String::from("fira_sans"));
    font_definitions
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, String::from("cs2_icons"));

    ctx.set_fonts(font_definitions);

    ctx.style_mut_of(egui::Theme::Dark, |style| {
        gui_style(style, accent_color);
    });
}

fn gui_style(style: &mut Style, accent_color: egui::Color32) {
    style.interaction.selectable_labels = false;

    // --- Typography ---
    for font in style.text_styles.iter_mut() {
        font.1.size = 14.0;
    }
    // Bigger heading
    if let Some(h) = style.text_styles.get_mut(&egui::TextStyle::Heading) {
        h.size = 18.0;
    }

    // --- Panel / window backgrounds ---
    style.visuals.window_fill = Colors::BASE;
    style.visuals.panel_fill = Colors::BASE;
    style.visuals.extreme_bg_color = Colors::BACKDROP;
    style.visuals.window_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 16));
    style.visuals.window_shadow = egui::Shadow {
        offset: [0, 6].into(),
        blur: 22,
        spread: 0,
        color: Color32::from_black_alpha(140),
    };

    // --- Corner radii (more modern, slightly larger) ---
    let r6 = egui::CornerRadius::same(6);
    let r8 = egui::CornerRadius::same(8);
    style.visuals.window_corner_radius = r8;
    style.visuals.menu_corner_radius = r8;

    // --- Selection ---
    style.visuals.selection.bg_fill =
        Color32::from_rgba_unmultiplied(accent_color.r(), accent_color.g(), accent_color.b(), 210);
    style.visuals.selection.stroke = Stroke::new(1.0, accent_color);

    // --- Hyperlink ---
    style.visuals.hyperlink_color = accent_color;

    // Subtle separator line
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 18));

    // --- Widget: inactive (default resting state) ---
    style.visuals.widgets.inactive.bg_fill      = Colors::SURFACE;
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(30, 32, 42);
    style.visuals.widgets.inactive.bg_stroke    = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 24));
    style.visuals.widgets.inactive.fg_stroke    = Stroke::new(1.5, Colors::TEXT);
    style.visuals.widgets.inactive.corner_radius = r6;

    // --- Widget: hovered ---
    style.visuals.widgets.hovered.bg_fill      = Colors::HIGHLIGHT;
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(46, 49, 68);
    style.visuals.widgets.hovered.bg_stroke    = Stroke::new(1.0, Color32::from_rgba_unmultiplied(
        accent_color.r(), accent_color.g(), accent_color.b(), 90,
    ));
    style.visuals.widgets.hovered.fg_stroke    = Stroke::new(1.5, Colors::TEXT);
    style.visuals.widgets.hovered.corner_radius = r6;
    style.visuals.widgets.hovered.expansion    = 1.0;

    // --- Widget: active (pressed / selected) ---
    style.visuals.widgets.active.bg_fill      = Color32::from_rgba_unmultiplied(
        accent_color.r(), accent_color.g(), accent_color.b(), 210,
    );
    style.visuals.widgets.active.weak_bg_fill = Color32::from_rgba_unmultiplied(
        accent_color.r(), accent_color.g(), accent_color.b(), 170,
    );
    style.visuals.widgets.active.bg_stroke    = Stroke::new(1.0, accent_color);
    style.visuals.widgets.active.fg_stroke    = Stroke::new(1.5, Colors::TEXT);
    style.visuals.widgets.active.corner_radius = r6;

    // --- Widget: open (e.g. open combo-box) ---
    style.visuals.widgets.open.bg_fill      = Color32::from_rgb(44, 47, 64);
    style.visuals.widgets.open.weak_bg_fill = Color32::from_rgb(38, 41, 56);
    style.visuals.widgets.open.bg_stroke    = Stroke::new(1.0, Color32::from_rgba_unmultiplied(
        accent_color.r(), accent_color.g(), accent_color.b(), 110,
    ));
    style.visuals.widgets.open.fg_stroke    = Stroke::new(1.5, Colors::TEXT);
    style.visuals.widgets.open.corner_radius = r6;

    // --- Widget: noninteractive (labels, panels) ---
    style.visuals.widgets.noninteractive.bg_fill      = Colors::BASE;
    style.visuals.widgets.noninteractive.weak_bg_fill = Colors::BACKDROP;
    style.visuals.widgets.noninteractive.fg_stroke    = Stroke::new(1.0, Colors::SUBTEXT);
    style.visuals.widgets.noninteractive.corner_radius = r6;

    // --- Spacing ---
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.spacing.button_padding = egui::vec2(11.0, 6.0);
    style.spacing.indent = 14.0;
    style.spacing.slider_width = 160.0;
    style.spacing.combo_width = 140.0;
}
