use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    WebM,
    Mp4,
    Mkv,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::WebM => "webm",
            OutputFormat::Mp4 => "mp4",
            OutputFormat::Mkv => "mkv",
        }
    }

    fn codec(&self) -> &'static str {
        match self {
            OutputFormat::WebM => "libvpx",
            OutputFormat::Mp4 => "libx264",
            OutputFormat::Mkv => "libx264",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AudioConfig {
    #[serde(default)]
    pub system: bool,
    #[serde(default)]
    pub microphone: bool,
}

impl AudioConfig {
    pub fn none() -> Self {
        Self {
            system: false,
            microphone: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureRegion {
    FullScreen,
    Selection,
}

#[derive(Debug, Clone)]
pub struct RecordingConfig {
    pub format: OutputFormat,
    pub audio: AudioConfig,
    pub region: CaptureRegion,
    pub output: Option<String>,
    pub framerate: u32,
    pub output_dir: PathBuf,
    pub geometry: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputInfo {
    pub name: String,
    pub description: String,
    /// Screen geometry in `WxH+X+Y` format, from wlr-randr.
    pub geometry: Option<String>,
    /// Human-readable position label (Left, Center, Right, Top, Bottom).
    pub position_label: Option<String>,
}

impl std::fmt::Display for OutputInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} — {}", self.name, self.description)?;
        if let Some(label) = &self.position_label {
            write!(f, " [{label}]")?;
        }
        Ok(())
    }
}

fn shorten_description(desc: &str, name: &str) -> String {
    let desc = desc
        .strip_suffix(&format!(" ({name})"))
        .unwrap_or(desc)
        .trim();

    let significant: Vec<&str> = desc
        .split_whitespace()
        .filter(|t| {
            !(t.starts_with("0x")
                || (t.len() > 6
                    && t.chars().all(|c| c.is_alphanumeric())
                    && t.chars().any(|c| c.is_ascii_digit())
                    && t.chars().any(|c| c.is_ascii_uppercase())))
        })
        .collect();

    let words = if significant.len() > 2 {
        &significant[significant.len() - 2..]
    } else {
        &significant[..]
    };

    words.join(" ")
}

/// Parses `wlr-randr` output and returns a map of output name -> `WxH+X+Y` geometry.
fn wlr_randr_geometries() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let Ok(output) = Command::new("wlr-randr").output() else {
        return map;
    };
    let text = String::from_utf8_lossy(&output.stdout).into_owned();

    let mut name: Option<String> = None;
    let mut res: Option<String> = None;
    let mut pos: Option<String> = None;

    let mut flush =
        |name: &mut Option<String>, res: &mut Option<String>, pos: &mut Option<String>| {
            if let (Some(n), Some(r), Some(p)) = (name.take(), res.take(), pos.take()) {
                map.insert(n, format!("{}+{}", r, p.replace(',', "+")));
            }
        };

    for line in text.lines() {
        if !line.starts_with(' ') && !line.is_empty() {
            flush(&mut name, &mut res, &mut pos);
            name = line.split_whitespace().next().map(str::to_string);
        } else if line.contains("current") && res.is_none() {
            if let Some(r) = line.split_whitespace().next() {
                res = Some(r.to_string());
            }
        } else if let Some(p) = line.trim().strip_prefix("Position: ") {
            pos = Some(p.to_string());
        }
    }
    flush(&mut name, &mut res, &mut pos);

    map
}

pub fn list_outputs() -> Vec<OutputInfo> {
    let Ok(result) = Command::new("wf-recorder").arg("-L").output() else {
        return Vec::new();
    };
    let geometries = wlr_randr_geometries();
    let mut outputs: Vec<OutputInfo> = String::from_utf8_lossy(&result.stdout)
        .lines()
        .filter_map(|line| {
            let name_start = line.find("Name: ")? + 6;
            let rest = &line[name_start..];
            let name_end = rest.find(' ').unwrap_or(rest.len());
            let name = rest[..name_end].to_string();

            let description = line
                .find("Description: ")
                .map(|i| shorten_description(&line[i + 13..], &name))
                .unwrap_or_default();

            let geometry = geometries.get(&name).cloned();

            Some(OutputInfo {
                name,
                description,
                geometry,
                position_label: None,
            })
        })
        .collect();

    assign_position_labels(&mut outputs);
    outputs
}

/// Parses a wlr-randr geometry string `"WxH+X+Y"` into `(x, y, w, h)`.
fn parse_wlr_geometry(geo: &str) -> Option<(i32, i32, i32, i32)> {
    let mut parts = geo.splitn(3, '+');
    let size = parts.next()?;
    let x: i32 = parts.next()?.parse().ok()?;
    let y: i32 = parts.next()?.parse().ok()?;
    let (w_str, h_str) = size.split_once('x')?;
    let w: i32 = w_str.parse().ok()?;
    let h: i32 = h_str.parse().ok()?;
    Some((x, y, w, h))
}

/// Validates a slurp geometry (`"X,Y WxH"`) against known outputs.
/// Returns `Some(error_message)` if the region spans multiple displays, `None` if valid.
/// If output geometry info is unavailable, passes through without error.
pub fn validate_geometry(geometry: &str, outputs: &[OutputInfo]) -> Option<String> {
    if outputs.iter().all(|o| o.geometry.is_none()) {
        return None; // No geometry info available, can't validate
    }

    // Parse slurp format: "X,Y WxH"
    let (pos, size) = geometry.split_once(' ')?;
    let (x_str, y_str) = pos.split_once(',')?;
    let (w_str, h_str) = size.split_once('x')?;
    let rx: i32 = x_str.parse().ok()?;
    let ry: i32 = y_str.parse().ok()?;
    let rw: i32 = w_str.parse().ok()?;
    let rh: i32 = h_str.parse().ok()?;

    // Count outputs whose bounding box intersects the selected region
    let overlapping = outputs
        .iter()
        .filter(|o| {
            o.geometry
                .as_deref()
                .and_then(parse_wlr_geometry)
                .map(|(ox, oy, ow, oh)| {
                    rx < ox + ow && rx + rw > ox && ry < oy + oh && ry + rh > oy
                })
                .unwrap_or(false)
        })
        .count();

    if overlapping > 1 {
        Some(
            "Selected region spans multiple displays.\nPlease select within a single display."
                .to_string(),
        )
    } else {
        None
    }
}

/// Assigns position labels (Left/Right, Top/Bottom, combinations) based on relative positions.
/// Handles grids, pure horizontal/vertical, and 4+ screen layouts.
fn assign_position_labels(outputs: &mut [OutputInfo]) {
    if outputs.len() < 2 {
        return;
    }

    // Parse X,Y offsets from geometry strings
    let positions: Vec<Option<(i32, i32)>> = outputs
        .iter()
        .map(|o| {
            let geo = o.geometry.as_ref()?;
            let mut parts = geo.split('+').skip(1);
            let x = parts.next()?.parse::<i32>().ok()?;
            let y = parts.next()?.parse::<i32>().ok()?;
            Some((x, y))
        })
        .collect();

    // Collect unique sorted X and Y values
    let mut unique_xs: Vec<i32> = positions.iter().filter_map(|p| p.map(|(x, _)| x)).collect();
    let mut unique_ys: Vec<i32> = positions.iter().filter_map(|p| p.map(|(_, y)| y)).collect();
    unique_xs.sort_unstable();
    unique_xs.dedup();
    unique_ys.sort_unstable();
    unique_ys.dedup();

    let col_labels = axis_labels(unique_xs.len(), true);
    let row_labels = axis_labels(unique_ys.len(), false);
    let multi_col = unique_xs.len() > 1;
    let multi_row = unique_ys.len() > 1;

    for (output, pos) in outputs.iter_mut().zip(positions.iter()) {
        let Some((x, y)) = pos else { continue };
        let col = unique_xs.partition_point(|&v| v < *x);
        let row = unique_ys.partition_point(|&v| v < *y);

        output.position_label = Some(match (multi_col, multi_row) {
            (true, false) => col_labels[col].clone(),
            (false, true) => row_labels[row].clone(),
            (true, true) => format!("{}-{}", row_labels[row], col_labels[col]),
            (false, false) => return,
        });
    }
}

/// Returns position labels for N items along one axis.
fn axis_labels(count: usize, horizontal: bool) -> Vec<String> {
    let (first, mid, last) = if horizontal {
        ("Left", "Center", "Right")
    } else {
        ("Top", "Center", "Bottom")
    };
    match count {
        0 | 1 => vec![String::new(); count],
        2 => vec![first.into(), last.into()],
        3 => vec![first.into(), mid.into(), last.into()],
        4 => {
            let (mid_first, mid_last) = if horizontal {
                ("Center-Left", "Center-Right")
            } else {
                ("Center-Top", "Center-Bottom")
            };
            vec![first.into(), mid_first.into(), mid_last.into(), last.into()]
        }
        n => {
            let mut labels = vec![first.into()];
            for i in 1..n - 1 {
                labels.push(format!("{mid} {i}"));
            }
            labels.push(last.into());
            labels
        }
    }
}

/// Runs `pactl list sources short` once and returns `(monitors, mics)`.
/// `monitors` are `.monitor` sources; `mics` are real input sources (RUNNING first).
fn list_sources() -> (Vec<String>, Vec<(String, bool)>) {
    let Ok(output) = Command::new("pactl")
        .args(["list", "sources", "short"])
        .output()
    else {
        return (Vec::new(), Vec::new());
    };
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let mut monitors = Vec::new();
    let mut mics: Vec<(String, bool)> = Vec::new();
    for line in text.lines() {
        let Some(name) = line.split('\t').nth(1).map(str::trim) else {
            continue;
        };
        if name.ends_with(".monitor") {
            monitors.push(name.to_string());
        } else {
            mics.push((name.to_string(), line.ends_with("RUNNING")));
        }
    }
    (monitors, mics)
}

fn pick_monitor(monitors: Vec<String>) -> Option<String> {
    if monitors.is_empty() {
        return None;
    }
    // Prefer the monitor that matches the default sink
    let default_sink = Command::new("pactl")
        .arg("get-default-sink")
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        })
        .or_else(|| {
            // Fallback: parse `pactl info`
            let info = Command::new("pactl").arg("info").output().ok()?;
            let info_str = String::from_utf8_lossy(&info.stdout).into_owned();
            let line = info_str.lines().find(|l| l.starts_with("Default Sink:"))?;
            Some(line.split_once(':')?.1.trim().to_string())
        })?;

    let preferred = format!("{}.monitor", default_sink);
    if monitors.iter().any(|m| m == &preferred) {
        Some(preferred)
    } else {
        monitors.into_iter().next()
    }
}

fn pick_mic(mics: Vec<(String, bool)>) -> Option<String> {
    // Prefer RUNNING sources over SUSPENDED ones
    mics.iter()
        .find(|(_, r)| *r)
        .or_else(|| mics.first())
        .map(|(name, _)| name.clone())
}

fn find_monitor_source() -> Option<String> {
    let (monitors, _) = list_sources();
    pick_monitor(monitors)
}

fn find_mic_source() -> Option<String> {
    let (_, mics) = list_sources();
    pick_mic(mics)
}

pub struct Recorder {
    config: RecordingConfig,
    child: Option<std::process::Child>,
    stderr: Option<std::process::ChildStderr>,
    /// PulseAudio module IDs to unload on stop (used for combined audio).
    audio_modules: Vec<String>,
}

impl Recorder {
    pub fn new(config: RecordingConfig) -> Self {
        Self {
            config,
            child: None,
            stderr: None,
            audio_modules: Vec::new(),
        }
    }

    fn generate_filename(&self) -> PathBuf {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let mut path = self.config.output_dir.clone();
        path.push(format!(
            "recording_{}.{}",
            timestamp,
            self.config.format.extension()
        ));
        path
    }

    pub fn start(&mut self) -> Result<()> {
        which::which("wf-recorder").context("wf-recorder not found. Please install it first.")?;

        let mut cmd = Command::new("wf-recorder");

        let output_file = self.generate_filename();
        cmd.arg("-f").arg(&output_file);
        cmd.arg("--codec").arg(self.config.format.codec());
        cmd.arg("-r").arg(self.config.framerate.to_string());

        match (self.config.audio.system, self.config.audio.microphone) {
            (false, false) => {}
            (true, false) => {
                // System audio only: use the default sink monitor
                if let Some(monitor) = find_monitor_source() {
                    cmd.arg(format!("--audio={monitor}"));
                } else {
                    cmd.arg("--audio");
                }
            }
            (false, true) => {
                // Microphone only: use the first non-monitor source
                if let Some(mic) = find_mic_source() {
                    cmd.arg(format!("--audio={mic}"));
                } else {
                    cmd.arg("--audio");
                }
            }
            (true, true) => {
                // Both: create a temporary combined null sink via PulseAudio.
                // Query pactl once and derive both sources from the same output.
                let (monitors, mics) = list_sources();
                let monitor = pick_monitor(monitors).unwrap_or_default();
                let mic = pick_mic(mics).unwrap_or_default();

                let m1 = Command::new("pactl")
                    .args([
                        "load-module",
                        "module-null-sink",
                        "sink_name=wfrecorder_mix",
                        "sink_properties=device.description=WF-Recorder-Mix",
                    ])
                    .output()
                    .context("Failed to create combined audio sink")?;
                let m1_id = String::from_utf8_lossy(&m1.stdout).trim().to_string();

                let m2 = Command::new("pactl")
                    .args([
                        "load-module",
                        "module-loopback",
                        &format!("source={}", monitor),
                        "sink=wfrecorder_mix",
                        "latency_msec=1",
                    ])
                    .output()
                    .context("Failed to loopback system audio")?;
                let m2_id = String::from_utf8_lossy(&m2.stdout).trim().to_string();

                let m3 = Command::new("pactl")
                    .args([
                        "load-module",
                        "module-loopback",
                        &format!("source={}", mic),
                        "sink=wfrecorder_mix",
                        "latency_msec=1",
                    ])
                    .output()
                    .context("Failed to loopback microphone")?;
                let m3_id = String::from_utf8_lossy(&m3.stdout).trim().to_string();

                // Store in reverse order so we unload loopbacks before the sink
                self.audio_modules = vec![m3_id, m2_id, m1_id];
                cmd.arg("--audio=wfrecorder_mix.monitor");
            }
        }

        if let (CaptureRegion::FullScreen, Some(output)) =
            (&self.config.region, &self.config.output)
        {
            cmd.arg("-o").arg(output);
        }

        if let Some(ref geometry) = self.config.geometry {
            cmd.arg("-g").arg(geometry);
        }

        cmd.stderr(std::process::Stdio::piped());
        let mut child = cmd.spawn().context("Failed to start wf-recorder")?;
        self.stderr = child.stderr.take();
        self.child = Some(child);

        Ok(())
    }

    /// Returns `Some(error_message)` if wf-recorder has exited unexpectedly.
    /// Returns `None` if the process is still running.
    pub fn check_running(&mut self) -> Option<String> {
        let child = self.child.as_mut()?;
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stderr_msg = String::new();
                if let Some(mut stderr) = self.stderr.take() {
                    use std::io::Read;
                    let _ = stderr.read_to_string(&mut stderr_msg);
                }
                let msg = stderr_msg.trim().to_string();
                Some(if !msg.is_empty() {
                    msg
                } else if !status.success() {
                    format!("wf-recorder exited with code {status}")
                } else {
                    "wf-recorder stopped unexpectedly".to_string()
                })
            }
            _ => None, // Still running
        }
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            Command::new("kill")
                .args(["-s", "INT", &child.id().to_string()])
                .status()
                .context("Failed to send SIGINT to wf-recorder")?;
            let _ = child.wait();
        }
        // Unload combined audio modules if any
        for module_id in self.audio_modules.drain(..) {
            if !module_id.is_empty() {
                Command::new("pactl")
                    .args(["unload-module", &module_id])
                    .status()
                    .ok();
            }
        }
        Ok(())
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
