use std::os;
use std::from_str::FromStr;

#[deriving(PartialEq)]
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
    ssbo: ConfigOption,
    compute: ConfigOption,
    instanced: ConfigOption,
    profile: ConfigOption,
    opencl: ConfigOption,
    fps: ConfigOption,
    culling: ConfigOption,
    chromatic: ConfigOption,
    vignette: ConfigOption,
    timewarp: ConfigOption
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
            hmd_size: get_setting_from_str("HMD_SIZE", 0.7f32),
            max_size: get_setting_from_str("MAX_OBJECTS", 64u*1024),
            drawlist_count: get_setting_from_str("DRAWLIST_COUNT", 3u),
            thread_pool_size: get_setting_from_str("THREAD_POOL_SIZE", 4u),
            opencl: get_setting_option("OPENCL", Enabled),
            profile: get_setting_option("PROFILE", Disabled),
            fps: get_setting_option("FPS", Enabled),
            compute: check_gl_version(gl_version, (4, 3), 
                get_setting_option("COMPUTE", Enabled)
            ),
            ssbo: check_gl_version(gl_version, (4, 3), 
                get_setting_option("SSBO", Enabled)
            ),
            instanced: get_setting_option("INSTANCED", Enabled),
            culling: get_setting_option("CULLING", Enabled),
            chromatic: get_setting_option("HMD_CHROMATRIC", Enabled),
            vignette: get_setting_option("HMD_VIGNETTE", Enabled),
            timewarp: get_setting_option("HMD_TIMEWARP", Enabled),
        }
    }

    pub fn compute(&self) -> bool { self.compute.enabled() }
    pub fn ssbo(&self) -> bool { self.ssbo.enabled() }
    pub fn max_size(&self) -> uint { self.max_size }
    pub fn hmd_size(&self) -> f32 { self.hmd_size }
    pub fn profile(&self) -> bool { self.profile.enabled() }
    pub fn fps(&self) -> bool { self.profile.enabled() || self.fps.enabled() }
    pub fn instanced(&self) -> bool { self.instanced.enabled() }
    pub fn opencl(&self) -> bool { self.opencl.enabled() }
    pub fn drawlist_count(&self) -> uint { self.drawlist_count }
    pub fn thread_pool_size(&self) -> uint { self.thread_pool_size }
    pub fn culling(&self) -> bool { self.culling.enabled() }
    pub fn chromatic(&self) -> bool { self.chromatic.enabled() }
    pub fn vignette(&self) -> bool { self.vignette.enabled() }
    pub fn timewarp(&self) -> bool { self.timewarp.enabled() }
}