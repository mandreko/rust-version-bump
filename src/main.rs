use std::env;
use std::fs;
use std::path::{Path};
use std::io::{self, Write};
use regex::Regex;
use semver::Version;
use xmltree::{Element, XMLNode};

fn main() -> io::Result<()> {
    let input_version = env::var("INPUT_VERSION")
        .expect("INPUT_VERSION environment variable not set");
    let file_path = env::var("INPUT_FILE_PATH")
        .expect("INPUT_FILE_PATH environment variable not set");

    let path = Path::new(&file_path);

    let file_name = path.file_name().unwrap();
    let file_extension = path
        .extension()
        //.and_then(OsStr::to_str);
        .unwrap()
        .to_str()
        .unwrap();


    match file_extension {
        ".xml" | ".props" | ".csproj" => update_xml(&input_version, path),
        ".json" => update_json(&input_version, path),
        ".plist" => update_plist(&input_version, path),
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
    todo!()
}
fn update_plist(version: &String, path: &Path) -> io::Result<()> {
    todo!()
}
fn update_json(version: &String, path: &Path) -> io::Result<()> {
    todo!()
}

fn update_xml(version: &String, path: &Path) -> io::Result<()> {
    // let mut file = File::open(path)?;
    // let mut contents = String::new();
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

fn is_valid_semver(version: &str) -> bool {
    match Version::parse(version) {
        Ok(_) => true,
        Err(_) => false,
    }
}

// fn update_package_json(version: &str, path: &Path) -> io::Result<()> {
//     let content = fs::read_to_string(path)?;
//     let mut json: Value = serde_json::from_str(&content)
//         .map_err(|e| Error::new(ErrorKind::InvalidData, format!("JSON parse error: {}", e)))?;
//
//     if let Some(obj) = json.as_object_mut() {
//         obj.insert("version".to_string(), json!(version));
//     } else {
//         return Err(Error::new(ErrorKind::InvalidData, "Invalid package.json structure"));
//     }
//
//     fs::write(path, serde_json::to_string_pretty(&json)?)
// }
//
// fn update_csproj(version: &str, path: &Path) -> io::Result<()> {
//     let content = fs::read_to_string(path)?;
//
//     // Using regex for XML updating as it's simpler than full XML parsing for this task
//     let version_regex = Regex::new(r"<(Version|AssemblyVersion|FileVersion)>(.*?)</(Version|AssemblyVersion|FileVersion)>")
//         .map_err(|e| Error::new(ErrorKind::Other, format!("Regex error: {}", e)))?;
//
//     let updated_content = version_regex.replace_all(&content, |caps: &regex::Captures| {
//         format!("<{}>{}</{}>", &caps[1], version, &caps[3])
//     });
//
//     if content == updated_content {
//         return Ok(());  // No changes were made
//     }
//
//     fs::write(path, updated_content.to_string())
// }
//
// fn update_plist(version: &str, path: &Path) -> io::Result<()> {
//     let content = fs::read_to_string(path)?;
//
//     // Using regex for plist updating
//     let bundle_version_regex = Regex::new(r"<key>CFBundleShortVersionString</key>\s*<string>(.*?)</string>")
//         .map_err(|e| Error::new(ErrorKind::Other, format!("Regex error: {}", e)))?;
//
//     let updated_content = bundle_version_regex.replace_all(&content, |_: &regex::Captures| {
//         format!("<key>CFBundleShortVersionString</key>\n\t<string>{}</string>", version)
//     });
//
//     if content == updated_content {
//         return Ok(());  // No changes were made
//     }
//
//     fs::write(path, updated_content.to_string())
// }
//
// fn update_manifest_json(version: &str, path: &Path) -> io::Result<()> {
//     let content = fs::read_to_string(path)?;
//     let mut json: Value = serde_json::from_str(&content)
//         .map_err(|e| Error::new(ErrorKind::InvalidData, format!("JSON parse error: {}", e)))?;
//
//     if let Some(obj) = json.as_object_mut() {
//         obj.insert("version".to_string(), json!(version));
//     } else {
//         return Err(Error::new(ErrorKind::InvalidData, "Invalid manifest.json structure"));
//     }
//
//     fs::write(path, serde_json::to_string_pretty(&json)?)
// }
//
// fn update_pubspec(version: &str, path: &Path) -> io::Result<()> {
//     let content = fs::read_to_string(path)?;
//
//     // Using regex for YAML updating
//     let version_regex = Regex::new(r"version:\s*(.*?)(\s|$)")
//         .map_err(|e| Error::new(ErrorKind::Other, format!("Regex error: {}", e)))?;
//
//     let updated_content = version_regex.replace_all(&content, |caps: &regex::Captures| {
//         format!("version: {}{}", version, &caps[2])
//     });
//
//     if content == updated_content {
//         return Ok(());  // No changes were made
//     }
//
//     fs::write(path, updated_content.to_string())
// }
//
// fn update_infoplist_strings(version: &str, path: &Path) -> io::Result<()> {
//     let content = fs::read_to_string(path)?;
//
//     // Using regex for InfoPlist.strings updating
//     let version_regex = Regex::new(r#""CFBundleShortVersionString"\s*=\s*"(.*?)";"#)
//         .map_err(|e| Error::new(ErrorKind::Other, format!("Regex error: {}", e)))?;
//
//     let updated_content = version_regex.replace_all(&content, |_: &regex::Captures| {
//         format!(r#""CFBundleShortVersionString" = "{}";"#, version)
//     });
//
//     if content == updated_content {
//         return Ok(());  // No changes were made
//     }
//
//     fs::write(path, updated_content.to_string())
// }
//
// fn update_cargo_toml(version: &str, path: &Path) -> io::Result<()> {
//     let content = fs::read_to_string(path)?;
//     let mut doc = content.parse::<Document>()
//         .map_err(|e| Error::new(ErrorKind::InvalidData, format!("TOML parse error: {}", e)))?;
//
//     if let Some(package) = doc.as_table_mut().get_mut("package") {
//         if let Some(table) = package.as_table_mut() {
//             table["version"] = value(version);
//         }
//     }
//
//     fs::write(path, doc.to_string())
// }
