#![allow(unused_imports)]
use std::{io::Write, path::Path};

use crate::dev::*;

const OUT_DIR: &str = "./out/";

#[test]
#[ignore = "manual Fallout 4 fixture test; requires local Deathclaw assets"]
pub fn dev_test() {
    use log::*;
    use simplelog::*;
    use std::fs::File;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("./nif.log").unwrap(),
        ),
    ])
    .unwrap();

    const PATH: &str = ".\\Meshes\\Actors\\Deathclaw\\Deathclaw.nif";
    const SKEL: &str = ".\\Meshes\\Actors\\Deathclaw\\CharacterAssets\\skeleton.nif";

    let mut file = NifFileV3::open(PATH).unwrap();

    debug!("{:#?}", file);

    for block in &file.raw_blocks {
        match block {
            NifBlock::BSShaderTextureSet(tset) => {
                debug!("{:#?}", tset);
            }
            _ => {}
        }
    }

    //debug!("{:#?}", file.header);

    let skel = NifFileV3::open(SKEL).unwrap();

    let (gltf, bin_data) = file.to_gltf("Deathclaw".to_string(), Some(&skel));

    let path = Path::new(OUT_DIR).join("deathclaw.gltf");
    let mut file = File::create(&path).unwrap();
    let mut bin = File::create(path.with_extension("bin")).unwrap();

    file.write_all(gltf.to_string_pretty().unwrap().as_bytes())
        .unwrap();
    bin.write_all(&bin_data).unwrap();
}

/*
//#[test]
pub fn test_all_in_archive() {
    use ba2::prelude::*;
    use simplelog::*;
    use std::fs::File;


    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("./nif.log").unwrap()),
        ]
    ).unwrap();

    if let Ok(mut ba2) = GeneralArchive::open("D:\\SteamLibrary\\steamapps\\common\\Fallout 4\\Data\\Fallout4 - Meshes.ba2") {

        let names = ba2.entries.clone();

        for (name, _entry) in names {
            if name.ends_with(".nif") {
                if let Ok(data) = &ba2.read_file(name.as_str()) {
                    if let Ok((_data, nif)) = NifFile::parse(&data) {

                        let full = OUT_DIR.to_string() + &name.replace(".nif", ".gltf");

                        let path = Path::new(&full);


                        info!("Writing \"{}\"", path.to_string_lossy().to_string());

                        std::fs::create_dir_all(path.parent().unwrap()).unwrap();

                        if let Ok(mut file) = std::fs::File::create(path.to_string_lossy().to_string()) {
                            if let Ok(model) = Model::try_from(nif) {
                                let (root, bin_data) = model.to_gltf(path.file_stem().unwrap().to_string_lossy().to_string());

                                let mut bin = std::fs::File::create(path.with_extension("bin")).unwrap();

                                file.write_all(root.to_string_pretty().unwrap().as_bytes()).unwrap();
                                bin.write_all(&bin_data).unwrap();
                            } else {
                                error!("Failed to convert \"{}\"", name);
                            }
                        } else {
                            error!("Failed to write \"{}\"", name);
                        }




                    } else {
                        error!("Failed to parse \"{}\"", name);
                    }
                } else {
                    debug!("Failed to read \"{}\"", name);
                }

            } else {
                debug!("Skipping \"{}\"", name);
            }
        }

    } else {
        panic!("Failed to open archive");
    }

}
*/
