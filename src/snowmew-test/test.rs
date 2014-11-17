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

extern crate "snowmew-core" as snowmew;

mod db {
    use snowmew::common::Database;

    #[test]
    fn create_db() {
        let mut db = Database::new();

        let key = db.new_object(None, ~"main");

        assert!(db.get("main").unwrap() == key); 
    }

    #[test]
    fn scene_children() {
        let mut db = Database::new();

        let key = db.new_object(None, ~"main");

        let mut keys = ~[];

        for i in range(0, 100) {
            keys.push((i, db.new_object(Some(key), format!("obj_{}", i))));
        }

        for &(idx, key) in keys.iter() {
            assert!(db.get(format!("main/obj_{}", idx)).unwrap() == key);
        }
    }
}