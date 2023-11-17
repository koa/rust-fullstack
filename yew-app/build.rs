use std::fs;

use anyhow::Result;

use backend_impl::{create_anonymous_schema, create_schema};

fn main() -> Result<()> {
    write_graphql_schema()?;
    write_anonymous_graphql_schema()?;
    Ok(())
}

fn write_graphql_schema() -> Result<()> {
    let schema = create_schema();
    fs::write("graphql/schema.graphql", schema.sdl())?;
    Ok(())
}
fn write_anonymous_graphql_schema() -> Result<()> {
    let schema = create_anonymous_schema();
    fs::write("graphql/anonymous_schema.graphql", schema.sdl())?;
    Ok(())
}
