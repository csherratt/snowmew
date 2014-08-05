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

#![feature(globs)]

extern crate snowmew  = "snowmew-core";
extern crate cow;

mod core {
    use snowmew::common::{CommonData, Common};

    #[test]
    fn db_new_object() {
        let mut db = CommonData::new();

        let id = db.new_object(None, "main");

        assert!(db.find("main").unwrap() == id);
    }
}