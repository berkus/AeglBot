pub trait ExtendedCommand {
    fn prefix() -> &'static str;
    fn description() -> &'static str;
}
