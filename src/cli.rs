use clap::{clap_app, ArgMatches};

pub fn create_cli() -> ArgMatches<'static> {
    clap_app!(todo =>
        (@setting ArgRequiredElseHelp)
        (@setting DisableVersion)
        (@subcommand files =>
            (about: "Prints all files, filtered after the config")
        )
        (@subcommand list =>
            (about: "Lists all (un)reported")
            (@arg reported: -r --reported "Reported todos")
            (@arg unreported: -u --unreported "Unreported todos")
        )
    )
    .get_matches()
}
