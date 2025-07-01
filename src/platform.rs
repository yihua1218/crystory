use serde::Deserialize;
use std::process::Command;

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
}

fn process_usb_item(item: &USBItem) {
    // Check if the current item is a storage device by looking for the "Media" key.
    if !item.media.is_empty() {
        println!("Device: {}", item.name);
        // Use the serial_num from the parent item if available.
        if !item.serial_num.is_empty() {
            println!("  Serial Number: {}", item.serial_num);
        }

        for media in &item.media {
            println!("  Size: {} bytes", media.size_in_bytes);
            println!("  Partitions: {}", media.volumes.len());

            if media.volumes.len() == 1 {
                if let Some(volume) = media.volumes.first() {
                    println!("  File System: {}", volume.file_system);
                    println!("  Volume UUID: {}", volume.volume_uuid);
                }
            } else {
                for (i, volume) in media.volumes.iter().enumerate() {
                    println!("  - Partition {}:", i + 1);
                    println!("      Name: {}", volume.name);
                    println!("      File System: {}", volume.file_system);
                    println!("      Volume UUID: {}", volume.volume_uuid);
                }
            }
        }
        println!("--------------------------------------------------");
    }

    // Recursively process any sub-items (for hubs or nested devices).
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
