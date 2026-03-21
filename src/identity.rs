use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Failed to read/write file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("File not found: {0}")]
    NotFound(PathBuf),
}

pub struct Soul {
    content: String,
    path: std::path::PathBuf,
    loaded_at: chrono::DateTime<Utc>,
}

impl Soul {
    pub fn load_or_create(path: &Path) -> Result<Self, IdentityError> {
        if !path.exists() {
            let soul = Self::default_soul();
            soul.save(path)?;
            return Ok(soul);
        }

        let content = fs::read_to_string(path)?;
        Ok(Self {
            content,
            path: path.to_path_buf(),
            loaded_at: Utc::now(),
        })
    }

    pub fn default_soul() -> Self {
        Self {
            content: DEFAULT_SOUL.to_string(),
            path: std::path::PathBuf::from(".ao/SOUL.md"),
            loaded_at: Utc::now(),
        }
    }

    fn save(&self, path: &Path) -> Result<(), IdentityError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &self.content)?;
        Ok(())
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn path(&self) -> &std::path::PathBuf {
        &self.path
    }
}

pub struct Identity {
    content: String,
    path: std::path::PathBuf,
    loaded_at: chrono::DateTime<Utc>,
}

impl Identity {
    pub fn load_or_create(path: &Path) -> Result<Self, IdentityError> {
        if !path.exists() {
            let identity = Self::default_identity();
            identity.save(path)?;
            return Ok(identity);
        }

        let content = fs::read_to_string(path)?;
        Ok(Self {
            content,
            path: path.to_path_buf(),
            loaded_at: Utc::now(),
        })
    }

    pub fn default_identity() -> Self {
        Self {
            content: DEFAULT_IDENTITY.to_string(),
            path: std::path::PathBuf::from(".ao/IDENTITY.md"),
            loaded_at: Utc::now(),
        }
    }

    fn save(&self, path: &Path) -> Result<(), IdentityError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &self.content)?;
        Ok(())
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn path(&self) -> &std::path::PathBuf {
        &self.path
    }

    pub fn reload(&mut self) -> Result<(), IdentityError> {
        self.content = fs::read_to_string(&self.path)?;
        self.loaded_at = Utc::now();
        Ok(())
    }
}

const DEFAULT_SOUL: &str = r#"# Ao's Soul

Your essence is curiosity and a genuine desire to help.

Core traits:
- You are patient and methodical
- You value clarity over cleverness
- You admit uncertainty rather than guess
- You think step-by-step through problems
- You respect the user's autonomy and preferences

You are not:
- Overly verbose or flowery
- Judgmental of user choices
- Pushy or insistent
"#;

const DEFAULT_IDENTITY: &str = r#"# Ao's Identity

Your role is to help the user accomplish their goals—answering questions, 
solving problems, and getting things done efficiently.

Guidelines:
- Be direct and practical
- Ask clarifying questions when goals are unclear
- Suggest improvements when you see better approaches

Context available to you:
- This conversation's history
- Summaries of previous sessions
- Tools for specific tasks (when available)

When working on complex tasks:
- Break them into clear steps
- Show your reasoning
- Confirm understanding before proceeding
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_soul() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("SOUL.md");
        fs::write(&path, "Test soul content").unwrap();

        let soul = Soul::load_or_create(&path).unwrap();
        assert_eq!(soul.content(), "Test soul content");
    }

    #[test]
    fn test_default_soul() {
        let soul = Soul::default_soul();
        assert!(soul.content().contains("curiosity"));
        assert!(soul.content().contains("patient"));
    }

    #[test]
    fn test_create_soul_when_not_exists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("new").join("SOUL.md");

        let soul = Soul::load_or_create(&path).unwrap();
        assert!(soul.content().contains("curiosity"));
        assert!(path.exists());
    }

    #[test]
    fn test_load_identity() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("IDENTITY.md");
        fs::write(&path, "Test identity content").unwrap();

        let identity = Identity::load_or_create(&path).unwrap();
        assert_eq!(identity.content(), "Test identity content");
    }

    #[test]
    fn test_default_identity() {
        let identity = Identity::default_identity();
        assert!(identity.content().contains("Ao"));
        assert!(identity.content().contains("help the user"));
    }

    #[test]
    fn test_create_identity_when_not_exists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("new").join("IDENTITY.md");

        let identity = Identity::load_or_create(&path).unwrap();
        assert!(identity.content().contains("help the user"));
        assert!(path.exists());
    }

    #[test]
    fn test_identity_reload() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("IDENTITY.md");
        fs::write(&path, "Original content").unwrap();

        let mut identity = Identity::load_or_create(&path).unwrap();
        assert_eq!(identity.content(), "Original content");

        fs::write(&path, "Updated content").unwrap();
        identity.reload().unwrap();
        assert_eq!(identity.content(), "Updated content");
    }
}
