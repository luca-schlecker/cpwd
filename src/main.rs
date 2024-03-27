use std::{
    env::{self, current_dir},
    path::{Path, PathBuf, Prefix},
};

const HOME_ICON: &str = " ";
const GIT_ICON: &str = "󰊢 ";
const ROOT_ICON: &str = "";

fn main() {
    let current = current_dir().unwrap();
    let anchor = determine_anchor(&current);

    match anchor {
        Anchor::Home(rel) => println!("{}{}", HOME_ICON, shorten_relative(&rel, false)),
        Anchor::Git(repo, rel) => println!("{}{}{}", GIT_ICON, repo, shorten_relative(&rel, false)),
        Anchor::Root(disk, rel) => {
            println!("{}{}{}", disk, ROOT_ICON, shorten_relative(&rel, true))
        }
    }
}

enum Anchor {
    Home(PathBuf),
    Git(String, PathBuf),
    Root(String, PathBuf),
}

fn determine_anchor(p: &Path) -> Anchor {
    if let Some(git_dir) = get_git_dir(p) {
        Anchor::Git(
            git_dir.file_name().unwrap().to_str().unwrap().to_string(),
            p.strip_prefix(git_dir).unwrap().to_path_buf(),
        )
    } else {
        let home = if cfg!(target_os = "windows") {
            env::var("USERPROFILE").unwrap()
        } else {
            env::var("HOME").unwrap()
        };
        let home = Path::new(&home);
        if let Ok(rel) = p.strip_prefix(home) {
            Anchor::Home(rel.to_path_buf())
        } else {
            let (disk, prefix) = if cfg!(target_os = "windows") {
                match p.components().next().unwrap() {
                    std::path::Component::Prefix(prefix) => {
                        if let Prefix::Disk(d) = prefix.kind() {
                            (format!("{}:", d as char), format!("{}:/", d as char))
                        } else {
                            (String::new(), String::from(""))
                        }
                    }
                    _ => unreachable!(),
                }
            } else {
                (String::new(), String::from("/"))
            };

            Anchor::Root(disk, p.strip_prefix(prefix).unwrap().to_path_buf())
        }
    }
}

fn shorten_relative(p: &Path, slash_on_empty: bool) -> String {
    let component_count = p.components().count();

    if component_count == 0 {
        if slash_on_empty { "/" } else { "" }.to_owned()
    } else if component_count < 2 {
        format!("/{}", p.to_str().unwrap().to_owned())
    } else {
        format!("/../{}", p.file_name().unwrap().to_str().unwrap())
    }
}

fn get_git_dir(mut p: &Path) -> Option<PathBuf> {
    loop {
        if p.join(".git").exists() {
            break Some(p.to_path_buf());
        }
        if let Some(parent) = p.parent() {
            p = parent;
        } else {
            break None;
        }
    }
}
