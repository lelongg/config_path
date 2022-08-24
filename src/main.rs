use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    borrow::Cow,
    collections::HashMap,
    env::args,
    fs::read_to_string,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
struct PathsConfig {
    #[serde(flatten)]
    paths: HashMap<String, PathBuf>,
}

fn main() -> Result<()> {
    let paths_str = read_to_string(args().nth(1).context("no paths config provided")?)
        .context("cannot open path config file")?;
    let paths_config: PathsConfig =
        serde_json::from_str(&paths_str).context("cannot parse JSON file")?;
    if let Some(path_name) = args().nth(2) {
        let raw_path = paths_config
            .paths
            .get(&path_name)
            .context(format!("path `{}` is undefined", path_name))?;
        display_path(raw_path).context(format!(
            "cannot display path: `{}`",
            raw_path.to_string_lossy()
        ))?;
    } else {
        let mut radioactive_paths = Vec::new();
        for error in paths_config
            .paths
            .iter()
            .map(|(path_name, raw_path)| {
                let expanded_path = expand_path(raw_path)?;
                if is_radioactive(&expanded_path) {
                    radioactive_paths.push((
                        path_name.clone(),
                        raw_path.clone(),
                        expanded_path.clone(),
                    ));
                }
                display_path_entry(path_name, raw_path, expanded_path)
                    .context(format!("failed to display path with name: `{}`", path_name))
            })
            .filter_map(|result| result.err())
        {
            eprintln!("\n{:?}", error);
        }
        if !radioactive_paths.is_empty() {
            eprintln!("\n☢ RADIOACTIVE PATHS DETECTED ☢");
            for (path_name, raw_path, expanded_path) in radioactive_paths {
                display_path_entry(&path_name, &raw_path, expanded_path)?;
            }
        }
    }
    Ok(())
}

fn display_path(raw_path: &impl AsRef<Path>) -> Result<()> {
    println!("{}", expand_path(raw_path)?);
    Ok(())
}

fn display_path_entry(
    path_name: &impl AsRef<str>,
    raw_path: &impl AsRef<Path>,
    expanded_path: Cow<str>,
) -> Result<()> {
    if matches!(expanded_path, Cow::Borrowed(_)) {
        println!("{}: `{}`", path_name.as_ref(), expanded_path);
    } else {
        println!(
            "{}: `{}` ➟ `{}`",
            path_name.as_ref(),
            raw_path.as_ref().display(),
            expanded_path
        );
    }
    Ok(())
}

fn expand_path(raw_path: &impl AsRef<Path>) -> Result<Cow<str>> {
    let path = raw_path.as_ref().to_str().context("path is not UTF-8")?;
    let expanded_path = shellexpand::full(path).context("cannot expand path")?;
    Ok(expanded_path)
}

fn is_radioactive(path: &impl AsRef<str>) -> bool {
    path.as_ref()
        .split(':')
        .any(|sub_path| !sub_path.starts_with("/nix/store/"))
}
