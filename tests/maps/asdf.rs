pub mod parsing {
    use crate::helpers::{create_config, TestType};
    use osmgraphing::Parser;
    use std::path::PathBuf;

    #[test]
    pub fn wrong_extension() {
        let mut cfg = create_config(TestType::Small);
        cfg.graph.map_file = PathBuf::from("foo.asdf");
        assert!(
            Parser::parse(&cfg.graph).is_err(),
            "File-extension 'asdf' should not be supported."
        );
    }
}
