use std::{
    env,
    ffi::{CString, c_void},
    fs,
    path::{Path, PathBuf},
};

use windows::{
    Win32::Graphics::Direct3D::{
        Fxc::{D3DCOMPILE_ENABLE_STRICTNESS, D3DCOMPILE_OPTIMIZATION_LEVEL3, D3DCompile},
        ID3DBlob,
    },
    core::{PCSTR, s},
};

fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();

        res.set_icon("assets/meltick.ico");

        res.compile().expect("failed to compile Windows resources");

        compile_shaders();
    }
}

#[cfg(windows)]
fn compile_shaders() {
    println!("cargo:rerun-if-changed=shaders/fullscreen.hlsl");
    println!("cargo:rerun-if-changed=shaders/bg.hlsl");
    println!("cargo:rerun-if-changed=shaders/glass.hlsl");
    println!("cargo:rerun-if-changed=shaders/digits.hlsl");
    println!("cargo:rerun-if-changed=shaders/digit.hlsli");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    compile_shader(
        Path::new("shaders/fullscreen.hlsl"),
        s!("vs_5_0"),
        &out_dir.join("fullscreen_vs.cso"),
    );

    compile_shader(Path::new("shaders/bg.hlsl"), s!("ps_5_0"), &out_dir.join("bg_ps.cso"));

    compile_shader(Path::new("shaders/glass.hlsl"), s!("ps_5_0"), &out_dir.join("glass_ps.cso"));

    compile_shader(Path::new("shaders/digits.hlsl"), s!("ps_5_0"), &out_dir.join("digits_ps.cso"));

    compile_shader(Path::new("shaders/blit.hlsl"), s!("ps_5_0"), &out_dir.join("blit_ps.cso"));
}

#[cfg(windows)]
fn compile_shader(path: &Path, target: PCSTR, output: &Path) {
    let source = expand_includes(path);

    let source_name = CString::new(path.to_string_lossy().replace('\\', "/"))
        .expect("shader source name contains an interior NUL");

    let mut bytecode: Option<ID3DBlob> = None;
    let mut errors: Option<ID3DBlob> = None;

    unsafe {
        let result = D3DCompile(
            source.as_ptr() as *const c_void,
            source.len(),
            PCSTR(source_name.as_ptr() as *const u8),
            None,
            None,
            s!("main"),
            target,
            D3DCOMPILE_ENABLE_STRICTNESS | D3DCOMPILE_OPTIMIZATION_LEVEL3,
            0,
            &mut bytecode,
            Some(&mut errors),
        );

        if let Err(error) = result {
            let message = errors
                .as_ref()
                .map(|blob| {
                    let bytes = std::slice::from_raw_parts(
                        blob.GetBufferPointer() as *const u8,
                        blob.GetBufferSize(),
                    );

                    String::from_utf8_lossy(bytes).into_owned()
                })
                .unwrap_or_else(|| format!("{error:?}"));

            panic!("failed to compile {}:\n{}", path.display(), message);
        }

        let blob = bytecode.expect("D3DCompile returned no bytecode");

        let bytes =
            std::slice::from_raw_parts(blob.GetBufferPointer() as *const u8, blob.GetBufferSize());

        fs::write(output, bytes).unwrap_or_else(|error| {
            panic!("failed to write compiled shader {}: {error}", output.display())
        });
    }
}

#[cfg(windows)]
fn expand_includes(path: &Path) -> String {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read shader {}: {error}", path.display()));

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut output = String::new();

    for line in source.lines() {
        let trimmed = line.trim();

        if let Some(include) = parse_local_include(trimmed) {
            let include_path = parent.join(include);
            output.push_str(&expand_includes(&include_path));
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    output
}

#[cfg(windows)]
fn parse_local_include(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("#include")?.trim();

    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;

    Some(&rest[..end])
}
