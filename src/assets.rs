use directories::ProjectDirs;
use errors::*;
use std::borrow::Cow;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use syntect::dumps::{dump_to_file, from_binary, from_reader};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

lazy_static! {
    static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("", "", crate_name!());
}

pub struct HighlightingAssets {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}

impl HighlightingAssets {
    pub fn new() -> Self {
        Self::from_cache().unwrap_or_else(|_| Self::from_binary())
    }

    pub fn from_files(dir: Option<&Path>) -> Result<Self> {
        let source_dir = dir.unwrap_or_else(|| PROJECT_DIRS.config_dir());

        let theme_dir = source_dir.join("themes");
        let theme_set =
            ThemeSet::load_from_folder(&theme_dir).chain_err(|| "Could not load themes from '{}'")?;
        let mut syntax_set = SyntaxSet::new();
        let syntax_dir = source_dir.join("syntaxes");
        if !syntax_dir.exists() {
            return Err(format!(
                "Could not load syntaxes from '{}'",
                syntax_dir.to_string_lossy()
            ).into());
        }
        let _ = syntax_set.load_syntaxes(syntax_dir, true);
        syntax_set.load_plain_text_syntax();

        Ok(HighlightingAssets {
            syntax_set,
            theme_set,
        })
    }

    fn from_cache() -> Result<Self> {
        let theme_set_path = theme_set_path();
        let syntax_set_file = File::open(&syntax_set_path()).chain_err(|| {
            format!(
                "Could not load cached syntax set '{}'",
                syntax_set_path().to_string_lossy()
            )
        })?;
        let mut syntax_set: SyntaxSet =
            from_reader(syntax_set_file).chain_err(|| "Could not parse cached syntax set")?;
        syntax_set.link_syntaxes();

        let theme_set_file = File::open(&theme_set_path).chain_err(|| {
            format!(
                "Could not load cached theme set '{}'",
                theme_set_path.to_string_lossy()
            )
        })?;
        let theme_set: ThemeSet =
            from_reader(theme_set_file).chain_err(|| "Could not parse cached theme set")?;

        Ok(HighlightingAssets {
            syntax_set,
            theme_set,
        })
    }

    fn from_binary() -> Self {
        let mut syntax_set: SyntaxSet = from_binary(include_bytes!("../assets/syntaxes.bin"));
        syntax_set.link_syntaxes();
        let theme_set: ThemeSet = from_binary(include_bytes!("../assets/themes.bin"));

        HighlightingAssets {
            syntax_set,
            theme_set,
        }
    }

    pub fn save(&self, dir: Option<&Path>) -> Result<()> {
        let target_dir = dir.unwrap_or_else(|| PROJECT_DIRS.cache_dir());
        let _ = fs::create_dir(target_dir);
        let theme_set_path = target_dir.join("themes.bin");
        let syntax_set_path = target_dir.join("syntaxes.bin");

        print!(
            "Writing theme set to {} ... ",
            theme_set_path.to_string_lossy()
        );
        dump_to_file(&self.theme_set, &theme_set_path).chain_err(|| {
            format!(
                "Could not save theme set to {}",
                theme_set_path.to_string_lossy()
            )
        })?;
        println!("okay");

        print!(
            "Writing syntax set to {} ... ",
            syntax_set_path.to_string_lossy()
        );
        dump_to_file(&self.syntax_set, &syntax_set_path).chain_err(|| {
            format!(
                "Could not save syntax set to {}",
                syntax_set_path.to_string_lossy()
            )
        })?;
        println!("okay");

        Ok(())
    }

    pub fn get_theme(&self, theme: &str) -> Result<&Theme> {
        Ok(self.theme_set.themes.get(theme).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Could not find '{}' theme", theme),
            )
        })?)
    }
}

pub fn theme_set_path() -> PathBuf {
    PROJECT_DIRS.cache_dir().join("themes.bin")
}

pub fn syntax_set_path() -> PathBuf {
    PROJECT_DIRS.cache_dir().join("syntaxes.bin")
}

pub fn config_dir() -> Cow<'static, str> {
    PROJECT_DIRS.config_dir().to_string_lossy()
}
