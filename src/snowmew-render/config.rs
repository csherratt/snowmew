

enum ConfigOption
{
    Unsupported,
    Disabled,
    Enabled
}

impl ConfigOption
{
    fn enabled(&self) -> bool
    {
        match *self {
            Enabled => true,
            _ => false
        }
    }
}

pub struct Config
{
    priv bindless: ConfigOption
}

impl Config
{
    pub fn new(gl_version: (uint, uint)) -> Config
    {
        Config {
            bindless: match gl_version {
                (x, _) if x >= 5 => Enabled,
                (4, x) if x >= 4 => Enabled,
                (_, _) => Unsupported
            },
        }
    }

    pub fn use_bindless(&self) -> bool {self.bindless.enabled()}
}