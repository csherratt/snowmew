//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use std::default;
use serialize::Encodable;

#[deriving(Clone, Show, Encodable, Decodable)]
pub enum ConfigField {
    ConfigNone,
    ConfigBool(bool),
    ConfigInt(int),
    ConfigFloat(f64),
    ConfigString(String),
}

impl default::Default for ConfigField {
    fn default() -> ConfigField  { ConfigNone }
}

/// converts a standard rust type into a ConfigField
pub trait ToConfig {
    fn to_config_field(self) -> ConfigField;
}


impl ToConfig for () {
    fn to_config_field(self) -> ConfigField {
        ConfigNone
    }
}

impl ToConfig for bool {
    fn to_config_field(self) -> ConfigField {
        ConfigBool(self)
    }
}

impl ToConfig for int {
    fn to_config_field(self) -> ConfigField {
        ConfigInt(self)
    }
}

impl ToConfig for f64 {
    fn to_config_field(self) -> ConfigField {
        ConfigFloat(self)
    }
}

impl ToConfig for String {
    fn to_config_field(self) -> ConfigField {
        ConfigString(self)
    }
}

/// converts a ConfigField into a standard rust type
pub trait FromConfig {
    fn from_config_field(ConfigField) -> Option<Self>;
}

impl FromConfig for bool {
    fn from_config_field(c: ConfigField) -> Option<bool> {
        if let ConfigBool(boolean) = c {
            return Some(boolean);
        } else {
            return None;
        }
    }
}

impl FromConfig for int {
    fn from_config_field(c: ConfigField) -> Option<int> {
        if let ConfigInt(integer) = c {
            return Some(integer);
        } else {
            return None;
        }
    }
}

impl FromConfig for f64 {
    fn from_config_field(c: ConfigField) -> Option<f64> {
        if let ConfigFloat(float) = c {
            return Some(float);
        } else {
            return None;
        }
    }
}

impl FromConfig for String {
    fn from_config_field(c: ConfigField) -> Option<String> {
        if let ConfigString(string) = c {
            return Some(string);
        } else {
            return None;
        }
    }
}

