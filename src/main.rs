#[macro_use]
extern crate clap;

use clap::{crate_name, crate_version, App, AppSettings};

fn build_app() -> App<'static, 'static> {
    let app = clap_app!(app =>
        (name: crate_name!())
        (version: crate_version!())
        (about: "A tool stitches scripts and commands together by YAML.")
        (max_term_width: 100)
        (global_setting: AppSettings::ColoredHelp)
        (global_setting: AppSettings::UnifiedHelpMessage)
        (global_setting: AppSettings::HidePossibleValuesInHelp)
        (setting: AppSettings::ArgsNegateSubcommands)
        (setting: AppSettings::AllowExternalSubcommands)
        (setting: AppSettings::DisableHelpSubcommand)
        (setting: AppSettings::VersionlessSubcommands)
    );

    app
}

fn main() {
    build_app().get_matches();
}
