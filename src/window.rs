use std::os::fd::OwnedFd;

use cosmic::app::Core;
use cosmic::cosmic_config::{self, ConfigGet, ConfigSet};
use cosmic::iced::Task;
use cosmic::Element;
use serde::{Deserialize, Serialize};
use zbus::blocking::Connection;
use zbus::zvariant::OwnedFd as ZbusFd;

const ID: &str = "com.github.codevardhan.caffeine-applet";
const ON: &str = "com.github.codevardhan.caffeine-applet.On";
const OFF: &str = "com.github.codevardhan.caffeine-applet.Off";
const CONFIG_VERSION: u64 = 1;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaffeineConfig {
    pub inhibit_lid: bool,
}

pub struct CaffeineApplet {
    core: Core,
    inhibit_fd: Option<OwnedFd>,
    config: CaffeineConfig,
}

impl Default for CaffeineApplet {
    fn default() -> Self {
        Self {
            core: Core::default(),
            inhibit_fd: None,
            config: CaffeineConfig::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    ToggleCaffeine,
}

fn build_what(inhibit_lid: bool) -> String {
    let mut what = String::from("idle:sleep");
    if inhibit_lid {
        what.push_str(":handle-lid-switch");
    }
    what
}

fn acquire_inhibit(inhibit_lid: bool) -> Result<OwnedFd, Box<dyn std::error::Error>> {
    let conn = Connection::system()?;
    let what = build_what(inhibit_lid);
    let reply: ZbusFd = conn
        .call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            "Inhibit",
            &(
                &*what,
                "Caffeine Applet",
                "Caffeine session active",
                "block",
            ),
        )?
        .body()
        .deserialize()?;

    Ok(reply.into())
}

fn load_or_create_config() -> CaffeineConfig {
    let context = match cosmic_config::Config::new(ID, CONFIG_VERSION) {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("Failed to open config: {err}");
            return CaffeineConfig::default();
        }
    };

    match context.get::<bool>("inhibit_lid") {
        Ok(inhibit_lid) => CaffeineConfig { inhibit_lid },
        Err(_) => {
            let config = CaffeineConfig::default();
            if let Err(err) = context.set("inhibit_lid", config.inhibit_lid) {
                eprintln!("Failed to write default config: {err}");
            }
            config
        }
    }
}

impl cosmic::Application for CaffeineApplet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(
        core: Core,
        _flags: Self::Flags,
    ) -> (Self, cosmic::Task<cosmic::Action<Self::Message>>) {
        let config = load_or_create_config();
        let window = CaffeineApplet {
            core,
            inhibit_fd: None,
            config,
        };
        (window, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> cosmic::Task<cosmic::Action<Self::Message>> {
        match message {
            Message::ToggleCaffeine => {
                if self.inhibit_fd.is_some() {
                    self.inhibit_fd = None;
                } else {
                    match acquire_inhibit(self.config.inhibit_lid) {
                        Ok(fd) => self.inhibit_fd = Some(fd),
                        Err(err) => eprintln!(
                            "Failed to acquire inhibit lock (is logind/elogind running?): {err}"
                        ),
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let icon = if self.inhibit_fd.is_some() { ON } else { OFF };
        self.core
            .applet
            .icon_button(icon)
            .on_press_down(Message::ToggleCaffeine)
            .into()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

// Tests live in tests/unit/window.rs.
// #[path] lets Rust load them from there while keeping access to the
// private functions above via `use super::*`.
#[cfg(test)]
#[path = "../tests/unit/window.rs"]
mod tests;

