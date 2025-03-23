use std::env;
use std::fs;
use std::fs::File;
use std::path::{Path};
use std::io::{self, Read, Write};
use regex::Regex;
use serde_yaml;
use xmltree::{Element, XMLNode};
use plist::Value as PlistValue;
use serde_json::Value as JsonValue;

fn main() -> io::Result<()> {
    let input_version = env::var("INPUT_VERSION")
        .expect("INPUT_VERSION environment variable not set");
    let file_path = env::var("INPUT_FILE_PATH")
        .expect("INPUT_FILE_PATH environment variable not set");

    let path = Path::new(&file_path);

    let file_name = path.file_name()
        .ok_or("Failed to extract file name from path")?;

    let file_extension = path.extension()
        .and_then(|ext| ext.to_str())
        .ok_or("Failed to extract file extension as string")?;

    match file_extension {
        "xml" | "props" | "csproj" => update_xml(&input_version, path),
        "json" => update_json(&input_version, path),
        "plist" => update_plist(&input_version, path),
        _ if file_name == "Chart.yaml" || file_name == "Chart.yml" => update_yaml(&input_version, path),
        //_ => return Err("No file was recognized as a supported format.".into()),

        _ => Ok({})
    }.expect("No file was recognized as a supported format.");

    if let Ok(github_output) = std::env::var("GITHUB_OUTPUT") {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .append(true)
            .open(&github_output)
        {
            writeln!(file, "status=Updated {}", file_path)
                .expect("Failed to write to GITHUB_OUTPUT file");
        }
    }

    Ok(())
}

fn update_yaml(version: &String, path: &Path) -> io::Result<()> {
    // Read the YAML file
    let contents = fs::read_to_string(path)?;

    // Parse the YAML file into a serde_yaml::Value
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Update the version field
    if let Some(map) = doc.as_mapping_mut() {
        map.insert(serde_yaml::Value::String("version".to_string()), serde_yaml::Value::String(version.clone()));
    }

    // Write the updated YAML back to the file
    let new_contents = serde_yaml::to_string(&doc).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, new_contents)?;

    Ok(())
}
fn update_plist(version: &String, path: &Path) -> io::Result<()> {
    // Open the plist file for reading
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Parse the plist
    let mut plist_data: PlistValue = plist::from_bytes(&buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Update the version
    if let PlistValue::Dictionary(ref mut dict) = plist_data {
        dict.insert("CFBundleShortVersionString".to_string(), PlistValue::String(version.clone()));
    }

    // Write back the updated plist
    let file = File::create(path)?;
    plist::to_writer_xml(file, &plist_data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}
fn update_json(version: &String, path: &Path) -> io::Result<()> {
    // Read the file content
    let file_content = fs::read_to_string(path)?;

    // Parse JSON
    let mut data: JsonValue = serde_json::from_str(&file_content)?;

    // Update the version
    if let Some(obj) = data.as_object_mut() {
        obj.insert("version".to_string(), JsonValue::String(version.clone()));
        if let Some(packages) = obj.get_mut("packages").and_then(|p| p.as_object_mut()) {
            if let Some(empty_key) = packages.get_mut("").and_then(|p| p.as_object_mut()) {
                empty_key.insert("version".to_string(), JsonValue::String(version.clone()));
            }
        }
    }

    // Write the updated JSON back to file
    let mut file = fs::File::create(path)?;
    file.write_all(serde_json::to_string_pretty(&data)?.as_bytes())?;
    file.write_all(b"\n")?; // Append a newline

    Ok(())
}

fn update_xml(version: &String, path: &Path) -> io::Result<()> {
    let mut file_content = fs::read_to_string(path)?;

    // Parse XML
    let mut root = Element::parse(file_content.as_bytes()).unwrap();

    // Android Manifest
    if root.name == "manifest" {
        let re = Regex::new("android:versionName=\"[0-9]+\\.[0-9]+\\.[0-9]+\"").unwrap();
        file_content = re.replace_all(&file_content, format!("android:versionName=\"{}\"", version)).to_string();
        fs::write(path, file_content)?;
    }
    // Microsoft .NET project files
        // TODO: Identify why it reformats the entire document
    else if let Some(sdk) = root.attributes.get("Sdk") {
        if sdk.contains("Microsoft.NET.Sdk") {
            if let Some(property_group) = root.get_mut_child("PropertyGroup") {
                for child in &mut property_group.children {
                    if let XMLNode::Element(element) = child {
                        if element.name == "Version" {
                            // xmltree can't set the value directly so we have to delete and re-create
                            element.children.clear();
                            element.children.push(XMLNode::Text(version.to_string()));
                            let mut output = Vec::new();
                            root.write(&mut output).unwrap();
                            fs::write(path, output)?;
                            break;
                        }
                    }
                }
            }
        }
    }
    // MSBuild Props
        // TODO: Figure out why it reformats the whole document
    else if let Some(property_group) = root.get_mut_child("PropertyGroup") {
        for child in &mut property_group.children {
            if let XMLNode::Element(element) = child {
                if element.name == "Version" {
                    // xmltree can't set the value directly so we have to delete and re-create
                    element.children.clear();
                    element.children.push(XMLNode::Text(version.to_string()));
                    let mut output = Vec::new();
                    root.write(&mut output).unwrap();
                    fs::write(path, output)?;
                    break;
                }
            }
        }
    }

    Ok(())
}

