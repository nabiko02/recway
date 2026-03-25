use iced::border::Radius;
use iced::widget::{button, container};
use iced::{Background, Border, Color, Shadow, Vector};

// Modern glass-morphism inspired color palette
#[derive(Debug, Clone, Copy)]
pub struct ColorPalette {
    // Base colors - modern dark theme with glass effects
    pub background: Color,       // Main app background - deep dark
    pub surface: Color,          // Glass card backgrounds
    pub surface_hover: Color,    // Hover states with glow
    pub surface_elevated: Color, // Elevated surfaces (modals, dropdowns)

    // Text colors - high contrast for accessibility
    pub text: Color,           // Primary text - pure white
    pub text_secondary: Color, // Secondary text - soft gray

    // Modern accent colors
    pub primary: Color,       // Electric blue primary
    pub primary_hover: Color, // Brighter primary hover
    pub primary_light: Color, // Light primary variant
    pub danger: Color,        // Red for destructive actions

    // Glass effect colors
    pub glass_border: Color, // Subtle glass borders
    pub glass_shadow: Color, // Drop shadows
    pub glow_enabled: bool,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            background: Color::from_rgb(0.05, 0.05, 0.08),
            surface: Color::from_rgba(0.1, 0.1, 0.15, 0.8),
            surface_hover: Color::from_rgba(0.15, 0.15, 0.2, 0.9),
            surface_elevated: Color::from_rgba(0.12, 0.12, 0.18, 0.95),
            text: Color::from_rgb(0.98, 0.98, 1.0),
            text_secondary: Color::from_rgb(0.7, 0.72, 0.8),
            primary: Color::from_rgb(0.0, 0.48, 1.0),
            primary_hover: Color::from_rgb(0.2, 0.6, 1.0),
            primary_light: Color::from_rgba(0.0, 0.48, 1.0, 0.15),
            danger: Color::from_rgb(1.0, 0.27, 0.27),
            glass_border: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
            glass_shadow: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            glow_enabled: true,
        }
    }
}

impl ColorPalette {
    /// Returns a shadow using the given color when glow is enabled, or no shadow otherwise.
    pub fn shadow(&self, color: Color, offset_y: f32, blur: f32) -> Shadow {
        if self.glow_enabled {
            Shadow {
                color,
                offset: Vector::new(0.0, offset_y),
                blur_radius: blur,
            }
        } else {
            Shadow::default()
        }
    }
}

// Dynamic, responsive design constants
pub mod design {
    // Base reference resolution (1920x1080)
    pub const REFERENCE_WIDTH: f32 = 1920.0;
    pub const REFERENCE_HEIGHT: f32 = 1080.0;

    // Scaling limits
    pub const MIN_SCALE_FACTOR: f32 = 0.7;
    pub const MAX_SCALE_FACTOR: f32 = 1.5;

    // Window sizing (responsive)
    pub const BASE_WINDOW_WIDTH: f32 = 560.0;
    pub const MIN_WINDOW_WIDTH: f32 = 400.0;
    pub const MAX_WINDOW_WIDTH: f32 = 700.0;

    // Compact window sizing
    pub const COMPACT_WINDOW_WIDTH: f32 = 200.0;
    pub const COMPACT_WINDOW_HEIGHT: f32 = 60.0;
    pub const COMPACT_WINDOW_PADDING: f32 = 20.0;

    // Error popup sizing
    pub const ERROR_WINDOW_WIDTH: f32 = 420.0;
    pub const ERROR_WINDOW_HEIGHT: f32 = 100.0;

    // Spacing (responsive)
    pub const BASE_WINDOW_PADDING: u16 = 20;
    pub const BASE_CONTAINER_PADDING: u16 = 12;
    pub const BASE_SECTION_SPACING: u16 = 16;
    #[allow(dead_code)]
    pub const BASE_ELEMENT_SPACING: u16 = 8;

    // Modern border radius - more rounded for glass effect
    pub const BORDER_RADIUS_SMALL: f32 = 12.0; // Cards and buttons
    pub const BORDER_RADIUS_LARGE: f32 = 20.0; // Modals and dialogs
    #[allow(dead_code)]
    pub const BORDER_RADIUS_ROUND: f32 = 50.0; // Fully rounded elements

    // Button sizing (responsive)
    pub const BASE_BUTTON_HEIGHT: u16 = 56;
    pub const BASE_BUTTON_PADDING_V: u16 = 16;
    pub const BASE_BUTTON_PADDING_H: u16 = 20;
    pub const COMPACT_BUTTON_PADDING: u16 = 6;

    // Text sizes (responsive)
    pub const BASE_TITLE_SIZE: u16 = 24;
    pub const BASE_SUBTITLE_SIZE: u16 = 14;
    pub const BASE_LABEL_SIZE: u16 = 11;
    pub const BASE_BUTTON_TEXT_SIZE: u16 = 14;
    pub const BASE_INPUT_TEXT_SIZE: u16 = 16;
    #[allow(dead_code)]
    pub const BASE_COMPACT_TEXT_SIZE: u16 = 12;
    pub const BASE_TIMER_TEXT_SIZE: u16 = 26;
    pub const COMPACT_COUNTDOWN_SIZE: u16 = 28;

    // Container sizes (responsive)
    pub const BASE_SMALL_SPACE: f32 = 8.0;
    pub const BASE_TINY_SPACE: f32 = 4.0;

    // Helper functions for responsive sizing
    pub fn scale_factor(screen_width: f32, screen_height: f32) -> f32 {
        let width_scale = screen_width / REFERENCE_WIDTH;
        let height_scale = screen_height / REFERENCE_HEIGHT;

        // Use the minimum scale but ensure it's not too small for usability
        let base_scale = width_scale.min(height_scale);

        // For smaller screens, use a slightly higher minimum to ensure UI is usable
        let adjusted_min = if screen_width < 1600.0 || screen_height < 900.0 {
            0.85
        } else {
            MIN_SCALE_FACTOR
        };

        base_scale.clamp(adjusted_min, MAX_SCALE_FACTOR)
    }

    pub fn scaled_size(base_size: u16, scale: f32) -> u16 {
        (base_size as f32 * scale).round() as u16
    }

    pub fn scaled_f32(base_size: f32, scale: f32) -> f32 {
        base_size * scale
    }

    // Dynamic constants based on scale factor
    pub fn window_padding(scale: f32) -> u16 {
        scaled_size(BASE_WINDOW_PADDING, scale)
    }
    pub fn container_padding(scale: f32) -> u16 {
        scaled_size(BASE_CONTAINER_PADDING, scale)
    }
    pub fn section_spacing(scale: f32) -> u16 {
        scaled_size(BASE_SECTION_SPACING, scale)
    }
    pub fn button_height(scale: f32) -> u16 {
        scaled_size(BASE_BUTTON_HEIGHT, scale)
    }
    pub fn button_padding_v(scale: f32) -> u16 {
        scaled_size(BASE_BUTTON_PADDING_V, scale)
    }
    pub fn button_padding_h(scale: f32) -> u16 {
        scaled_size(BASE_BUTTON_PADDING_H, scale)
    }

    // Text sizes
    pub fn title_size(scale: f32) -> u16 {
        scaled_size(BASE_TITLE_SIZE, scale)
    }
    pub fn subtitle_size(scale: f32) -> u16 {
        scaled_size(BASE_SUBTITLE_SIZE, scale)
    }
    pub fn label_size(scale: f32) -> u16 {
        scaled_size(BASE_LABEL_SIZE, scale)
    }
    pub fn button_text_size(scale: f32) -> u16 {
        scaled_size(BASE_BUTTON_TEXT_SIZE, scale)
    }
    pub fn input_text_size(scale: f32) -> u16 {
        scaled_size(BASE_INPUT_TEXT_SIZE, scale)
    }
    #[allow(dead_code)]
    pub fn compact_text_size(scale: f32) -> u16 {
        scaled_size(BASE_COMPACT_TEXT_SIZE, scale)
    }
    pub fn timer_text_size(scale: f32) -> u16 {
        scaled_size(BASE_TIMER_TEXT_SIZE, scale)
    }
    pub fn compact_countdown_size(scale: f32) -> u16 {
        scaled_size(COMPACT_COUNTDOWN_SIZE, scale)
    }

    // Spacing
    pub fn small_space(scale: f32) -> f32 {
        scaled_f32(BASE_SMALL_SPACE, scale)
    }
    pub fn tiny_space(scale: f32) -> f32 {
        scaled_f32(BASE_TINY_SPACE, scale)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ThemeAccent {
    #[default]
    Blue,
    Red,
    Orange,
    Purple,
    Pink,
}

impl ThemeAccent {
    pub fn color(&self) -> Color {
        match self {
            Self::Blue => Color::from_rgb(0.0, 0.48, 1.0),
            Self::Red => Color::from_rgb(1.0, 0.23, 0.19),
            Self::Orange => Color::from_rgb(1.0, 0.58, 0.0),
            Self::Purple => Color::from_rgb(0.6, 0.2, 1.0),
            Self::Pink => Color::from_rgb(1.0, 0.18, 0.58),
        }
    }

    pub fn hover_color(&self) -> Color {
        match self {
            Self::Blue => Color::from_rgb(0.2, 0.6, 1.0),
            Self::Red => Color::from_rgb(1.0, 0.4, 0.35),
            Self::Orange => Color::from_rgb(1.0, 0.7, 0.2),
            Self::Purple => Color::from_rgb(0.72, 0.38, 1.0),
            Self::Pink => Color::from_rgb(1.0, 0.38, 0.7),
        }
    }
}

#[derive(Default)]
pub struct Theme {
    pub colors: ColorPalette,
    pub accent: ThemeAccent,
}

impl Theme {
    pub fn with_accent(accent: ThemeAccent, glow: bool) -> Self {
        let c = accent.color();
        let h = accent.hover_color();
        let colors = ColorPalette {
            primary: c,
            primary_hover: h,
            primary_light: Color::from_rgba(c.r, c.g, c.b, 0.15),
            glow_enabled: glow,
            ..ColorPalette::default()
        };
        Self { colors, accent }
    }
}

// Pill-shaped toggle switch background
pub struct GlowToggleStyle(pub ColorPalette, pub bool /* active */);

impl button::StyleSheet for GlowToggleStyle {
    type Style = iced::Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.1 {
                self.0.primary
            } else {
                Color::from_rgba(1.0, 1.0, 1.0, 0.08)
            })),
            border: Border {
                color: if self.1 {
                    Color::TRANSPARENT
                } else {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.15)
                },
                width: 1.0,
                radius: Radius::from(50.0),
            },
            shadow: if self.1 {
                self.0.shadow(
                    Color::from_rgba(self.0.primary.r, self.0.primary.g, self.0.primary.b, 0.4),
                    2.0,
                    8.0,
                )
            } else {
                Shadow::default()
            },
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
}

// White dot inside the toggle switch
pub struct ToggleDotStyle;

impl container::StyleSheet for ToggleDotStyle {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::WHITE)),
            border: Border {
                radius: Radius::from(50.0),
                ..Border::default()
            },
            ..container::Appearance::default()
        }
    }
}

// White rounded square for stop button icon
pub struct StopIconStyle;

impl container::StyleSheet for StopIconStyle {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::WHITE)),
            border: Border {
                radius: Radius::from(2.0),
                ..Border::default()
            },
            ..container::Appearance::default()
        }
    }
}

// Small filled circle button for theme accent selection
pub struct ColorDotStyle(pub Color, pub bool /* active */);

impl button::StyleSheet for ColorDotStyle {
    type Style = iced::Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.0)),
            border: Border {
                color: if self.1 {
                    Color::WHITE
                } else {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.3)
                },
                width: if self.1 { 2.0 } else { 1.0 },
                radius: Radius::from(50.0),
            },
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut a = self.active(style);
        a.border.color = Color::WHITE;
        a.border.width = 2.0;
        a
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
}

// Modern glass window container with gradient background
pub struct WindowStyle(pub ColorPalette);

impl container::StyleSheet for WindowStyle {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(self.0.background)),
            text_color: Some(self.0.text),
            border: Border {
                color: self.0.glass_border,
                width: 1.0,
                radius: Radius::from(design::BORDER_RADIUS_LARGE),
            },
            shadow: self.0.shadow(self.0.glass_shadow, 8.0, 32.0),
        }
    }
}

// Transparent container for content organization
pub struct ContainerStyle(pub ColorPalette);

impl container::StyleSheet for ContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: None, // Transparent for content flow
            text_color: Some(self.0.text),
            border: Border::default(),
            shadow: Shadow::default(),
        }
    }
}

// Modern glass card style for inputs and content areas
pub struct CardStyle(pub ColorPalette);

impl container::StyleSheet for CardStyle {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(self.0.surface)),
            text_color: Some(self.0.text),
            border: Border {
                color: self.0.glass_border,
                width: 1.0,
                radius: Radius::from(design::BORDER_RADIUS_SMALL),
            },
            shadow: self.0.shadow(self.0.glass_shadow, 4.0, 16.0),
        }
    }
}

// Modern option card button with glass effect
pub struct OptionCardStyle(pub ColorPalette, pub bool); // bool for selected state

impl button::StyleSheet for OptionCardStyle {
    type Style = iced::Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        if self.1 {
            // Selected state with vibrant accent
            button::Appearance {
                background: Some(Background::Color(self.0.primary_light)),
                text_color: self.0.text,
                border: Border {
                    color: self.0.primary,
                    width: 2.0,
                    radius: Radius::from(design::BORDER_RADIUS_SMALL),
                },
                shadow: self.0.shadow(
                    Color::from_rgba(self.0.primary.r, self.0.primary.g, self.0.primary.b, 0.3),
                    4.0,
                    12.0,
                ),
                shadow_offset: Vector::new(0.0, 0.0),
            }
        } else {
            button::Appearance {
                background: Some(Background::Color(self.0.surface)),
                text_color: self.0.text,
                border: Border {
                    color: self.0.glass_border,
                    width: 1.0,
                    radius: Radius::from(design::BORDER_RADIUS_SMALL),
                },
                shadow: self.0.shadow(self.0.glass_shadow, 2.0, 8.0),
                shadow_offset: Vector::new(0.0, 0.0),
            }
        }
    }

    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        if self.1 {
            button::Appearance {
                background: Some(Background::Color(self.0.primary_light)),
                text_color: self.0.text,
                border: Border {
                    color: self.0.primary_hover,
                    width: 2.0,
                    radius: Radius::from(design::BORDER_RADIUS_SMALL),
                },
                shadow: self.0.shadow(
                    Color::from_rgba(self.0.primary.r, self.0.primary.g, self.0.primary.b, 0.5),
                    6.0,
                    20.0,
                ),
                shadow_offset: Vector::new(0.0, 0.0),
            }
        } else {
            button::Appearance {
                background: Some(Background::Color(self.0.surface_hover)),
                text_color: self.0.text,
                border: Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
                    width: 1.0,
                    radius: Radius::from(design::BORDER_RADIUS_SMALL),
                },
                shadow: self.0.shadow(self.0.glass_shadow, 4.0, 16.0),
                shadow_offset: Vector::new(0.0, 0.0),
            }
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
}

// Legacy alias for compatibility

// Modern primary button with vibrant gradient and glow
pub struct PrimaryButton(pub ColorPalette);

impl button::StyleSheet for PrimaryButton {
    type Style = iced::Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.0.primary)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(design::BORDER_RADIUS_SMALL),
            },
            shadow: self.0.shadow(
                Color::from_rgba(self.0.primary.r, self.0.primary.g, self.0.primary.b, 0.4),
                4.0,
                16.0,
            ),
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.0.primary_hover)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(design::BORDER_RADIUS_SMALL),
            },
            shadow: self.0.shadow(
                Color::from_rgba(self.0.primary.r, self.0.primary.g, self.0.primary.b, 0.6),
                6.0,
                24.0,
            ),
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
}

// Modern secondary button with glass effect
pub struct SecondaryButton(pub ColorPalette);

impl button::StyleSheet for SecondaryButton {
    type Style = iced::Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.0.surface)),
            text_color: self.0.text,
            border: Border {
                color: self.0.glass_border,
                width: 1.0,
                radius: Radius::from(design::BORDER_RADIUS_SMALL),
            },
            shadow: self.0.shadow(self.0.glass_shadow, 2.0, 8.0),
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.0.surface_hover)),
            text_color: self.0.text,
            border: Border {
                color: self.0.primary,
                width: 1.0,
                radius: Radius::from(design::BORDER_RADIUS_SMALL),
            },
            shadow: self.0.shadow(
                Color::from_rgba(self.0.primary.r, self.0.primary.g, self.0.primary.b, 0.2),
                4.0,
                12.0,
            ),
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.hovered(style)
    }
}

// Modern compact floating window with enhanced glass effect
pub struct CompactStyle(pub ColorPalette);

impl container::StyleSheet for CompactStyle {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(
                self.0.surface_elevated.r,
                self.0.surface_elevated.g,
                self.0.surface_elevated.b,
                0.95, // High transparency for floating effect
            ))),
            text_color: Some(self.0.text),
            border: Border {
                color: self.0.glass_border,
                width: 1.0,
                radius: Radius::from(design::BORDER_RADIUS_ROUND),
            },
            shadow: self
                .0
                .shadow(Color::from_rgba(0.0, 0.0, 0.0, 0.5), 8.0, 32.0),
        }
    }
}

// Modern compact button with subtle glass effect
pub struct CompactButton(pub ColorPalette);

impl button::StyleSheet for CompactButton {
    type Style = iced::Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(
                self.0.danger.r,
                self.0.danger.g,
                self.0.danger.b,
                0.95,
            ))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.3),
                width: 1.0,
                radius: Radius::from(design::BORDER_RADIUS_ROUND),
            },
            shadow: self
                .0
                .shadow(Color::from_rgba(1.0, 0.27, 0.27, 0.3), 2.0, 8.0),
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.0.danger)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(design::BORDER_RADIUS_ROUND),
            },
            shadow: self
                .0
                .shadow(Color::from_rgba(1.0, 0.27, 0.27, 0.5), 4.0, 12.0),
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.hovered(style)
    }
}

// Error popup with danger-tinted border
pub struct ErrorIndicator(pub ColorPalette);

impl container::StyleSheet for ErrorIndicator {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(
                self.0.surface.r,
                self.0.surface.g,
                self.0.surface.b,
                0.95,
            ))),
            text_color: Some(self.0.text),
            border: Border {
                color: Color::from_rgba(self.0.danger.r, self.0.danger.g, self.0.danger.b, 0.6),
                width: 1.0,
                radius: Radius::from(design::BORDER_RADIUS_LARGE),
            },
            shadow: self
                .0
                .shadow(Color::from_rgba(1.0, 0.27, 0.27, 0.35), 4.0, 16.0),
        }
    }
}

// Modern recording indicator with vibrant glow effect
pub struct RecordingIndicator(pub ColorPalette);

impl container::StyleSheet for RecordingIndicator {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(
                self.0.surface.r,
                self.0.surface.g,
                self.0.surface.b,
                0.9, // Glass-morphism background
            ))),
            text_color: Some(self.0.text),
            border: Border {
                color: self.0.glass_border,
                width: 1.0,
                radius: Radius::from(design::BORDER_RADIUS_ROUND),
            },
            shadow: self
                .0
                .shadow(Color::from_rgba(1.0, 0.27, 0.27, 0.4), 4.0, 16.0),
        }
    }
}
