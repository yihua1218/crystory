use std::fs;
use std::io::{Read, Write};
use walkdir::WalkDir;
use sysinfo::Disks;
use uuid::Uuid;

#[cfg(target_os = "macos")]
use std::process::Command;
#[cfg(target_os = "macos")]
use serde::Deserialize;
#[cfg(target_os = "macos")]
use std::collections::HashMap;
#[cfg(target_os = "macos")]
use std::path::PathBuf;

#[cfg(target_os = "macos")]
#[derive(Deserialize, Debug)]
struct SPUSBDataType {
    #[serde(rename = "SPUSBDataType")]
    usb_devices: Vec<USBDevice>,
}

#[cfg(target_os = "macos")]
#[derive(Deserialize, Debug)]
struct USBDevice {
    _name: Option<String>, // Use _name to avoid warning if not used directly
    #[serde(rename = "_items")]
    items: Option<Vec<USBDeviceItem>>,
}

#[cfg(target_os = "macos")]
#[derive(Deserialize, Debug)]
struct USBDeviceItem {
    _name: Option<String>, // Use _name to avoid warning if not used directly
    #[serde(rename = "Media")]
    media: Option<Vec<MediaItem>>,
    #[serde(rename = "_items")] // Add this for recursive structure
    items: Option<Vec<USBDeviceItem>>,
}

#[cfg(target_os = "macos")]
#[allow(dead_code)] // Suppress warning for bsd_name not being read
#[derive(Deserialize, Debug)]
struct MediaItem {
    #[serde(rename = "bsd_name")]
    bsd_name: Option<String>,
    #[serde(rename = "volumes")]
    volumes: Option<Vec<VolumeItem>>,
}

#[cfg(target_os = "macos")]
#[derive(Deserialize, Debug)]
struct VolumeItem {
    #[serde(rename = "bsd_name")]
    bsd_name: Option<String>,
    #[serde(rename = "mount_point")]
    mount_point: Option<String>,
    #[serde(rename = "volume_uuid")]
    volume_uuid: Option<String>,
}

pub fn scan_directory(path: &str) {
    println!("Scanning directory: {}", path);
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Ok(metadata) = fs::metadata(path) {
            println!("--------------------------------------------------");
            println!("Path: {}", path.display());
            println!("  Is file: {}", metadata.is_file());
            println!("  Is directory: {}", metadata.is_dir());
            println!("  Size: {} bytes", metadata.len());
            if let Ok(created) = metadata.created() {
                println!("  Created: {:?}", created);
            }
            if let Ok(modified) = metadata.created() {
                println!("  Modified: {:?}", modified);
            }
        }
    }
}

// Helper function to recursively collect USB volume mount points and UUIDs
#[cfg(target_os = "macos")]
fn collect_usb_mount_points(usb_device_items: Option<Vec<USBDeviceItem>>, usb_mount_points_map: &mut HashMap<PathBuf, (String, Option<String>)>) {
    if let Some(items) = usb_device_items {
        for item in items {
            if let Some(media_items) = item.media {
                for media_item in media_items {
                    if let Some(volumes) = media_item.volumes {
                        for volume in volumes {
                            if let Some(mp_str) = volume.mount_point {
                                usb_mount_points_map.insert(PathBuf::from(mp_str), (volume.bsd_name.unwrap_or_default(), volume.volume_uuid));
                            }
                        }
                    }
                }
            }
            // Recursively call for nested items
            collect_usb_mount_points(item.items, usb_mount_points_map);
        }
    }
}

// Function to get or generate UUID for a given mount point
fn get_or_generate_uuid(mount_point: &PathBuf) -> Option<String> {
    let uuid_file_path = mount_point.join(".crystory_uuid");

    // Try to read UUID from file
    if uuid_file_path.exists() {
        match fs::File::open(&uuid_file_path) {
            Ok(mut file) => {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    let trimmed_contents = contents.trim().to_string();
                    if trimmed_contents != "00000000-0000-0000-0000-000000000000" {
                        return Some(trimmed_contents);
                    }
                }
            }
            Err(e) => eprintln!("Error opening UUID file {:?}: {}", uuid_file_path, e),
        }
    }

    // If file doesn't exist or couldn't be read, or if it contained a null UUID, generate a new UUID and write it
    let new_uuid = Uuid::new_v4().to_string();
    match fs::File::create(&uuid_file_path) {
        Ok(mut file) => {
            if file.write_all(new_uuid.as_bytes()).is_err() {
                eprintln!("Error writing UUID to file {:?}", uuid_file_path);
            }
        }
        Err(e) => eprintln!("Error creating UUID file {:?}: {}", uuid_file_path, e),
    }
    Some(new_uuid)
}

pub fn list_storage_devices() {
    let disks = Disks::new_with_refreshed_list();

    #[cfg(target_os = "macos")]
    let usb_mount_points_map = {
        let mut mount_points_map = HashMap::new();
        if let Ok(output) = Command::new("system_profiler")
            .arg("SPUSBDataType")
            .arg("-json")
            .output()
        {
            if output.status.success() {
                let json_str = String::from_utf8_lossy(&output.stdout);
                match serde_json::from_str::<SPUSBDataType>(&json_str) {
                    Ok(sp_data) => {
                        for usb_device in sp_data.usb_devices {
                            collect_usb_mount_points(usb_device.items, &mut mount_points_map);
                        }
                    },
                    Err(e) => {
                        eprintln!("Error deserializing system_profiler JSON: {}", e);
                    }
                }
            } else {
                eprintln!("Error: system_profiler command failed: {:?}", output.stderr);
            }
        }
        mount_points_map
    };

    println!("Listing all storage devices and partitions:");
    for disk in &disks {
        #[cfg(target_os = "macos")]
        {
            let is_usb_disk = usb_mount_points_map.contains_key(disk.mount_point());

            if !is_usb_disk {
                continue; // Skip if not a USB disk
            }
        }

        println!("--------------------------------------------------");
        println!("  Device: {:?}", disk.name());
        println!("  Type: {:?}", disk.kind());
        println!("  File system: {:?}", disk.file_system());
        println!("  Mount point: {:?}", disk.mount_point());
        println!("  Total space: {} bytes", disk.total_space());
        println!("  Available space: {} bytes", disk.available_space());
        
        let mut display_uuid: Option<String> = None;

        #[cfg(target_os = "macos")]
        {
            if let Some((_, uuid_option)) = usb_mount_points_map.get(disk.mount_point()) {
                if let Some(uuid) = uuid_option {
                    // If system_profiler provided a UUID, use it unless it's the null UUID
                    if uuid != "00000000-0000-0000-0000-000000000000" {
                        display_uuid = Some(uuid.to_uppercase());
                    } else {
                        // If it's the null UUID, try to get/generate one
                        if let Some(mount_point_path) = disk.mount_point().to_path_buf().into_os_string().into_string().ok() {
                            if let Some(uuid) = get_or_generate_uuid(&PathBuf::from(mount_point_path)) {
                                display_uuid = Some(uuid.to_uppercase());
                            }
                        }
                    }
                } else {
                    // If system_profiler didn't provide a UUID, try to get/generate one
                    if let Some(mount_point_path) = disk.mount_point().to_path_buf().into_os_string().into_string().ok() {
                        if let Some(uuid) = get_or_generate_uuid(&PathBuf::from(mount_point_path)) {
                            display_uuid = Some(uuid.to_uppercase());
                        }
                    }
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            // For non-macOS, sysinfo::Disk does not provide UUID directly
            // Try to get/generate one if a mount point exists
            if let Some(mount_point_path) = disk.mount_point().to_path_buf().into_os_string().into_string().ok() {
                if let Some(uuid) = get_or_generate_uuid(&PathBuf::from(mount_point_path)) {
                    display_uuid = Some(uuid.to_uppercase());
                }
            }
        }

        if let Some(uuid) = display_uuid {
            println!("  UUID: {}", uuid);
        } else {
            println!("  UUID: N/A");
        }
    }
}
