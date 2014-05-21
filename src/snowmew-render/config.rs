use std::os;

#[deriving(Eq)]
enum ConfigOption {
    Unsupported,
    Disabled,
    Enabled
}

impl ConfigOption {
    fn enabled(&self) -> bool {
        match *self {
            Enabled => true,
            _ => false
        }
    }
}

pub struct Config {
    max_size: uint,
    hmd_size: f32,
    bindless: ConfigOption,
    instanced: ConfigOption,
    profile: ConfigOption
}

fn get_setting_option(name: &str, default: ConfigOption) -> ConfigOption {
   let is_set = match os::getenv(name) {
        Some(s) => {
            let s: ~str = s.chars().map(|c| c.to_lowercase()).collect();
            match s.as_slice() {
                "true" => Some(true),
                "enabled" => Some(true),
                "1" => Some(true),
                "false" => Some(false),
                "disabled" => Some(false),
                "0" => Some(false),
                _ => None
            }
        }
        None => None
    };

    match is_set {
        Some(true) => Enabled,
        Some(false) => Disabled,
        None => default
    }
}

fn check_gl_version(gl_version: (uint, uint), min: (uint, uint), if_supported: ConfigOption) -> ConfigOption {
    let (m_major, m_minor) = min;
    let (gl_major, gl_minor) = gl_version;

    if gl_major > m_major {
        if_supported
    } else if gl_major == m_major && gl_minor >= m_minor {
        if_supported
    } else {
        Unsupported
    }
}

impl Config
{
    pub fn new(gl_version: (uint, uint)) -> Config {
        Config {
            profile: get_setting_option("PROFILE", Disabled),
            hmd_size: 1.7,
            max_size: 1024*1024,
            bindless: check_gl_version(gl_version, (4, 4),
                get_setting_option("BINDLESS", Enabled)
            ),
            instanced: check_gl_version(gl_version, (3, 1),
                get_setting_option("INSTANCED", Enabled)
            ),
        }
    }

    pub fn use_bindless(&self) -> bool { self.bindless.enabled() }

    pub fn max_size(&self) -> uint { self.max_size }

    pub fn hmd_size(&self) -> f32 { self.hmd_size }

    pub fn profile(&self) -> bool { self.profile == Enabled }

    pub fn instanced(&self) -> bool { self.instanced == Enabled }
}