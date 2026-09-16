use crate::{CargoResult, Context, error};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use url::Url;

const CRATES_IO_INDEX: &str = "sparse+https://index.crates.io/";
const CRATES_IO_REGISTRY: &str = "crates-io";

#[derive(Debug, Default, Clone)]
struct Source {
    replace_with: Option<String>,

    registry: Option<String>,
}

pub fn latest_version(crate_name: &str, registry: &Url) -> CargoResult<semver::Version> {
    find_version(crate_name, &semver::VersionReq::STAR, registry)
}

pub fn compatible_version(
    crate_name: &str,
    version_req: &str,
    registry: &Url,
) -> CargoResult<semver::Version> {
    let version = semver::VersionReq::parse(version_req)?;
    find_version(crate_name, &version, registry)
}

fn find_version(
    crate_name: &str,
    version_req: &semver::VersionReq,
    registry: &Url,
) -> CargoResult<semver::Version> {
    let index = crates_index::SparseIndex::from_url(registry.as_str())?;

    let crate_ = match index.crate_from_cache(crate_name) {
        Ok(crate_) => crate_,
        Err(_) => {
            update_cache(&index, crate_name)?;
            index.crate_from_cache(crate_name)?
        }
    };

    crate_
        .versions()
        .iter()
        .filter(|version| !version.is_yanked())
        .filter_map(|version| {
            let version = version.version().parse::<semver::Version>().ok()?;

            if version_req.matches(&version) {
                Some(version)
            } else {
                None
            }
        })
        .max()
        .ok_or_else(|| error!("no compatible version of `{crate_name}` found for `{version_req}`"))
}

pub fn registry_url(manifest_path: &Path, registry: Option<&str>) -> CargoResult<Url> {
    let sources = load_sources(manifest_path)?;

    let mut source = match registry {
        None | Some(CRATES_IO_INDEX) => {
            let mut source = sources.get(CRATES_IO_REGISTRY).cloned().unwrap_or_default();

            source
                .registry
                .get_or_insert_with(|| CRATES_IO_INDEX.to_owned());

            source
        }

        Some(registry) => sources
            .get(registry)
            .cloned()
            .with_context(|| error!("The registry `{registry}` could not be found"))?,
    };

    while let Some(replace_with) = source.replace_with.clone() {
        let is_crates_io = replace_with == CRATES_IO_INDEX;

        source = sources
            .get(&replace_with)
            .cloned()
            .with_context(|| error!("The source `{replace_with}` could not be found"))?;

        if is_crates_io {
            source
                .registry
                .get_or_insert_with(|| CRATES_IO_INDEX.to_owned());
        }
    }

    let registry = source
        .registry
        .context("registry source has no index URL")?;

    Url::parse(&registry).context("Invalid cargo config")
}

fn load_sources(manifest_path: &Path) -> CargoResult<HashMap<String, Source>> {
    let mut sources = HashMap::new();

    let parent = manifest_path
        .parent()
        .context("manifest path has no parent directory")?;

    for directory in parent.ancestors() {
        read_config_if_exists(&mut sources, &directory.join(".cargo/config"))?;
        read_config_if_exists(&mut sources, &directory.join(".cargo/config.toml"))?;
    }

    let cargo_home = cargo_home()?;

    read_config_if_exists(&mut sources, &cargo_home.join("config"))?;

    read_config_if_exists(&mut sources, &cargo_home.join("config.toml"))?;

    Ok(sources)
}

fn read_config_if_exists(sources: &mut HashMap<String, Source>, path: &Path) -> CargoResult<()> {
    if !path.is_file() {
        return Ok(());
    }

    let content = std::fs::read_to_string(path)?;

    let config = content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|_| error!("Invalid cargo config"))?;

    if let Some(registries) = config.get("registries").and_then(|item| item.as_table()) {
        for (name, item) in registries {
            let Some(table) = item.as_table() else {
                continue;
            };

            let registry = table
                .get("index")
                .and_then(|item| item.as_str())
                .map(str::to_owned);

            sources.entry(name.to_owned()).or_insert(Source {
                registry,
                replace_with: None,
            });
        }
    }

    if let Some(source) = config.get("source").and_then(|item| item.as_table()) {
        for (name, item) in source {
            let Some(table) = item.as_table() else {
                continue;
            };

            let replace_with = table
                .get("replace-with")
                .and_then(|item| item.as_str())
                .map(str::to_owned);

            let registry = table
                .get("registry")
                .and_then(|item| item.as_str())
                .map(str::to_owned);

            sources.entry(name.to_owned()).or_insert(Source {
                registry,
                replace_with,
            });
        }
    }

    Ok(())
}

fn cargo_home() -> CargoResult<PathBuf> {
    if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        return Ok(PathBuf::from(cargo_home));
    }

    dirs_next::home_dir()
        .map(|path| path.join(".cargo"))
        .context("failed to determine Cargo home directory")
}

fn update_cache(index: &crates_index::SparseIndex, crate_name: &str) -> CargoResult<()> {
    let request = index
        .make_cache_request(crate_name)?
        .body(())?
        .map(|_| Vec::<u8>::new());

    let request = reqwest::blocking::Request::try_from(request)?;

    let response = reqwest::blocking::Client::new()
        .execute(request)
        .map_err(|error| error!("failed to fetch `{crate_name}`: {error}"))?;

    let status = response.status().as_u16();

    let body = response
        .bytes()
        .map_err(|error| error!("failed to read `{crate_name}` response: {error}"))?
        .to_vec();

    let response = crates_index::http::Response::builder()
        .status(status)
        .body(body)?;

    index.parse_cache_response(crate_name, response, true)?;

    Ok(())
}
