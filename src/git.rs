use url::Url;

pub fn extract_repo_name(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let mut segments = parsed.path_segments()?;
    let username = segments.next()?;
    let mut repo = segments.next()?;

    if let Some(stripped) = repo.strip_suffix(".git") {
        repo = stripped;
    }

    Some(format!("{}-{}", username, repo))
}

pub fn extract_version(url: &str) -> (String, Option<String>) {
    let repo_name = url.replace("https://tangled.sh/", "").replace("@", "");
    match url.rfind('@') {
        Some(idx) => {
            let (repo, version) = url.split_at(idx);
            let version = &version[1..];
            match version == repo_name {
                true => (url.to_string(), None),
                false => (repo.to_string(), Some(version.to_string())),
            }
        }
        None => (url.to_string(), None),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_extract_repo_name() {
        let url = "https://github.com/tsirysndr/pkgs.git";
        let expected = Some("tsirysndr-pkgs".into());
        let result = extract_repo_name(url);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_repo_name_no_git() {
        let url = "https://github.com/tsirysndr/pkgs";
        let expected = Some("tsirysndr-pkgs".into());
        let result = extract_repo_name(url);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_repo_name_invalid_url() {
        let url = "invalid-url";
        let expected = None;
        let result = extract_repo_name(url);
        assert_eq!(result, expected);
    }
}
