/// Synthetic Steam metadata: no user/account fields or real app data.
pub fn appinfo(version: u8, entries: &[(u32, &str)]) -> Vec<u8> {
    appinfo_with_icon(version, entries, None)
}

pub fn appinfo_with_icon(version: u8, entries: &[(u32, &str)], icon_hash: Option<&str>) -> Vec<u8> {
    let mut data = vec![version, 0x44, 0x56, 0x07, 1, 0, 0, 0];
    if version == 0x29 {
        data.extend([0; 8]);
    }
    for (id, kind) in entries {
        let mut payload = Vec::new();
        for (index, key, tag) in [(0u32, "appinfo", 0), (1, "common", 0), (2, "type", 1)] {
            payload.push(tag);
            if version == 0x29 {
                payload.extend(index.to_le_bytes());
            } else {
                payload.extend(key.as_bytes());
                payload.push(0);
            }
        }
        payload.extend(kind.as_bytes());
        payload.push(0);
        if let Some(hash) = icon_hash {
            payload.push(1);
            if version == 0x29 {
                payload.extend(3u32.to_le_bytes());
            } else {
                payload.extend(b"icon\0");
            }
            payload.extend(hash.as_bytes());
            payload.push(0);
        }
        payload.extend([8, 8, 8]);
        data.extend(id.to_le_bytes());
        data.extend((60 + payload.len() as u32).to_le_bytes());
        data.extend([0; 60]);
        data.extend(payload);
    }
    data.extend(0u32.to_le_bytes());
    if version == 0x29 {
        let offset = data.len() as u64;
        data[8..16].copy_from_slice(&offset.to_le_bytes());
        data.extend(4u32.to_le_bytes());
        data.extend(b"appinfo\0common\0type\0icon\0");
    }
    data
}
