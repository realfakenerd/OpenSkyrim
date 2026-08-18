fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    engine::run(engine::config::EngineConfig::from_env())
}
