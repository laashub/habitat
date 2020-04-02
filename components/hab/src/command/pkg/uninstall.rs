use super::{ExecutionStrategy,
            Scope};
use crate::{command::pkg::uninstall_impl,
            error::Result};
use clap::ArgMatches;
use habitat_common::ui::UI;
use habitat_core::package::PackageIdent;
use std::path::Path;

#[derive(Clone, Copy)]
pub enum UninstallMode {
    Single,
    KeepLatest(usize),
}

impl<'a> From<&'a ArgMatches<'a>> for UninstallMode {
    fn from(m: &ArgMatches) -> Self {
        m.value_of("KEEP_LATEST")
         .and_then(|s| s.parse().ok())
         .map(Self::KeepLatest)
         .unwrap_or(Self::Single)
    }
}

pub async fn start(ui: &mut UI,
                   ident: &PackageIdent,
                   fs_root_path: &Path,
                   execution_strategy: ExecutionStrategy,
                   mode: UninstallMode,
                   scope: Scope,
                   excludes: &[PackageIdent])
                   -> Result<()> {
    match mode {
        UninstallMode::Single => {
            uninstall_impl::uninstall(ui,
                                      ident,
                                      fs_root_path,
                                      execution_strategy,
                                      scope,
                                      excludes,
                                      false).await
        }
        UninstallMode::KeepLatest(number_latest_to_keep) => {
            uninstall_impl::uninstall_all_but_latest(ui,
                                                     ident,
                                                     number_latest_to_keep,
                                                     fs_root_path,
                                                     execution_strategy,
                                                     scope,
                                                     excludes,
                                                     false).await?;
            Ok(())
        }
    }
}
