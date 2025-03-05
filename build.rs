use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Basisverzeichnis mit den GLSL-Shadern (inkl. Unterverzeichnisse)
const SHADER_DIR: &str = "C:/Dev/vudeljump/shaders";

/// Vulkan-Shader-Compiler (glslangValidator)
const VULKAN_COMPILER: &str = "C:/VulkanSDK/1.3.268.0/Bin/glslc.exe";

fn main() -> Result<(), Error> {

    println!("cargo:rerun-if-changed={}", SHADER_DIR);

    // Alle Shader-Dateien (inkl. Unterordner) finden
    let shader_files = get_shader_files(SHADER_DIR)?;

    for shader_path in shader_files {
        // Prüfen, ob die Datei neu kompiliert werden muss
        if should_compile(&shader_path)? {
            compile_shader(&shader_path)?;
        }
    }

    Ok(())
}

/// Rekursive Suche nach Shader-Dateien (.frag, .vert, .comp) in einem Verzeichnis.
fn get_shader_files(dir: &str) -> Result<Vec<PathBuf>, Error> {
    let mut shader_files = Vec::new();

    // Verzeichnis rekursiv durchlaufen
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Rekursive Suche in Unterverzeichnissen
            shader_files.extend(get_shader_files(path.to_str().unwrap())?);
        } else if let Some(extension) = path.extension() {
            // Nur Shader mit den Endungen .frag, .vert, .comp
            if extension == "frag" || extension == "vert" || extension == "comp" {
                shader_files.push(path);
            }
        }
    }

    Ok(shader_files)
}

/// Prüft, ob der Shader neu kompiliert werden muss (basierend auf dem Änderungsdatum).
fn should_compile(shader_path: &Path) -> Result<bool, Error> {
    let compiled_shader_path = get_spirv_output_path(shader_path);

    // Wenn die .spv-Datei nicht existiert, muss der Shader kompiliert werden
    if !compiled_shader_path.exists() {
        return Ok(true);
    }

    // Änderungsdatum des GLSL- und SPIR-V-Shaders prüfen
    let shader_modified = fs::metadata(shader_path)?.modified()?;
    let compiled_modified = fs::metadata(compiled_shader_path)?.modified()?;

    Ok(shader_modified > compiled_modified)
}

/// Gibt den Pfad der SPIR-V-Ausgabe für den gegebenen Shader zurück (z.B. shader.frag -> frag.spv).
fn get_spirv_output_path(shader_path: &Path) -> PathBuf {
    let extension = shader_path.extension().unwrap().to_str().unwrap();
    let output_file_name = format!("{}.spv", extension);
    shader_path.with_file_name(output_file_name)
}

/// Kompiliert den GLSL-Shader mit dem Vulkan-Compiler.
fn compile_shader(shader_path: &Path) -> Result<(), Error> {
    let output_path = get_spirv_output_path(shader_path);

    // Vulkan-Compiler (glslangValidator) ausführen
    let status = Command::new(VULKAN_COMPILER)
        .arg(shader_path) // Pfad zur Shader-Datei
        .arg("-o") // Ausgabeparameter
        .arg(&output_path) // Pfad zur Ausgabedatei
        .status()?;

    if status.success() {
        println!(
            "Successfully compiled shader: {} -> {}",
            shader_path.display(),
            output_path.display()
        );
    } else {
        eprintln!("Failed to compile shader: {}", shader_path.display());
    }

    Ok(())
}