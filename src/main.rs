mod audio;
mod config;
mod recorder;
mod theme;

use anyhow::Result;
use iced::widget::{Column, Space, button, column, container, pick_list, row, text};
use iced::{
    Application, Command, Element, Font, Length, Point, Settings, Size, Theme as IcedTheme,
    alignment, executor, window,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use config::Config;
use recorder::{
    CaptureRegion, OutputFormat, OutputInfo, Recorder, RecordingConfig, list_outputs,
    validate_geometry,
};
use theme::{Theme, ThemeAccent, design};

// Static slice avoids a heap allocation on every settings view render
static OUTPUT_FORMATS: &[OutputFormat] =
    &[OutputFormat::WebM, OutputFormat::Mp4, OutputFormat::Mkv];

fn settings_window_height(scale: f32, has_display: bool) -> f32 {
    let lh = 1.4_f32;
    let win_pad = 2.0 * design::BASE_WINDOW_PADDING as f32;
    let sp = design::BASE_SECTION_SPACING as f32;
    let title_h = design::BASE_TITLE_SIZE as f32 * lh
        + design::BASE_TINY_SPACE
        + design::BASE_SUBTITLE_SIZE as f32 * lh
        + design::BASE_SECTION_SPACING as f32;
    let label_h = design::BASE_LABEL_SIZE as f32 * lh;
    let small_sp = design::BASE_SMALL_SPACE;
    let btn_h = design::BASE_BUTTON_HEIGHT as f32;
    let list_h =
        design::BASE_INPUT_TEXT_SIZE as f32 * lh + 2.0 * design::BASE_CONTAINER_PADDING as f32;
    let btn_sec = label_h + small_sp + btn_h;
    let list_sec = label_h + small_sp + list_h;
    let left_h = if has_display {
        btn_sec + sp + list_sec + sp + btn_sec
    } else {
        btn_sec + sp + btn_sec
    };
    let right_h = btn_sec + sp + list_sec + sp + list_sec;
    let record_h =
        2.0 * design::BASE_BUTTON_PADDING_V as f32 + design::BASE_INPUT_TEXT_SIZE as f32 * lh;
    (win_pad + title_h + sp + left_h.max(right_h) + sp + record_h + 16.0) * scale
}

fn main() -> Result<()> {
    let screen_size = App::detect_screen_size();
    let scale_factor = design::scale_factor(screen_size.width, screen_size.height);

    let config = Config::load().unwrap_or_default();
    let outputs = list_outputs();
    let has_display = matches!(config.region, CaptureRegion::FullScreen) && !outputs.is_empty();

    let initial_size = Size::new(
        (design::BASE_WINDOW_WIDTH * scale_factor)
            .clamp(design::MIN_WINDOW_WIDTH, design::MAX_WINDOW_WIDTH),
        settings_window_height(scale_factor, has_display),
    );

    App::run(Settings {
        window: iced::window::Settings {
            size: initial_size,
            resizable: false,
            decorations: true,
            transparent: false,
            ..Default::default()
        },
        flags: (config, outputs),
        antialiasing: true,
        default_font: Font::default(),
        ..Default::default()
    })?;
    Ok(())
}

#[derive(Debug, Clone)]
enum Message {
    FormatSelected(OutputFormat),
    ToggleRegion(bool),
    ToggleSystemAudio(bool),
    ToggleMicAudio(bool),
    OutputSelected(OutputInfo),
    FramerateSelected(u32),
    AccentSelected(ThemeAccent),
    ToggleGlow,
    BrowseFolder,
    FolderSelected(PathBuf),
    StartRecording,
    RegionSelected(Option<String>),
    StopRecording,
    DismissError,
    Tick,
    NoOp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppState {
    Settings,
    CompactCountdown(u8),
    CompactRecording,
    CompactError,
}

struct App {
    state: AppState,
    config: Config,
    recorder: Option<Recorder>,
    recording_start: Option<Instant>,
    recording_duration: Duration,
    theme: Theme,
    screen_size: Size,
    scale_factor: f32,
    available_outputs: Vec<OutputInfo>,
    pending_geometry: Option<String>,
    last_error: Option<String>,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = IcedTheme;
    type Flags = (Config, Vec<OutputInfo>);

    fn new((mut config, available_outputs): Self::Flags) -> (Self, Command<Message>) {
        let screen_size = Self::detect_screen_size();
        let scale_factor = design::scale_factor(screen_size.width, screen_size.height);

        // Always pre-select first display when none is configured
        if config.output.is_none()
            && let Some(first) = available_outputs.first()
        {
            config.output = Some(first.name.clone());
            let _ = config.save();
        }

        let theme = Theme::with_accent(config.accent, config.glow);
        let app = App {
            state: AppState::Settings,
            config,
            recorder: None,
            recording_start: None,
            recording_duration: Duration::default(),
            theme,
            screen_size,
            scale_factor,
            available_outputs,
            pending_geometry: None,
            last_error: None,
        };

        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("WF Recorder")
    }

    fn theme(&self) -> IcedTheme {
        IcedTheme::Dark
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FormatSelected(format) => {
                self.config.format = format;
                let _ = self.config.save();
                Command::none()
            }
            Message::ToggleRegion(is_fullscreen) => {
                self.config.region = if is_fullscreen {
                    CaptureRegion::FullScreen
                } else {
                    CaptureRegion::Selection
                };
                let _ = self.config.save();
                window::resize(window::Id::MAIN, self.get_settings_size())
            }
            Message::ToggleSystemAudio(on) => {
                self.config.audio.system = on;
                let _ = self.config.save();
                Command::none()
            }
            Message::ToggleMicAudio(on) => {
                self.config.audio.microphone = on;
                let _ = self.config.save();
                Command::none()
            }
            Message::OutputSelected(info) => {
                self.config.output = Some(info.name);
                let _ = self.config.save();
                Command::none()
            }
            Message::FramerateSelected(fps) => {
                self.config.framerate = fps;
                let _ = self.config.save();
                Command::none()
            }
            Message::AccentSelected(accent) => {
                self.config.accent = accent;
                self.theme = Theme::with_accent(accent, self.config.glow);
                let _ = self.config.save();
                Command::none()
            }
            Message::ToggleGlow => {
                self.config.glow = !self.config.glow;
                self.theme = Theme::with_accent(self.config.accent, self.config.glow);
                let _ = self.config.save();
                Command::none()
            }
            Message::BrowseFolder => {
                let current_dir = self.config.output_dir.clone();
                Command::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_directory(current_dir)
                            .pick_folder()
                            .await
                            .map(|path| path.path().to_path_buf())
                    },
                    |path| {
                        if let Some(p) = path {
                            Message::FolderSelected(p)
                        } else {
                            Message::NoOp
                        }
                    },
                )
            }
            Message::FolderSelected(path) => {
                self.config.output_dir = path;
                let _ = self.config.save();
                Command::none()
            }
            Message::StartRecording => {
                self.last_error = None;
                if matches!(self.config.region, CaptureRegion::Selection) {
                    // Run slurp first, before showing countdown
                    Command::perform(
                        async {
                            let result = tokio::task::spawn_blocking(|| {
                                std::process::Command::new("slurp")
                                    .output()
                                    .ok()
                                    .and_then(|o| {
                                        let geo =
                                            String::from_utf8_lossy(&o.stdout).trim().to_string();
                                        if geo.is_empty() { None } else { Some(geo) }
                                    })
                            })
                            .await;
                            result.ok().flatten()
                        },
                        Message::RegionSelected,
                    )
                } else {
                    self.state = AppState::CompactCountdown(3);
                    let compact_size = self.get_compact_size();
                    let position = self.get_compact_position();
                    Command::batch([
                        window::resize(window::Id::MAIN, compact_size),
                        window::move_to(window::Id::MAIN, position),
                        Command::perform(
                            async {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            },
                            |_| Message::Tick,
                        ),
                    ])
                }
            }
            Message::RegionSelected(geo) => {
                match geo {
                    None => Command::none(), // user cancelled slurp, stay on settings
                    Some(geometry) => {
                        // Pre-validate before countdown: detect multi-display overlap immediately
                        if let Some(err) = validate_geometry(&geometry, &self.available_outputs) {
                            self.last_error = Some(err);
                            self.state = AppState::CompactError;
                            let error_size = self.get_error_size();
                            let pos = self.get_compact_position_for(error_size);
                            return Command::batch([
                                window::resize(window::Id::MAIN, error_size),
                                window::move_to(window::Id::MAIN, pos),
                            ]);
                        }
                        self.pending_geometry = Some(geometry);
                        self.state = AppState::CompactCountdown(3);
                        let compact_size = self.get_compact_size();
                        let position = self.get_compact_position();
                        Command::batch([
                            window::resize(window::Id::MAIN, compact_size),
                            window::move_to(window::Id::MAIN, position),
                            Command::perform(
                                async {
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                },
                                |_| Message::Tick,
                            ),
                        ])
                    }
                }
            }
            Message::DismissError => {
                self.last_error = None;
                self.state = AppState::Settings;
                let settings_size = self.get_settings_size();
                let center = self.get_center_position(settings_size);
                Command::batch([
                    window::resize(window::Id::MAIN, settings_size),
                    window::move_to(window::Id::MAIN, center),
                ])
            }
            Message::StopRecording => {
                if let Some(recorder) = &mut self.recorder {
                    let _ = recorder.stop();
                }
                self.recorder = None;
                self.state = AppState::Settings;
                self.recording_start = None;
                self.recording_duration = Duration::default();

                let settings_size = self.get_settings_size();
                let center_position = self.get_center_position(settings_size);
                window::move_to(window::Id::MAIN, center_position)
            }
            Message::Tick => {
                match self.state {
                    AppState::CompactCountdown(count) => {
                        if count > 1 {
                            self.state = AppState::CompactCountdown(count - 1);
                            Command::perform(
                                async {
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                },
                                |_| Message::Tick,
                            )
                        } else {
                            // Start recording
                            let output_geometry =
                                if matches!(self.config.region, CaptureRegion::FullScreen) {
                                    self.config.output.as_ref().and_then(|name| {
                                        self.available_outputs
                                            .iter()
                                            .find(|o| &o.name == name)
                                            .and_then(|o| o.geometry.clone())
                                    })
                                } else {
                                    None
                                };
                            let recording_config = RecordingConfig {
                                format: self.config.format,
                                audio: self.config.audio,
                                region: self.config.region,
                                output: self.config.output.clone(),
                                framerate: self.config.framerate,
                                output_dir: self.config.output_dir.clone(),
                                geometry: self.pending_geometry.take().or(output_geometry),
                            };

                            let mut recorder = Recorder::new(recording_config);
                            if let Err(e) = recorder.start() {
                                eprintln!("Failed to start recording: {e}");
                                self.state = AppState::Settings;
                                let settings_size = self.get_settings_size();
                                let center_position = self.get_center_position(settings_size);
                                window::move_to(window::Id::MAIN, center_position)
                            } else {
                                self.recorder = Some(recorder);
                                // Always use compact recording mode - non-intrusive
                                self.state = AppState::CompactRecording;
                                self.recording_start = Some(Instant::now());

                                // Start timer updates
                                Command::perform(
                                    async {
                                        tokio::time::sleep(Duration::from_secs(1)).await;
                                    },
                                    |_| Message::Tick,
                                )
                            }
                        }
                    }
                    AppState::CompactRecording => {
                        if let Some(start) = self.recording_start {
                            self.recording_duration = start.elapsed();
                        }
                        // Detect if wf-recorder exited unexpectedly (e.g. geometry error)
                        if let Some(recorder) = &mut self.recorder
                            && let Some(error) = recorder.check_running()
                        {
                            self.recorder = None;
                            self.state = AppState::CompactError;
                            self.recording_start = None;
                            self.recording_duration = Duration::default();
                            self.last_error = Some(error);
                            let error_size = self.get_error_size();
                            let pos = self.get_compact_position_for(error_size);
                            return Command::batch([
                                window::resize(window::Id::MAIN, error_size),
                                window::move_to(window::Id::MAIN, pos),
                            ]);
                        }
                        Command::perform(
                            async {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            },
                            |_| Message::Tick,
                        )
                    }
                    AppState::Settings | AppState::CompactError => Command::none(),
                }
            }
            Message::NoOp => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = match self.state {
            AppState::Settings => self.view_settings(),
            AppState::CompactCountdown(count) => self.view_compact_countdown(count),
            AppState::CompactRecording => self.view_compact_recording(),
            AppState::CompactError => self.view_compact_error(),
        };

        // Dynamic window padding based on scale factor and mode
        let padding = match self.state {
            AppState::CompactCountdown(_) | AppState::CompactRecording | AppState::CompactError => {
                design::COMPACT_BUTTON_PADDING
            }
            _ => design::window_padding(self.scale_factor),
        };

        // Main window container with responsive padding
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(padding)
            .style(iced::theme::Container::Custom(Box::new(
                theme::WindowStyle(self.theme.colors),
            )))
            .into()
    }
}

impl App {
    // Detect screen size using multiple methods
    pub fn detect_screen_size() -> Size {
        // Method 1: Check environment variables
        if let (Ok(width), Ok(height)) = (
            std::env::var("SCREEN_WIDTH")
                .and_then(|w| w.parse::<f32>().map_err(|_| std::env::VarError::NotPresent)),
            std::env::var("SCREEN_HEIGHT")
                .and_then(|h| h.parse::<f32>().map_err(|_| std::env::VarError::NotPresent)),
        ) {
            return Size::new(width, height);
        }

        // Method 2: Try wlr-randr on Wayland (wlroots compositors)
        if let Ok(output) = std::process::Command::new("wlr-randr").output()
            && let Ok(output_str) = String::from_utf8(output.stdout)
        {
            // Lines with current mode look like: "  1920x1080 px, 60.000 Hz (current)"
            for line in output_str.lines() {
                if line.contains("current")
                    && let Some(resolution) = line.split_whitespace().next()
                {
                    let parts: Vec<&str> = resolution.split('x').collect();
                    if parts.len() == 2
                        && let (Ok(w), Ok(h)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>())
                    {
                        return Size::new(w, h);
                    }
                }
            }
        }

        // Method 3: Common screen size detection based on typical resolutions
        // This could be expanded with more platform-specific code

        // Default fallback - use reference resolution
        Size::new(design::REFERENCE_WIDTH, design::REFERENCE_HEIGHT)
    }
    fn view_settings(&self) -> Element<'_, Message> {
        let colors = self.theme.colors;

        let title_size = design::title_size(self.scale_factor);
        let subtitle_size = design::subtitle_size(self.scale_factor);
        let section_spacing = design::section_spacing(self.scale_factor);
        let container_padding = design::container_padding(self.scale_factor);
        let btn_gap = Length::Fixed(container_padding as f32);

        // Theme accent dots
        let dot_size = 18.0_f32 * self.scale_factor;
        let accent_dot = |accent: ThemeAccent| -> Element<Message> {
            let active = self.theme.accent == accent;
            button(Space::with_width(Length::Fixed(dot_size)))
                .on_press(Message::AccentSelected(accent))
                .width(Length::Fixed(dot_size))
                .height(Length::Fixed(dot_size))
                .padding(0)
                .style(iced::theme::Button::Custom(Box::new(theme::ColorDotStyle(
                    accent.color(),
                    active,
                ))))
                .into()
        };
        let dot_gap = Length::Fixed(6.0 * self.scale_factor);
        let s = self.scale_factor;
        let dot_size = 12.0 * s;
        let pill_w = 30.0 * s;
        let pill_h = 18.0 * s;
        let gap = pill_w - dot_size - 6.0 * s;
        let dot = container(Space::new(Length::Fixed(dot_size), Length::Fixed(dot_size))).style(
            iced::theme::Container::Custom(Box::new(theme::ToggleDotStyle)),
        );
        let toggle_inner = if self.config.glow {
            row![Space::with_width(Length::Fixed(gap)), dot]
                .align_items(alignment::Alignment::Center)
        } else {
            row![dot, Space::with_width(Length::Fixed(gap))]
                .align_items(alignment::Alignment::Center)
        };
        let glow_btn = button(toggle_inner)
            .on_press(Message::ToggleGlow)
            .width(Length::Fixed(pill_w))
            .height(Length::Fixed(pill_h))
            .padding([3, 3])
            .style(iced::theme::Button::Custom(Box::new(
                theme::GlowToggleStyle(colors, self.config.glow),
            )));

        let accent_dots = row![
            accent_dot(ThemeAccent::Blue),
            Space::with_width(dot_gap),
            accent_dot(ThemeAccent::Purple),
            Space::with_width(dot_gap),
            accent_dot(ThemeAccent::Pink),
            Space::with_width(dot_gap),
            accent_dot(ThemeAccent::Red),
            Space::with_width(dot_gap),
            accent_dot(ThemeAccent::Orange),
            Space::with_width(dot_gap),
            glow_btn,
        ]
        .align_items(alignment::Alignment::Center);

        // Title
        let title_section = container(
            row![
                column![
                    text("WF Recorder")
                        .size(title_size)
                        .font(Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .style(iced::theme::Text::Color(colors.text)),
                    text(format!(
                        "{} • {}FPS",
                        self.config.format, self.config.framerate
                    ))
                    .size(subtitle_size)
                    .style(iced::theme::Text::Color(colors.text_secondary)),
                ]
                .spacing(design::tiny_space(self.scale_factor) as u16)
                .width(Length::Fill),
                accent_dots,
            ]
            .align_items(alignment::Alignment::Center),
        )
        .width(Length::Fill)
        .padding([0, 0, section_spacing, 0])
        .style(iced::theme::Container::Custom(Box::new(
            theme::ContainerStyle(colors),
        )));

        // Capture mode
        let capture_section = self.create_section(
            "CAPTURE MODE",
            row![
                self.create_option_button(
                    "🖥",
                    "Screen",
                    matches!(self.config.region, CaptureRegion::FullScreen),
                    Message::ToggleRegion(true)
                ),
                Space::with_width(btn_gap),
                self.create_option_button(
                    "◰",
                    "Region",
                    matches!(self.config.region, CaptureRegion::Selection),
                    Message::ToggleRegion(false)
                ),
            ],
        );

        // Display picker (left col, conditional)
        let display_section = if matches!(self.config.region, CaptureRegion::FullScreen)
            && !self.available_outputs.is_empty()
        {
            let options = self.available_outputs.clone();
            let selected = self.config.output.as_ref().and_then(|name| {
                self.available_outputs
                    .iter()
                    .find(|o| &o.name == name)
                    .cloned()
            });
            Some(
                self.create_section(
                    "DISPLAY",
                    container(
                        pick_list(options, selected, Message::OutputSelected)
                            .padding([container_padding, container_padding])
                            .width(Length::Fill)
                            .text_size(design::input_text_size(self.scale_factor)),
                    )
                    .width(Length::Fill)
                    .style(iced::theme::Container::Custom(Box::new(theme::CardStyle(
                        colors,
                    )))),
                ),
            )
        } else {
            None
        };

        // Framerate
        let fps_btn = |fps: u32| -> Element<Message> {
            let active = self.config.framerate == fps;
            button(
                container(
                    text(format!("{fps} fps")).size(design::button_text_size(self.scale_factor)),
                )
                .width(Length::Fill)
                .center_x()
                .center_y(),
            )
            .on_press(Message::FramerateSelected(fps))
            .padding([design::button_padding_v(self.scale_factor), 0])
            .width(Length::Fill)
            .height(Length::Fixed(
                design::button_height(self.scale_factor) as f32
            ))
            .style(iced::theme::Button::Custom(Box::new(
                theme::OptionCardStyle(self.theme.colors, active),
            )))
            .into()
        };
        let framerate_section = self.create_section(
            "FRAMERATE",
            row![
                fps_btn(24),
                Space::with_width(btn_gap),
                fps_btn(30),
                Space::with_width(btn_gap),
                fps_btn(60)
            ],
        );

        // Audio source — independent toggle buttons
        let audio_section = self.create_section(
            "AUDIO SOURCE",
            row![
                self.create_option_button(
                    "🔊",
                    "System",
                    self.config.audio.system,
                    Message::ToggleSystemAudio(!self.config.audio.system),
                ),
                Space::with_width(btn_gap),
                self.create_option_button(
                    "🎙",
                    "Micro",
                    self.config.audio.microphone,
                    Message::ToggleMicAudio(!self.config.audio.microphone),
                ),
            ],
        );

        // Output format
        let format_section = self.create_section(
            "OUTPUT FORMAT",
            container(
                pick_list(
                    OUTPUT_FORMATS,
                    Some(self.config.format),
                    Message::FormatSelected,
                )
                .padding([container_padding, container_padding])
                .width(Length::Fill)
                .text_size(design::input_text_size(self.scale_factor)),
            )
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(theme::CardStyle(
                colors,
            )))),
        );

        // Save location
        let folder_text = self.config.output_dir.to_string_lossy().to_string();
        let folder_display = if folder_text.len() > 22 {
            format!("...{}", &folder_text[folder_text.len() - 19..])
        } else {
            folder_text
        };
        let location_section = self.create_section(
            "SAVE LOCATION",
            container(
                row![
                    text(folder_display)
                        .size(design::button_text_size(self.scale_factor))
                        .style(iced::theme::Text::Color(colors.text_secondary)),
                    Space::with_width(Length::Fill),
                    button(text("Browse").size(design::button_text_size(self.scale_factor)))
                        .on_press(Message::BrowseFolder)
                        .padding([8, 16])
                        .style(iced::theme::Button::Custom(Box::new(
                            theme::SecondaryButton(colors)
                        ))),
                ]
                .align_items(alignment::Alignment::Center),
            )
            .padding(container_padding)
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(theme::CardStyle(
                colors,
            )))),
        );

        // Record button
        let record_button = button(
            container(text("Start Recording").size(design::input_text_size(self.scale_factor)))
                .width(Length::Fill)
                .center_x(),
        )
        .on_press(Message::StartRecording)
        .padding([
            design::button_padding_v(self.scale_factor),
            design::button_padding_h(self.scale_factor),
        ])
        .width(Length::Fill)
        .style(iced::theme::Button::Custom(Box::new(theme::PrimaryButton(
            colors,
        ))));

        // Left column: Capture Mode, [Display], Framerate
        let mut left_items: Vec<Element<Message>> = vec![capture_section];
        if let Some(section) = display_section {
            left_items.push(section);
        }
        left_items.push(framerate_section);
        let left_col = Column::with_children(left_items)
            .spacing(section_spacing)
            .width(Length::Fill);

        // Right column: Audio Source, Output Format, Save Location
        let right_col = column![audio_section, format_section, location_section]
            .spacing(section_spacing)
            .width(Length::Fill);

        let col_gap = Length::Fixed(section_spacing as f32);

        container(
            column![
                title_section,
                row![left_col, Space::with_width(col_gap), right_col],
                record_button,
            ]
            .spacing(section_spacing),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(
            theme::ContainerStyle(colors),
        )))
        .into()
    }

    // Helper to create sections with labels
    fn create_section<'a>(
        &self,
        label: &str,
        content: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        column![
            text(label)
                .size(design::label_size(self.scale_factor))
                .style(iced::theme::Text::Color(self.theme.colors.text_secondary)),
            Space::with_height(Length::Fixed(design::small_space(self.scale_factor))),
            content.into(),
        ]
        .spacing(0)
        .into()
    }

    // Helper to create option buttons matching onagre's row style
    fn create_option_button(
        &self,
        icon: &str,
        label: &str,
        is_active: bool,
        message: Message,
    ) -> Element<'_, Message> {
        button(
            column![
                text(icon).size(24),
                Space::with_height(Length::Fixed(design::tiny_space(self.scale_factor))),
                text(label).size(design::button_text_size(self.scale_factor))
            ]
            .spacing(0)
            .align_items(alignment::Alignment::Center)
            .width(Length::Fill),
        )
        .on_press(message)
        .padding([design::button_padding_v(self.scale_factor), 0])
        .width(Length::Fill)
        .height(Length::Fixed(
            design::button_height(self.scale_factor) as f32
        ))
        .style(iced::theme::Button::Custom(Box::new(
            theme::OptionCardStyle(self.theme.colors, is_active),
        )))
        .into()
    }

    fn view_compact_error(&self) -> Element<'_, Message> {
        let colors = self.theme.colors;
        let msg = self
            .last_error
            .as_deref()
            .unwrap_or("wf-recorder exited unexpectedly");

        let back_btn = button(
            container(text("Back").size(design::button_text_size(self.scale_factor)))
                .width(Length::Fill)
                .center_x()
                .center_y(),
        )
        .on_press(Message::DismissError)
        .width(Length::Shrink)
        .padding([4, 14])
        .style(iced::theme::Button::Custom(Box::new(theme::PrimaryButton(
            colors,
        ))));

        container(
            column![
                text(msg)
                    .size(design::label_size(self.scale_factor))
                    .style(iced::theme::Text::Color(colors.danger)),
                back_btn,
            ]
            .spacing(8)
            .align_items(alignment::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding(design::COMPACT_BUTTON_PADDING)
        .style(iced::theme::Container::Custom(Box::new(
            theme::ErrorIndicator(colors),
        )))
        .into()
    }

    // Compact countdown view - minimal UI for recording
    fn stop_button(&self) -> Element<'_, Message> {
        let colors = self.theme.colors;
        button(
            container(Space::new(Length::Fixed(10.0), Length::Fixed(10.0))).style(
                iced::theme::Container::Custom(Box::new(theme::StopIconStyle)),
            ),
        )
        .on_press(Message::StopRecording)
        .padding(design::COMPACT_BUTTON_PADDING)
        .style(iced::theme::Button::Custom(Box::new(theme::CompactButton(
            colors,
        ))))
        .into()
    }

    fn view_compact_countdown(&self, count: u8) -> Element<'_, Message> {
        let colors = self.theme.colors;
        container(
            row![
                text(count.to_string())
                    .size(design::compact_countdown_size(self.scale_factor))
                    .font(Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .style(iced::theme::Text::Color(colors.primary)),
                Space::with_width(Length::Fixed(design::small_space(self.scale_factor))),
                self.stop_button(),
            ]
            .align_items(alignment::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding(design::COMPACT_BUTTON_PADDING)
        .style(iced::theme::Container::Custom(Box::new(
            theme::CompactStyle(colors),
        )))
        .into()
    }

    fn view_compact_recording(&self) -> Element<'_, Message> {
        let colors = self.theme.colors;
        let minutes = self.recording_duration.as_secs() / 60;
        let seconds = self.recording_duration.as_secs() % 60;
        container(
            row![
                text(format!("{minutes:02}:{seconds:02}"))
                    .size(design::timer_text_size(self.scale_factor))
                    .font(Font {
                        family: iced::font::Family::Monospace,
                        weight: iced::font::Weight::Medium,
                        ..Default::default()
                    })
                    .style(iced::theme::Text::Color(colors.text)),
                Space::with_width(Length::Fixed(6.0 * self.scale_factor.max(1.0))),
                self.stop_button(),
            ]
            .align_items(alignment::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding(design::COMPACT_BUTTON_PADDING)
        .style(iced::theme::Container::Custom(Box::new(
            theme::RecordingIndicator(colors),
        )))
        .into()
    }

    // Helper methods for window sizing and positioning
    fn get_settings_size(&self) -> Size {
        let has_display = matches!(self.config.region, CaptureRegion::FullScreen)
            && !self.available_outputs.is_empty();
        Size::new(
            (design::BASE_WINDOW_WIDTH * self.scale_factor)
                .clamp(design::MIN_WINDOW_WIDTH, design::MAX_WINDOW_WIDTH),
            settings_window_height(self.scale_factor, has_display),
        )
    }

    fn get_compact_size(&self) -> Size {
        // Scale compact window slightly for very high DPI displays
        let compact_scale = self.scale_factor.max(1.0);
        Size::new(
            design::COMPACT_WINDOW_WIDTH * compact_scale,
            design::COMPACT_WINDOW_HEIGHT * compact_scale,
        )
    }

    fn get_error_size(&self) -> Size {
        let s = self.scale_factor.max(1.0);
        Size::new(
            design::ERROR_WINDOW_WIDTH * s,
            design::ERROR_WINDOW_HEIGHT * s,
        )
    }

    /// Returns a top-right position for a window of the given size.
    fn get_compact_position_for(&self, size: Size) -> Point {
        let padding = design::COMPACT_WINDOW_PADDING * self.scale_factor;
        Point::new(
            (self.screen_size.width - size.width - padding).max(0.0),
            padding,
        )
    }

    fn get_compact_position(&self) -> Point {
        self.get_compact_position_for(self.get_compact_size())
    }

    fn get_center_position(&self, window_size: Size) -> Point {
        let x = ((self.screen_size.width - window_size.width) / 2.0).max(0.0);
        let y = ((self.screen_size.height - window_size.height) / 2.0).max(0.0);

        Point::new(x, y)
    }
}

// Implement Display for our types
impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::WebM => write!(f, "WebM"),
            OutputFormat::Mp4 => write!(f, "MP4"),
            OutputFormat::Mkv => write!(f, "MKV"),
        }
    }
}

impl std::fmt::Display for CaptureRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureRegion::FullScreen => write!(f, "Full Screen"),
            CaptureRegion::Selection => write!(f, "Select Region"),
        }
    }
}
