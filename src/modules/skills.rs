use anyhow;

pub fn get_skills() -> anyhow::Result<String> {
    let mut dir = std::fs::read_dir("skills")?;
    let mut content = String::new();
    while let Some(entry) = dir.next() {
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("无效文件名"))?;
        let full_path = "skills/".to_string() + &name;
        let body = std::fs::read_to_string(full_path)?;
        content.push_str(&body);
        content.push('\n');
    }

    Ok(content)
}
