use es_fluent_build::FluentParseMode;

pub fn main() {
    if let Err(e) = es_fluent_build::FluentBuilder::new()
        .mode(FluentParseMode::Conservative)
        .build()
    {
        eprintln!("Error building FTL files: {}", e);
    }
}
