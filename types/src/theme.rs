use std::{
    collections::HashMap,
    io::{Read, Seek},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

macro_rules! make_theme {
    (colors: [$($color:ident,)*], icons: [$($icon:ident,)*]) => {
        #[derive(Debug, Clone)]
        pub struct Theme {
            pub colors: ThemeColors,
            pub icons: ThemeIcons,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case")]
        pub struct PartialThemeManifest {
            pub name: String,

            #[serde(default)]
            pub colors: PartialThemeColors,
            #[serde(default)]
            pub icons: PartialThemeManifestIcons,
        }

        #[derive(Debug, Clone)]
        pub struct PartialTheme {
            pub name: String,

            pub colors: PartialThemeColors,
            pub icons: PartialThemeIcons,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case")]
        pub struct ThemeColors {
            $(pub $color: String,)*
        }
        #[derive(Debug, Clone, Default, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case", default)]
        pub struct PartialThemeColors {
            $(pub $color: Option<String>,)*
        }

        #[derive(Debug, Clone, Default, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case", default)]
        pub struct PartialThemeManifestIcons {
            $(pub $icon: Option<String>,)*
        }

        #[derive(Debug, Clone)]
        pub struct PartialThemeIcons {
            $(pub $icon: Option<Vec<u8>>,)*
        }

        #[derive(Debug, Clone)]
        pub struct ThemeIcons {
            $(pub $icon: Vec<u8>,)*
        }


        impl Theme {
            pub fn from_partial(partial: PartialTheme) -> Option<Self> {
                Some(Self {
                    colors: ThemeColors {
                        $(
                            $color: {
                                let Some(color) = partial.colors.$color else {
                                    return None;
                                };
                                color
                            },
                        )*
                    },
                    icons: ThemeIcons {
                        $(
                            $icon:  {
                                let Some(icon) = partial.icons.$icon else {
                                    return None;
                                };
                                icon
                            },
                        )*
                    },
                })
            }

            pub fn apply_partials(&mut self, partials: &[PartialTheme]) {
                for partial in partials {
                    $(
                        if let Some(color) = &partial.colors.$color {
                            self.colors.$color = color.clone();
                        }
                    )*

                    $(
                        if let Some(icon) = &partial.icons.$icon {
                            self.icons.$icon = icon.clone();
                        }
                    )*
                }
            }
        }

        impl PartialThemeManifestIcons {
            fn build_required_icons<T: Default>(&self) -> HashMap<String, T> {
                let mut icons = HashMap::new();
                $(
                    if let Some(icon) = self.$icon.as_ref() {
                        icons.insert(icon.clone(), T::default());
                    }
                )*
                icons

            }
        }

        impl PartialTheme {
            fn build(manifest: PartialThemeManifest, mut required_icons: HashMap<String, Option<Vec<u8>>>) -> Result<Self, ThemeError> {
                let icons = PartialThemeIcons {
                    $(
                        $icon: {
                            if let Some(ref icon) = manifest.icons.$icon {
                                if let Some(icon) = required_icons.get_mut(icon) {
                                    icon.take()
                                } else {
                                    return Err(ThemeError::MissingIcon(
                                        manifest.icons.$icon.as_ref().unwrap().clone(),
                                    ));
                                }
                            } else {
                                None
                            }

                        },
                    )*
                };

                Ok(Self {
                    name: manifest.name,
                    colors: manifest.colors,
                    icons,
                })
            }
        }

    };
}

make_theme! {
    colors: [
        background,
        foreground,

        container,
        border,

        button_idle,
        button_hover,
        button_press,

        success,
        warning,
        error,
    ],

    icons: [
        // play,
        // pause,
        // stop,
        // next,
        // previous,
        // volume_up,
        // volume_down,
        // volume_mute,

        background_task_running,
    ]
}

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("Failed to read theme data")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse theme data")]
    ParseError(#[from] toml::de::Error),
    #[error("Invalid data in theme manifest")]
    InvalidManifest(#[from] std::string::FromUtf8Error),
    #[error("Missing manifest file in theme")]
    MissingManifest,
    #[error("Theme manifest declares icon '{0}' but it is missing in the archive")]
    MissingIcon(String),
}

impl PartialTheme {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(reader)))]
    pub fn load<R: Read + Seek>(reader: R) -> Result<Self, ThemeError> {
        let mut archive = tar::Archive::new(reader);

        let mut manifest: Option<PartialThemeManifest> = None;
        for entry in archive.entries_with_seek()? {
            let mut entry = entry?;
            let path = entry.path()?;

            let Some(name) = path.file_name() else {
                continue;
            };

            if name == "theme.toml" {
                let mut contents = Vec::new();
                entry.read_to_end(&mut contents)?;
                let contents = String::from_utf8(contents)?;
                manifest = Some(toml::from_str(&contents)?);
                break;
            }
        }

        let manifest = manifest.ok_or(ThemeError::MissingManifest)?;

        let mut required_icons = manifest.icons.build_required_icons();

        let mut reader = archive.into_inner();
        reader.seek(std::io::SeekFrom::Start(0))?;
        let mut archive = tar::Archive::new(reader);
        for entry in archive.entries_with_seek()? {
            let mut entry = entry?;
            let path = entry.path()?;

            let Some(name) = path.file_name() else {
                continue;
            };

            if let Some(icon_data) = required_icons.get_mut(name.to_string_lossy().as_ref()) {
                let mut contents = Vec::new();
                entry.read_to_end(&mut contents)?;
                *icon_data = Some(contents);
            }
        }

        let theme = PartialTheme::build(manifest, required_icons)?;

        Ok(theme)
    }
}
