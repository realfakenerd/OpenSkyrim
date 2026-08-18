use color_eyre::{Result, eyre::WrapErr};
use engine::world::cache::CellCache;
use rusqlite::{Connection, OptionalExtension, params};
use std::{env, path::PathBuf};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut args = env::args_os().skip(1);
    let assets = PathBuf::from(args.next().ok_or_else(|| {
        color_eyre::eyre::eyre!("usage: world-inspect <assets> <worldspace> <grid-x> <grid-y>")
    })?);
    let worldspace = parse_number(args.next(), "worldspace")?;
    let grid_x = parse_i32(args.next(), "grid-x")?;
    let grid_y = parse_i32(args.next(), "grid-y")?;
    let connection = Connection::open(assets.join("skyrim_world.db"))?;
    let cell_id = connection
        .query_row(
            "SELECT id FROM cells WHERE worldspace_id=?1 AND grid_x=?2 AND grid_y=?3",
            params![worldspace, grid_x, grid_y],
            |row| row.get::<_, u32>(0),
        )
        .optional()?
        .ok_or_else(|| color_eyre::eyre::eyre!("cell was not found"))?;
    let (reference_count, min_z, max_z): (u64, Option<f64>, Option<f64>) = connection.query_row(
        "SELECT COUNT(*),MIN(pos_z),MAX(pos_z) FROM \"references\" WHERE cell_id=?1",
        [cell_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let cache = CellCache::open(&assets.join("cell_cache.rkyv"))?;
    let terrain = cache
        .terrain(cell_id)
        .ok_or_else(|| color_eyre::eyre::eyre!("cell has no terrain cache entry"))?;
    let min_height = terrain.heights.iter().copied().reduce(f32::min);
    let max_height = terrain.heights.iter().copied().reduce(f32::max);
    let center = usize::from(terrain.height / 2) * usize::from(terrain.width)
        + usize::from(terrain.width / 2);
    println!(
        "cell={cell_id:08X} references={reference_count} reference_z={min_z:?}..{max_z:?} terrain={}x{} heights={min_height:?}..{max_height:?} center={:?} water={:?}",
        terrain.width,
        terrain.height,
        terrain.heights.get(center),
        terrain.water_height
    );
    let mut statement = connection.prepare(
        "SELECT model_path,bounds_min_x,bounds_min_y,bounds_min_z,bounds_max_x,bounds_max_y,bounds_max_z
         FROM statics WHERE bounds_valid=1
         ORDER BY MAX(ABS(bounds_min_x),ABS(bounds_min_y),ABS(bounds_min_z),ABS(bounds_max_x),ABS(bounds_max_y),ABS(bounds_max_z)) DESC
         LIMIT 10",
    )?;
    let suspicious = statement.query_map([], |row| {
        Ok((
            row.get::<_, Option<String>>(0)?,
            [row.get::<_, f32>(1)?, row.get(2)?, row.get(3)?],
            [row.get::<_, f32>(4)?, row.get(5)?, row.get(6)?],
        ))
    })?;
    for entry in suspicious {
        let (path, min, max) = entry?;
        println!("bounds path={path:?} min={min:?} max={max:?}");
    }
    Ok(())
}

fn parse_number(value: Option<std::ffi::OsString>, name: &str) -> Result<u32> {
    let value = value
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| color_eyre::eyre::eyre!("missing {name}"))?;
    value
        .strip_prefix("0x")
        .map_or_else(|| value.parse(), |value| u32::from_str_radix(value, 16))
        .wrap_err_with(|| format!("invalid {name}"))
}

fn parse_i32(value: Option<std::ffi::OsString>, name: &str) -> Result<i32> {
    value
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| color_eyre::eyre::eyre!("missing {name}"))?
        .parse()
        .wrap_err_with(|| format!("invalid {name}"))
}
