/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::env;
use std::fs::{canonicalize, read_dir, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
#[cfg(all(target_os = "windows", feature="dx11"))]
use std::process::{self, Command, Stdio};


#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
const SHADER_VERSION: &'static str = "#version 150\n";

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
const SHADER_VERSION: &'static str = "#version 300 es\n";
const SHADER_IMPORT: &str = "#include ";

fn write_shaders(glsl_files: Vec<PathBuf>, shader_file_path: &Path) -> HashMap<String, String> {
    let mut shader_file = File::create(shader_file_path).unwrap();
    let mut shader_map: HashMap<String, String> = HashMap::with_capacity(glsl_files.len());

    write!(shader_file, "/// AUTO GENERATED BY build.rs\n\n").unwrap();
    write!(shader_file, "use std::collections::HashMap;\n").unwrap();
    write!(shader_file, "lazy_static! {{\n").unwrap();
    write!(
        shader_file,
        "  pub static ref SHADERS: HashMap<&'static str, &'static str> = {{\n"
    ).unwrap();
    write!(shader_file, "    let mut h = HashMap::new();\n").unwrap();
    for glsl in glsl_files {
        let shader_name = glsl.file_name().unwrap().to_str().unwrap();
        // strip .glsl
        let shader_name = shader_name.replace(".glsl", "");
        let full_path = canonicalize(&glsl).unwrap();
        let full_name = full_path.as_os_str().to_str().unwrap();
        // if someone is building on a network share, I'm sorry.
        let full_name = full_name.replace("\\\\?\\", "");
        let full_name = full_name.replace("\\", "/");
        shader_map.insert(shader_name.clone(), full_name.clone());
        write!(
            shader_file,
            "    h.insert(\"{}\", include_str!(\"{}\"));\n",
            shader_name,
            full_name
        ).unwrap();
    }
    write!(shader_file, "    h\n").unwrap();
    write!(shader_file, "  }};\n").unwrap();
    write!(shader_file, "}}\n").unwrap();
    shader_map
}

fn create_shaders(out_dir: String, shaders: &HashMap<String, String>) -> Vec<String> {
    fn get_shader_source(shader_name: &str, shaders: &HashMap<String, String>) -> Option<String> {
        if let Some(shader_file) = shaders.get(shader_name) {
            let shader_file_path = Path::new(shader_file);
            if let Ok(mut shader_source_file) = File::open(shader_file_path) {
                let mut source = String::new();
                shader_source_file.read_to_string(&mut source).unwrap();
                Some(source)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parse_shader_source(source: String, shaders: &HashMap<String, String>, output: &mut String) {
        for line in source.lines() {
            if line.starts_with(SHADER_IMPORT) {
                let imports = line[SHADER_IMPORT.len()..].split(",");
                // For each import, get the source, and recurse.
                for import in imports {
                    if let Some(include) = get_shader_source(import, shaders) {
                        parse_shader_source(include, shaders, output);
                    }
                }
            } else {
                output.push_str(line);
                output.push_str("\n");
            }
        }
    }

    let mut file_names = Vec::new();
    for (filename, _) in shaders {
        let is_vert = filename.ends_with(".vs");
        let is_frag = filename.ends_with(".fs");
        let is_prim = filename.starts_with("ps_");
        let is_cache = filename.starts_with("cs_");
        let is_debug = filename.starts_with("debug_");
        if (is_vert || is_frag) || !(is_prim || is_cache || is_debug) {
            continue;
        }
        let is_clip_cache = filename.starts_with("cs_clip");
        let is_ps_rect = filename.starts_with("ps_rectangle");
        let is_line = filename.starts_with("ps_line");
        let is_ps_text_run = filename.starts_with("ps_text_run");
        let is_ps_blend = filename.starts_with("ps_blend");
        let is_ps_hw_composite = filename.starts_with("ps_hardware_composite");
        let is_ps_composite = filename.starts_with("ps_composite");
        let is_ps_split_composite = filename.starts_with("ps_split_composite");
        let use_dither  = filename.starts_with("ps_gradient") ||
                          filename.starts_with("ps_angle_gradient") ||
                          filename.starts_with("ps_radial_gradient");
        let is_ps_yuv = filename.starts_with("ps_yuv");

        let base_filename = filename.splitn(2, '.').next().unwrap();
        let mut shader_prefix = if cfg!(target_os = "windows") && cfg!(feature = "dx11") {
            format!("// Base shader: {}\n#define WR_MAX_VERTEX_TEXTURE_WIDTH {}\n#define WR_DX11\n",
                    base_filename, 1024)
        } else {
            format!("{}\n// Base shader: {}\n#define WR_MAX_VERTEX_TEXTURE_WIDTH {}\n",
                    SHADER_VERSION, base_filename, 1024)
        };
        if is_clip_cache {
            shader_prefix.push_str("#define WR_CLIP_SHADER\n");
        }

        let mut build_configs = vec!["#define WR_FEATURE_TRANSFORM\n"];
        if is_prim {
            // the transform feature may be disabled for the prim shaders
            build_configs.push("// WR_FEATURE_TRANSFORM disabled\n");
        }


        if is_ps_rect {
            build_configs.push("#define WR_FEATURE_TRANSFORM\n#define WR_FEATURE_CLIP\n");
            build_configs.push("// WR_FEATURE_TRANSFORM disabled\n#define WR_FEATURE_CLIP\n");
        }

        if is_line {
            build_configs.push("#define WR_FEATURE_CACHE\n");
        }

        if is_ps_text_run {
            build_configs.push("#define WR_FEATURE_TRANSFORM\n#define WR_FEATURE_SUBPIXEL_AA\n");
            build_configs.push("// WR_FEATURE_TRANSFORM disabled\n#define WR_FEATURE_SUBPIXEL_AA\n");
        }

        if use_dither {
            build_configs.push("#define WR_FEATURE_TRANSFORM\n#define WR_FEATURE_DITHERING\n");
            build_configs.push("// WR_FEATURE_TRANSFORM disabled\n#define WR_FEATURE_DITHERING\n");
        }

        if is_ps_yuv {
            build_configs = vec!["// WR_FEATURE_TRANSFORM disabled\n#define WR_FEATURE_NV12\n"];
            build_configs.push("// WR_FEATURE_TRANSFORM disabled\n");
            build_configs.push("// WR_FEATURE_TRANSFORM disabled\n#define WR_FEATURE_INTERLEAVED_Y_CB_CR\n");
            build_configs.push("// WR_FEATURE_TRANSFORM disabled\n#define WR_FEATURE_NV12\n#define WR_FEATURE_YUV_REC709\n");
            build_configs.push("// WR_FEATURE_TRANSFORM disabled\n#define WR_FEATURE_YUV_REC709\n");
            build_configs.push("// WR_FEATURE_TRANSFORM disabled\n#define WR_FEATURE_INTERLEAVED_Y_CB_CR\n#define WR_FEATURE_YUV_REC709\n");
            build_configs.push("#define WR_FEATURE_TRANSFORM\n#define WR_FEATURE_NV12\n");
            build_configs.push("#define WR_FEATURE_TRANSFORM\n");
            build_configs.push("#define WR_FEATURE_TRANSFORM\n#define WR_FEATURE_INTERLEAVED_Y_CB_CR\n");
            build_configs.push("#define WR_FEATURE_TRANSFORM\n#define WR_FEATURE_NV12\n#define WR_FEATURE_YUV_REC709\n");
            build_configs.push("#define WR_FEATURE_TRANSFORM\n#define WR_FEATURE_YUV_REC709\n");
            build_configs.push("#define WR_FEATURE_TRANSFORM\n#define WR_FEATURE_INTERLEAVED_Y_CB_CR\n#define WR_FEATURE_YUV_REC709\n");
        }

        for (iter, config_prefix) in build_configs.iter().enumerate() {
            let (mut vs_source, mut fs_source) = (String::new(), String::new());
            vs_source.push_str(shader_prefix.as_str());
            vs_source.push_str("#define WR_VERTEX_SHADER\n");
            fs_source.push_str(shader_prefix.as_str());
            fs_source.push_str("#define WR_FRAGMENT_SHADER\n");
            vs_source.push_str(config_prefix);
            fs_source.push_str(config_prefix);
            let mut shared_result = String::new();
            if let Some(shared_source) = get_shader_source(base_filename, shaders) {
                parse_shader_source(shared_source, shaders, &mut shared_result);
            }
            vs_source.push_str(&shared_result);
            fs_source.push_str(&shared_result);
            let vs_name = format!("{}.vs", base_filename);
            if let Some(old_vs_source) = get_shader_source(&vs_name, shaders) {
                vs_source.push_str(&old_vs_source);
            }
            let fs_name = format!("{}.fs", base_filename);
            if let Some(old_fs_source) = get_shader_source(&fs_name, shaders) {
                fs_source.push_str(&old_fs_source);
            }
            let mut out_file_name = String::from(base_filename);
            if !is_ps_yuv {
            // The following cases are possible:
            // 0: Default, transfrom feature is enabled.
            //    Except for ps_blend, ps_hw_composite, ps_composite and ps_split_composite shaders.
            // 1: If the shader is prim shader, and the transform feature is disabled.
            //    This is the default case for ps_blend, ps_hw_composite, ps_composite and ps_split_composite shaders.
            // 2: If the shader is the `ps_rectangle`/`ps_text_run`/`gradient` shader
            //    and the `clip`/`subpixel AA`/`dither`, transfrom features are enabled.
            // 3: If the shader is the `ps_rectangle`/`ps_text_run`/`gradient` shader
            //    and the `clip`/`subpixel AA`/`dither` feature is enabled but the the transfrom feature is disabled.
                match iter {
                    0 => {
                        if is_prim && !(is_ps_blend || is_ps_hw_composite || is_ps_composite || is_ps_split_composite) {
                            out_file_name.push_str("_transform");
                        }
                    },
                    1 => {},
                    2 => {
                        if is_ps_rect {
                            out_file_name.push_str("_clip_transform");
                        }
                        if is_ps_text_run {
                            out_file_name.push_str("_subpixel_transform");
                        }
                        if use_dither {
                            out_file_name.push_str("_dither_transform");
                        }
                        if is_line {
                            out_file_name.push_str("_cache");
                        }
                    },
                    3 => {
                        if is_ps_rect {
                            out_file_name.push_str("_clip");
                        }
                        if is_ps_text_run {
                            out_file_name.push_str("_subpixel");
                        }
                        if use_dither {
                            out_file_name.push_str("_dither");
                        }
                    },
                    _ => unreachable!(),
                }
            } else {
                match iter {
                    0 => out_file_name.push_str("_nv12_601"),
                    1 => out_file_name.push_str("_planar_601"),
                    2 => out_file_name.push_str("_interleaved_601"),
                    3 => out_file_name.push_str("_nv12_709"),
                    4 => out_file_name.push_str("_planar_709"),
                    5 => out_file_name.push_str("_interleaved_709"),
                    6 => out_file_name.push_str("_nv12_601_transform"),
                    7 => out_file_name.push_str("_planar_601_transform"),
                    8 => out_file_name.push_str("_interleaved_601_transform"),
                    9 => out_file_name.push_str("_nv12_709_transform"),
                    10 => out_file_name.push_str("_planar_709_transform"),
                    11 => out_file_name.push_str("_interleaved_709_transform"),
                    _ => unreachable!(),
                }
            }
            let (mut vs_name, mut fs_name) = (out_file_name.clone(), out_file_name);
            vs_name.push_str(".vert");
            fs_name.push_str(".frag");
            let (vs_file_path, fs_file_path) = (Path::new(&out_dir).join(vs_name.clone()), Path::new(&out_dir).join(fs_name.clone()));
            let (mut vs_file, mut fs_file) = (File::create(vs_file_path).unwrap(), File::create(fs_file_path).unwrap());
            write!(vs_file, "{}", vs_source).unwrap();
            write!(fs_file, "{}", fs_source).unwrap();
            file_names.push(vs_name);
            file_names.push(fs_name);
        }
    }
    file_names
}

#[cfg(all(target_os = "windows", feature="dx11"))]
fn compile_fx_files(file_names: Vec<String>, out_dir: String) {
    for mut file_name in file_names {
        //TODO: Remove SUPPORTED_SHADERS when all shader conversion is done.
        if file_name.contains("ps_clear")
           || !(file_name.contains("ps_") || file_name.contains("cs_") || file_name.starts_with("debug_")) {
            continue;
        }
        let is_vert = file_name.ends_with(".vert");
        if !is_vert && !file_name.ends_with(".frag") {
            continue;
        }
        let file_path = Path::new(&out_dir).join(&file_name);
        file_name.push_str(".fx");
        let fx_file_path = Path::new(&out_dir).join(&file_name);
        let pf_path = env::var("ProgramFiles(x86)").ok().expect("Please set the ProgramFiles(x86) enviroment variable");
        let pf_path = Path::new(&pf_path);
        let format = if is_vert {
            "vs_5_0"
        } else {
            "ps_5_0"
        };
        let mut command = Command::new(pf_path.join("Windows Kits").join("8.1").join("bin").join("x64").join("fxc.exe").to_str().unwrap());
        command.arg("/Zi"); // Debug info
        command.arg("/T");
        command.arg(format);
        command.arg("/Fo");
        command.arg(&fx_file_path);
        command.arg(&file_path);
        println!("{:?}", command);
        if command.stdout(Stdio::inherit()).stderr(Stdio::inherit()).status().unwrap().code().unwrap() != 0
        {
            println!("Error while executing fxc");
            process::exit(1)
        }
    }
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

    let shader_map = write_shaders(glsl_files, &shaders_file);
    let _file_names = create_shaders(out_dir.clone(), &shader_map);
    #[cfg(all(target_os = "windows", feature = "dx11"))]
    compile_fx_files(_file_names, out_dir);
}
