use just_core::kernel::Folder;
use just_core::result::BoxedResult;
use std::env::consts::EXE_SUFFIX;
use structopt::StructOpt;
use url::Url;

const JUST_PREFIX: &str = "just-";

fn prepend_just_prefix(name: &str) -> String {
    if name.starts_with(JUST_PREFIX) {
        name.to_string()
    } else {
        let mut s = JUST_PREFIX.to_string();
        s.push_str(name);

        s
    }
}

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
    use is_executable::is_executable;
    use std::env::consts::EXE_SUFFIX;
    use walkdir::WalkDir;

    let suffix = EXE_SUFFIX;
    let prefix = JUST_PREFIX;

    let it = WalkDir::new(&folder.bin_path)
        .into_iter()
        .filter_map(|dir| dir.ok())
        .filter_map(|dir| {
            let path = dir.path();
            match path.file_name().and_then(|s| s.to_str()) {
                Some(filename) => {
                    if filename.ends_with(suffix)
                        && filename.starts_with(prefix)
                        && is_executable(path)
                    {
                        Some(filename.to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        });

    for filename in it {
        let end = filename.len() - suffix.len();
        println!(" - {}", &filename[prefix.len()..end]);
    }

    Ok(())
}

fn is_github_url(url: &Url) -> bool {
    url.host_str() == Some("github.com")
}

fn get_repository_name(url: &str) -> BoxedResult<String> {
    use just_core::result::BoxedErr;

    let url = Url::parse(url)?;

    if !is_github_url(&url) {
        BoxedErr::with("Currently, only github.com is supported for just components")
    } else if let Some(segments) = url.path_segments() {
        let vec: Vec<&str> = segments.skip(1).take(1).collect();
        if let Some(name) = vec.first() {
            Ok(name.to_string())
        } else {
            BoxedErr::with("No repository name in segments")
        }
    } else {
        BoxedErr::with("Invalid URL")
    }
}

fn add(url: &str, folder: &Folder) -> BoxedResult<()> {
    use duct::cmd;
    use log::debug;
    use remove_dir_all::remove_dir_all;
    use std::env::current_dir;
    use std::fs::copy;

    let repo = get_repository_name(url)?;
    let repo_path = current_dir().expect("Invalid current path").join(&repo);
    let cargo_path = repo_path.join("Cargo.toml");

    if repo_path.exists() {
        debug!("Remove existing {:?}", repo_path);
        remove_dir_all(&repo_path)?;
    }

    debug!("Clone {:?} from git", url);
    cmd("git", &["clone", &url]).run()?;
    debug!("Build {:?} with cargo", cargo_path);
    cmd(
        "cargo",
        &[
            "build",
            "--release",
            "--manifest-path",
            cargo_path.to_str().expect("No Cargo path"),
        ],
    )
    .run()?;

    let exe_name = format!("{}{}", repo, EXE_SUFFIX);
    let target_path = repo_path.join("target").join("release").join(&exe_name);
    let exe_name = prepend_just_prefix(&exe_name);
    let bin_path = folder.bin_path.join(exe_name);

    debug!("Copy {:?} into {:?}", target_path, bin_path);

    copy(&target_path, &bin_path)?;
    remove_dir_all(&repo_path).map_err(|e| e.into())
}

fn remove(name: &str, folder: &Folder) -> BoxedResult<()> {
    use std::fs::remove_file;

    let exe_name = format!("{}{}", name, EXE_SUFFIX);
    let exe_name = prepend_just_prefix(&exe_name);
    let bin_path = folder.bin_path.join(exe_name);

    remove_file(bin_path).map_err(|e| e.into())
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
