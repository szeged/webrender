/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate cfg_if;
extern crate webrender_build;

cfg_if! {
    if #[cfg(not(feature = "gleam"))] {
        extern crate ron;
        #[macro_use]
        extern crate serde;
        extern crate gfx_hal;
        #[path = "src/device/gfx/vertex_types.rs"]
        mod vertex_types;
        mod build_gfx;
        use build_gfx::gfx_main;
    }
}

use std::collections::HashMap;
use std::borrow::Cow;
use std::env;
use std::fs::{canonicalize, read_dir, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use webrender_build::shader::*;

fn write_shaders(glsl_files: Vec<PathBuf>, shader_file_path: &Path) -> HashMap<String, String> {
    let mut shader_file = File::create(shader_file_path).unwrap();
    let mut shader_map: HashMap<String, String> = HashMap::with_capacity(glsl_files.len());

    write!(shader_file, "/// AUTO GENERATED BY build.rs\n\n").unwrap();
    write!(shader_file, "use std::collections::HashMap;\n\n").unwrap();
    write!(shader_file, "pub struct SourceWithDigest {{ pub source: &'static str, pub digest: &'static str }}\n\n")
        .unwrap();
    write!(shader_file, "lazy_static! {{\n").unwrap();
    write!(
        shader_file,
        "  pub static ref SHADERS: HashMap<&'static str, SourceWithDigest> = {{\n"
    ).unwrap();
    write!(shader_file, "    let mut h = HashMap::new();\n").unwrap();
    for glsl in glsl_files {
        // Compute the shader name.
        assert!(glsl.is_file());
        let shader_name = glsl.file_name().unwrap().to_str().unwrap();
        let shader_name = shader_name.replace(".glsl", "");

        // Compute a digest of the #include-expanded shader source. We store
        // this as a literal alongside the source string so that we don't need
        // to hash large strings at runtime.
        let mut hasher = Sha256::new();
        let base = glsl.parent().unwrap();
        assert!(base.is_dir());
        parse_shader_source(
            Cow::Owned(shader_source_from_file(&glsl)),
            &|f| Cow::Owned(shader_source_from_file(&base.join(&format!("{}.glsl", f)))),
            &mut |s| hasher.input(s.as_bytes()),
        );
        let digest: ProgramSourceDigest = hasher.into();

        // Compute the shader path for insertion into the include_str!() macro.
        // This makes for more compact generated code than inserting the literal
        // shader source into the generated file.
        //
        // If someone is building on a network share, I'm sorry.
        let full_path = canonicalize(&glsl).unwrap();
        let full_name = full_path.as_os_str().to_str().unwrap();
        let full_name = full_name.replace("\\\\?\\", "");
        let full_name = full_name.replace("\\", "/");
        shader_map.insert(shader_name.clone(), full_name.clone());

        write!(
            shader_file,
            "    h.insert(\"{}\", SourceWithDigest {{ source: include_str!(\"{}\"), digest: \"{}\"}});\n",
            shader_name,
            full_name,
            digest,
        ).unwrap();
    }
    write!(shader_file, "    h\n").unwrap();
    write!(shader_file, "  }};\n").unwrap();
    write!(shader_file, "}}\n").unwrap();
    shader_map
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap_or("out".to_owned());

    let shaders_file = Path::new(&out_dir).join("shaders.rs");
    let mut glsl_files = vec![];

    println!("cargo:rerun-if-changed=res");
    let res_dir = Path::new("res");
    for entry in read_dir(res_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if entry.file_name().to_str().unwrap().ends_with(".glsl") {
            println!("cargo:rerun-if-changed={}", path.display());
            glsl_files.push(path.to_owned());
        }
    }

    // Sort the file list so that the shaders.rs file is filled
    // deterministically.
    glsl_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let _shaders = write_shaders(glsl_files, &shaders_file);
    #[cfg(not(feature = "gleam"))]
    gfx_main(&out_dir, _shaders, &shaders_file)
}
