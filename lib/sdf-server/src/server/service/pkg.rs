use crate::server::impl_default_error_into_response;
use axum::{
    response::Response,
    routing::{get, post},
    Json, Router,
};
use convert_case::{Case, Casing};
use dal::{
    installed_pkg::InstalledPkgError, pkg::PkgError as DalPkgError, DalContextBuilder,
    StandardModelError, TenancyError, TransactionsError, WsEventError,
};
use serde::{Deserialize, Serialize};
use si_pkg::{SiPkg, SiPkgError};
use si_settings::{safe_canonically_join, CanonicalFileError};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs::read_dir;

const PKG_EXTENSION: &str = "sipkg";
const MAX_NAME_SEARCH_ATTEMPTS: usize = 100;

pub mod export_pkg;
pub mod get_pkg;
pub mod install_pkg;
pub mod list_pkgs;

#[derive(Error, Debug)]
pub enum PkgError {
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("tenancy error: {0}")]
    Tenancy(#[from] TenancyError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("No packages path provided")]
    NoPackagesPath,
    #[error("Could not canononicalize path: {0}")]
    Canononicalize(#[from] CanonicalFileError),
    #[error("Package could not be found: {0}")]
    PackageNotFound(String),
    #[error("Package with that name already installed: {0}")]
    PackageAlreadyInstalled(String),
    // add error for matching hash
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    #[error(transparent)]
    SiPkg(#[from] SiPkgError),
    #[error(transparent)]
    DalPkg(#[from] DalPkgError),
    #[error("That package already exists: {0}")]
    PackageAlreadyOnDisk(String),
    #[error("Invalid pacakge file name: {0}")]
    InvalidPackageFileName(String),
    #[error("No schema variants added to package export")]
    PackageExportEmpty,
    #[error("Package name required")]
    PackageNameEmpty,
    #[error("Package version required")]
    PackageVersionEmpty,
}

pub type PkgResult<T> = Result<T, PkgError>;

impl_default_error_into_response!(PkgError);

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgView {
    name: String,
    installed: bool,
    hash: Option<String>,
}

pub async fn get_pkgs_path(builder: &DalContextBuilder) -> PkgResult<&PathBuf> {
    match builder.pkgs_path().await {
        None => Err(PkgError::NoPackagesPath),
        Some(path) => Ok(path),
    }
}

pub async fn list_pkg_dir_entries(pkgs_path: &Path) -> PkgResult<Vec<String>> {
    let mut result = vec![];
    let mut entries = read_dir(pkgs_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        result.push(entry.file_name().to_string_lossy().to_string())
    }

    Ok(result)
}

pub async fn pkg_lookup(
    pkgs_path: &Path,
    name: &str,
) -> PkgResult<(Option<PathBuf>, Option<String>)> {
    let real_pkg_path = safe_canonically_join(pkgs_path, name)?;
    let file_name = real_pkg_path
        .file_name()
        .map(|file_name| file_name.to_string_lossy().to_string());

    Ok((real_pkg_path.is_file().then_some(real_pkg_path), file_name))
}

fn add_pkg_extension(to: &str, version: &str, attempts: usize) -> String {
    match attempts {
        0 => format!("{}-{}.{}", to, version, PKG_EXTENSION),
        more_than_zero => format!("{}-{}-{}.{}", to, version, more_than_zero, PKG_EXTENSION),
    }
}

/// Generate a file name automatically based on the package name, appending numbers to the name if
/// it conflicts with a file already on disk.
pub async fn get_new_pkg_path(
    builder: &DalContextBuilder,
    name: &str,
    version: &str,
) -> PkgResult<PathBuf> {
    let name_kebabed = name.to_case(Case::Kebab);
    let version_kebabed = version.to_case(Case::Kebab);

    let mut attempts = 0;
    loop {
        let file_name = add_pkg_extension(&name_kebabed, &version_kebabed, attempts);

        let real_pkg_path = match Path::new(&file_name).file_name() {
            None => return Err(PkgError::InvalidPackageFileName(file_name)),
            Some(file_name) => Path::join(get_pkgs_path(builder).await?, file_name),
        };

        if attempts > MAX_NAME_SEARCH_ATTEMPTS {
            return Err(PkgError::PackageAlreadyOnDisk(
                real_pkg_path.to_string_lossy().to_string(),
            ));
        } else if real_pkg_path.is_file() {
            attempts += 1;
            continue;
        }

        return Ok(real_pkg_path);
    }
}

pub async fn pkg_open(builder: &DalContextBuilder, name: &str) -> PkgResult<SiPkg> {
    let pkg_tuple = pkg_lookup(get_pkgs_path(builder).await?, name).await?;

    let real_pkg_path = match pkg_tuple {
        (None, _) => return Err(PkgError::PackageNotFound(name.to_string())),
        (Some(real_pkg_path), _) => real_pkg_path,
    };

    Ok(SiPkg::load_from_file(&real_pkg_path).await?)
}

pub fn routes() -> Router {
    Router::new()
        .route("/export_pkg", post(export_pkg::export_pkg))
        .route("/get_pkg", get(get_pkg::get_pkg))
        .route("/install_pkg", post(install_pkg::install_pkg))
        .route("/list_pkgs", get(list_pkgs::list_pkgs))
}