use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ThemeVisuals {
    pub name: String,
    pub dark_mode: bool,

    // Основные цвета
    pub override_text_color: Option<[u8; 3]>, // RGB
    pub hyperlink_color: [u8; 3],             // RGB
    pub faint_bg_color: [u8; 3],              // RGB
    pub extreme_bg_color: [u8; 3],            // RGB
    pub code_bg_color: [u8; 3],               // RGB
    pub warn_fg_color: [u8; 3],               // RGB
    pub error_fg_color: [u8; 3],              // RGB
    pub window_fill: [u8; 3],                 // RGB
    pub window_stroke_color: [u8; 3],         // RGB
    pub window_stroke_width: f32,
    pub panel_fill: [u8; 3], // RGB

    // Простые значения
    pub resize_corner_size: f32,
    pub button_frame: bool,
    pub collapsing_header_frame: bool,
    pub striped: bool,
    pub disabled_alpha: f32,
}

impl ThemeVisuals {
    pub fn to_egui_visuals(&self) -> eframe::egui::Visuals {
        use eframe::egui::Stroke;
        use eframe::egui::Visuals;

        let mut visuals = if self.dark_mode {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        visuals.dark_mode = self.dark_mode;
        visuals.override_text_color = self
            .override_text_color
            .map(|rgb| Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        visuals.hyperlink_color = Color32::from_rgb(
            self.hyperlink_color[0],
            self.hyperlink_color[1],
            self.hyperlink_color[2],
        );
        visuals.faint_bg_color = Color32::from_rgb(
            self.faint_bg_color[0],
            self.faint_bg_color[1],
            self.faint_bg_color[2],
        );
        visuals.extreme_bg_color = Color32::from_rgb(
            self.extreme_bg_color[0],
            self.extreme_bg_color[1],
            self.extreme_bg_color[2],
        );
        visuals.code_bg_color = Color32::from_rgb(
            self.code_bg_color[0],
            self.code_bg_color[1],
            self.code_bg_color[2],
        );
        visuals.warn_fg_color = Color32::from_rgb(
            self.warn_fg_color[0],
            self.warn_fg_color[1],
            self.warn_fg_color[2],
        );
        visuals.error_fg_color = Color32::from_rgb(
            self.error_fg_color[0],
            self.error_fg_color[1],
            self.error_fg_color[2],
        );
        visuals.window_fill = Color32::from_rgb(
            self.window_fill[0],
            self.window_fill[1],
            self.window_fill[2],
        );
        visuals.window_stroke = Stroke::new(
            self.window_stroke_width,
            Color32::from_rgb(
                self.window_stroke_color[0],
                self.window_stroke_color[1],
                self.window_stroke_color[2],
            ),
        );
        visuals.panel_fill =
            Color32::from_rgb(self.panel_fill[0], self.panel_fill[1], self.panel_fill[2]);
        visuals.resize_corner_size = self.resize_corner_size;
        visuals.button_frame = self.button_frame;
        visuals.collapsing_header_frame = self.collapsing_header_frame;
        visuals.striped = self.striped;
        visuals.disabled_alpha = self.disabled_alpha;

        visuals
    }

    // Создаем тему по умолчанию - темная
    pub fn default_dark() -> Self {
        Self {
            name: "Dark".to_string(),
            dark_mode: true,
            override_text_color: None,
            hyperlink_color: [96, 176, 255], // Синий
            faint_bg_color: [25, 25, 25],
            extreme_bg_color: [10, 10, 10],
            code_bg_color: [30, 30, 30],
            warn_fg_color: [255, 180, 0],  // Оранжевый
            error_fg_color: [255, 50, 50], // Красный
            window_fill: [30, 30, 30],
            window_stroke_color: [60, 60, 60],
            window_stroke_width: 1.0,
            panel_fill: [20, 20, 20],
            resize_corner_size: 12.0,
            button_frame: true,
            collapsing_header_frame: false,
            striped: true,
            disabled_alpha: 0.55,
        }
    }

    // Создаем тему по умолчанию - светлая
    pub fn default_light() -> Self {
        Self {
            name: "Light".to_string(),
            dark_mode: false,
            override_text_color: None,
            hyperlink_color: [0, 100, 200], // Синий
            faint_bg_color: [240, 240, 240],
            extreme_bg_color: [255, 255, 255],
            code_bg_color: [230, 230, 230],
            warn_fg_color: [200, 120, 0], // Оранжевый
            error_fg_color: [200, 0, 0],  // Красный
            window_fill: [240, 240, 240],
            window_stroke_color: [180, 180, 180],
            window_stroke_width: 1.0,
            panel_fill: [230, 230, 230],
            resize_corner_size: 12.0,
            button_frame: true,
            collapsing_header_frame: false,
            striped: true,
            disabled_alpha: 0.55,
        }
    }
}
