use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path};
use plist::Value as PlistValue;
use regex::Regex;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use xmltree::{Element, EmitterConfig};

fn main() -> io::Result<()> {
    let input_version = env::var("INPUT_VERSION")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "INPUT_VERSION environment variable not set"))?;

    let file_path = env::var("INPUT_FILE_PATH")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "INPUT_FILE_PATH environment variable not set"))?;

    let path = Path::new(&file_path);

    let file_name = path.file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Failed to extract file name from path"))?;

    let file_extension = path.extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Failed to extract file extension as string"))?;

    match file_extension {
        "xml" | "props" | "csproj" => update_xml(&input_version, path),
        "json" => update_json(&input_version, path),
        "plist" => update_plist(&input_version, path),
        _ if file_name == "Chart.yaml" || file_name == "Chart.yml" => update_yaml(&input_version, path),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "No file was recognized as a supported format.")),
    }?;

    if let Ok(github_output) = env::var("GITHUB_OUTPUT") {
        if let Ok(mut file) = fs::OpenOptions::new().append(true).open(&github_output) {
            writeln!(file, "status=Updated {}", file_path)?;
        }
    }

    Ok(())
}

fn update_yaml(version: &str, path: &Path) -> io::Result<()> {
    // Read the YAML file
    let contents = fs::read_to_string(path)?;

    // Parse the YAML file into a serde_yaml::Value
    let mut doc: YamlValue = serde_yaml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Update the version field
    if let Some(map) = doc.as_mapping_mut() {
        map.insert(YamlValue::String("version".to_string()), YamlValue::String(version.to_owned()));
    }

    // Write the updated YAML back to the file
    let new_contents = serde_yaml::to_string(&doc).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, new_contents)?;

    Ok(())
}
fn update_plist(version: &str, path: &Path) -> io::Result<()> {
    // Open the plist file for reading
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Parse the plist
    let mut plist_data: PlistValue = plist::from_bytes(&buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Update the version
    if let PlistValue::Dictionary(ref mut dict) = plist_data {
        dict.insert("CFBundleShortVersionString".to_string(), PlistValue::String(version.to_owned()));
    }

    // Write back the updated plist
    let file = File::create(path)?;
    plist::to_writer_xml(file, &plist_data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}
fn update_json(version: &str, path: &Path) -> io::Result<()> {
    // Read the file content
    let file_content = fs::read_to_string(path)?;

    // Parse JSON
    let mut data: JsonValue = serde_json::from_str(&file_content)?;

    // Update the version
    if let Some(obj) = data.as_object_mut() {
        obj.insert("version".to_string(), JsonValue::String(version.to_owned()));
        if let Some(packages) = obj.get_mut("packages").and_then(|p| p.as_object_mut()) {
            if let Some(empty_key) = packages.get_mut("").and_then(|p| p.as_object_mut()) {
                empty_key.insert("version".to_string(), JsonValue::String(version.to_owned()));
            }
        }
    }

    // Write the updated JSON back to file
    let mut file = fs::File::create(path)?;
    file.write_all(serde_json::to_string_pretty(&data)?.as_bytes())?;
    file.write_all(b"\n")?; // Append a newline

    Ok(())
}

fn update_xml(version: &str, path: &Path) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Try parsing XML
    if let Ok(mut root) = Element::parse(content.as_bytes()) {
        // Android Manifests
        if root.name == "manifest" {
            let re = Regex::new("android:versionName=\"[0-9]+\\.[0-9]+\\.[0-9]+\"").unwrap();
            let new_content = re.replace_all(&content, format!("android:versionName=\"{}\"", version));
            fs::write(path, new_content.as_ref())?;
        }
        // Microsoft .NET project files and MSBuild Props
        else if let Some(version_node) = root.get_mut_child("PropertyGroup")
            .and_then(|pg| pg.get_mut_child("Version")) {

            version_node.children.clear();
            version_node.children.push(xmltree::XMLNode::Text(version.to_string()));

            let mut buffer = Vec::new();
            let config = EmitterConfig::new().perform_indent(true).write_document_declaration(true);
            root.write_with_config(&mut buffer, config).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

            fs::write(path, buffer)?;
        }
    }
    Ok(())
}

