# Crystory

"Crystory" (Crystal + History) is a cross-platform USB/external drive indexing tool that builds a persistent and searchable record of all files on attached storage devices. Even after a device is removed, its mirrored data remains queryable locally or in the cloud.

## 🌟 Project Goals
- Identify USB and external drives by unique ID (UUID, serial, etc.)
- Cache full file/directory structure into local SQLite databases
- Allow fast query and lookup by device or file info
- Run as a background service to auto-scan newly attached storage
- Optionally sync device cache databases to the cloud
- CLI-first design, scriptable and extensible

## 🚀 Key Features
- Auto-detect new storage devices across OS platforms
- Scan and cache file metadata into per-device SQLite DBs
- Query file info without device being attached
- Track device metadata (label, UUID, mount path, etc.)
- Future: rclone-based cloud sync support (Dropbox, GDrive, S3)

## 📦 Example CLI Commands
```bash
# List all known indexed devices
crystory list-devices

# Scan a new device manually
crystory scan --mount /Volumes/USB_DRIVE

# Query a previously indexed device by UUID
crystory query --uuid 1234-5678 --filter ".pdf"

# Start or stop the background indexer service
crystory service start
crystory service stop
```

## 🛠️ To-do List (Prioritized)

### ✅ Phase 1 - Core Functionality
1. [ ] Device UUID and metadata detection (cross-platform)
2. [ ] Directory traversal and file metadata extraction
3. [ ] SQLite schema design for device + file cache
4. [ ] Basic CLI subcommands (`scan`, `list-devices`, `query`)
5. [ ] Local cache storage and lookup by UUID

### 🚧 Phase 2 - Platform Integration
6. [ ] Background service daemon for macOS / Linux / Windows
7. [ ] Auto-trigger scan on new device insertion
8. [ ] Logging and error handling

### 🌐 Phase 3 - Cloud and Extensibility
9. [ ] rclone or API-based sync to S3/Dropbox/GDrive
10. [ ] Web UI or TUI for browsing device contents
11. [ ] File hash support for duplication detection
12. [ ] Export/import index (for backup and sharing)

---

Built with ❤️ in Rust. Inspired by the clarity of crystals and the persistence of memory.
