use std::ffi::{OsStr};
use std::fs::File;
use std::path::Path;

use osmpbfreader::{OsmPbfReader,OsmObj,RelationId};

pub struct Reader {
    pbf: OsmPbfReader<File>,
}

impl Reader {
    // TODO: move out of this lib into example file
    pub fn stuff(&mut self) {
        fn wanted(obj: &OsmObj) -> bool {
            obj.id() == RelationId(7444).into() //id of relation for Paris
        }

        let objects = self.pbf.get_objs_and_deps(wanted).unwrap();
        // for _obj in pbf.iter().map(Result::unwrap) {
        println!(
            "The relation Paris is composed of {:?} items",
            objects.len()
        );
        for (id, _) in objects {
            println!("{:?}", id);
        }
    }
}

impl super::Read for Reader {
    fn from_path<S: AsRef<OsStr> + ?Sized>(path: &S) -> Reader {
        let path = Path::new(&path);
        let file = File::open(&path).expect("File exists");
        return Reader { pbf: OsmPbfReader::new(file) };
    }
}
