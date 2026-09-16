use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputDirIssue {
    NotChosen,
    Missing(PathBuf),
}

/// Resolve output directory for PDF generation (must exist on disk).
pub fn resolve_output_dir_for_generate(
    root: &Path,
    output_dir: &str,
) -> Result<PathBuf, OutputDirIssue> {
    let trimmed = output_dir.trim();
    if trimmed.is_empty() || trimmed == "output" {
        return Err(OutputDirIssue::NotChosen);
    }
    let p = PathBuf::from(trimmed);
    let path = if p.is_absolute() {
        p
    } else {
        root.join(p)
    };
    if !path.is_dir() {
        return Err(OutputDirIssue::Missing(path));
    }
    Ok(path)
}

pub fn output_dir_issue_message(issue: &OutputDirIssue) -> String {
    match issue {
        OutputDirIssue::NotChosen => "Aucun dossier de sortie choisi.".into(),
        OutputDirIssue::Missing(p) => format!(
            "Le dossier enregistré pour les PDF n’existe plus : {}",
            p.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn empty_output_dir_is_not_chosen() {
        assert_eq!(
            resolve_output_dir_for_generate(Path::new("."), ""),
            Err(OutputDirIssue::NotChosen)
        );
    }

    #[test]
    fn missing_directory_returns_missing() {
        let base = env::temp_dir().join(format!("marius-out-missing-{}", std::process::id()));
        let missing = base.join("gone-subdir");
        assert_eq!(
            resolve_output_dir_for_generate(&base, missing.to_string_lossy().as_ref()),
            Err(OutputDirIssue::Missing(missing))
        );
    }

    #[test]
    fn existing_directory_resolves() {
        let base = env::temp_dir().join(format!("marius-out-ok-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let got = resolve_output_dir_for_generate(Path::new("."), base.to_string_lossy().as_ref())
            .unwrap();
        assert_eq!(got, base);
        let _ = fs::remove_dir_all(&base);
    }
}
