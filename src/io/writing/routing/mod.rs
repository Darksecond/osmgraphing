use crate::{
    configs::writing,
    io::SupportingFileExts,
    network::Graph,
};
use log::info;

mod random;

trait Writing {
    fn write(&self, graph: &Graph, writing_cfg: &writing::routing::Config) -> Result<(), String>;
}

pub struct Writer;

impl Writer {
    pub fn write(graph: &Graph, writing_cfg: &writing::routing::Config) -> Result<(), String> {
        info!("START Write routes from graph");
        let result = match writing_cfg.category {
            writing::routing::Category::Random { seed, count } => {
                random::Writer::new(seed, count).write(graph, writing_cfg)
            }
        };
        info!("FINISHED");
        result
    }
}

impl SupportingFileExts for Writer {
    fn supported_exts<'a>() -> &'a [&'a str] {
        &["routes"]
    }
}
