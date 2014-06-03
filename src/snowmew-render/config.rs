use std::os;
use std::from_str::FromStr;

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
    drawlist_count: uint,
    thread_pool_size: uint,
    hmd_size: f32,
    bindless: ConfigOption,
    instanced: ConfigOption,
    profile: ConfigOption,
    opencl: ConfigOption,
    fps: ConfigOption
}

fn get_setting_option(name: &str, default: ConfigOption) -> ConfigOption {
   let is_set = match os::getenv(name) {
        Some(s) => {
            let s: String = s.as_slice().chars().map(|c| c.to_lowercase()).collect();
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

fn get_setting_from_str<T: FromStr>(name: &str, default: T) -> T {
    let str_value = match os::getenv(name) {
        Some(s) => s,
        None => return default
    };

    match FromStr::from_str(str_value.as_slice()) {
        Some(v) => v,
        None => default
    }
}

impl Config
{
    pub fn new(gl_version: (uint, uint)) -> Config {
        Config {
            hmd_size: get_setting_from_str("HMD_SIZE", 1.5f32),
            max_size: get_setting_from_str("MAX_OBJECTS", 64u*1024),
            drawlist_count: get_setting_from_str("DRAWLIST_COUNT", 3u),
            thread_pool_size: get_setting_from_str("THREAD_POOL_SIZE", 4u),
            opencl: get_setting_option("OPENCL", Enabled),
            profile: get_setting_option("PROFILE", Disabled),
            fps: get_setting_option("FPS", Enabled),
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
    pub fn profile(&self) -> bool { self.profile.enabled() }
    pub fn fps(&self) -> bool { self.profile.enabled() || self.fps.enabled() }
    pub fn instanced(&self) -> bool { self.instanced.enabled() }
    pub fn opencl(&self) -> bool { self.opencl.enabled() }
    pub fn drawlist_count(&self) -> uint { self.drawlist_count }
    pub fn thread_pool_size(&self) -> uint { self.thread_pool_size }
}