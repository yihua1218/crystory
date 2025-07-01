use serde::Deserialize;
use std::process::Command;
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct SystemProfilerReport {
    #[serde(rename = "_items")]
    items: Vec<USBItem>,
}

#[derive(Debug, Deserialize)]
struct USBItem {
    #[serde(rename = "_name")]
    name: String,
    
    #[serde(default)]
    serial_num: String,

    #[serde(rename = "Media")]
    #[serde(default)]
    media: Vec<MediaItem>,

    #[serde(rename = "_items")]
    #[serde(default)]
    sub_items: Vec<USBItem>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MediaItem {
    bsd_name: String,
    size_in_bytes: u64,
    #[serde(default)]
    volumes: Vec<Volume>,
}

#[derive(Debug, Deserialize)]
struct Volume {
    #[serde(rename = "_name")]
    name: String,
    file_system: String,
    volume_uuid: String,
    #[serde(default)]
    mount_point: String,
}

fn get_or_create_uuid(volume: &Volume) -> String {
    if volume.volume_uuid != "00000000-0000-0000-0000-000000000000" {
        return volume.volume_uuid.clone();
    }

    if volume.mount_point.is_empty() {
        return volume.volume_uuid.clone(); // Cannot create UUID file without a mount point
    }

    let uuid_file_path = Path::new(&volume.mount_point).join(".crystory_uuid");

    if let Ok(uuid_str) = fs::read_to_string(&uuid_file_path) {
        if let Ok(_) = Uuid::parse_str(&uuid_str) {
            return uuid_str;
        }
    }

    let new_uuid = Uuid::new_v4().to_string();
    if fs::write(&uuid_file_path, &new_uuid).is_ok() {
        // On macOS, we need to make the file hidden.
        Command::new("chflags")
            .arg("hidden")
            .arg(&uuid_file_path)
            .status()
            .ok();
        return new_uuid;
    }

    volume.volume_uuid.clone()
}

fn process_usb_item(item: &USBItem) {
    if !item.media.is_empty() {
        println!("Device: {}", item.name);
        if !item.serial_num.is_empty() {
            println!("  Serial Number: {}", item.serial_num);
        }

        for media in &item.media {
            println!("  Size: {} bytes", media.size_in_bytes);
            println!("  Partitions: {}", media.volumes.len());

            if media.volumes.len() == 1 {
                if let Some(volume) = media.volumes.first() {
                    let uuid = get_or_create_uuid(volume);
                    println!("  File System: {}", volume.file_system);
                    println!("  Volume UUID: {}", uuid);
                }
            } else {
                for (i, volume) in media.volumes.iter().enumerate() {
                    let uuid = get_or_create_uuid(volume);
                    println!("  - Partition {}:", i + 1);
                    println!("      Name: {}", volume.name);
                    println!("      File System: {}", volume.file_system);
                    println!("      Volume UUID: {}", uuid);
                }
            }
        }
        println!("--------------------------------------------------");
    }

    for sub_item in &item.sub_items {
        process_usb_item(sub_item);
    }
}

pub fn list_usb_devices() {
    let output = Command::new("system_profiler")
        .args(&["SPUSBDataType", "-xml"])
        .output()
        .expect("Failed to execute system_profiler");

    if !output.status.success() {
        eprintln!(
            "Error: system_profiler exited with status {}",
            output.status
        );
        return;
    }

    let result: Vec<SystemProfilerReport> =
        plist::from_bytes(&output.stdout).expect("Failed to parse plist XML");

    for report in result {
        for item in report.items {
            process_usb_item(&item);
        }
    }
}