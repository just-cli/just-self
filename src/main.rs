use just_core::kernel::Folder;
use just_core::result::BoxedResult;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "self", about = "...")]
enum JustSelf {
    #[structopt(name = "add")]
    Add { url: String },
    #[structopt(name = "remove")]
    Remove { name: String },
    #[structopt(name = "list")]
    List,
}

fn list(folder: &Folder) -> BoxedResult<()> {
    use just_extension::{Extension, JUST_PREFIX};
    use std::env::consts::EXE_SUFFIX;

    let prefix = JUST_PREFIX;
    let suffix = EXE_SUFFIX;

    for filename in Extension::new(folder).list() {
        let end = filename.len() - suffix.len();
        println!(" - {}", &filename[prefix.len()..end]);
    }

    Ok(())
}

fn add(url: &str, folder: &Folder) -> BoxedResult<()> {
    use just_extension::Extension;

    Extension::new(folder).install(url)
}

fn remove(name: &str, folder: &Folder) -> BoxedResult<()> {
    use just_extension::Extension;

    Extension::new(folder).uninstall(name)
}

fn main() -> BoxedResult<()> {
    use just_core::kernel::Kernel;

    let kernel = Kernel::load();

    let args: JustSelf = JustSelf::from_args();
    match args {
        JustSelf::Add { url } => add(&url, &kernel.path),
        JustSelf::Remove { name } => remove(&name, &kernel.path),
        JustSelf::List => list(&kernel.path),
    }
}
