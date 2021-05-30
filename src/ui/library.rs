//! Loads and displays the Miratope library.

use std::{
    ffi::{OsStr, OsString},
    fs, io,
    path::PathBuf,
};

use super::config::Config;
use crate::{
    lang::{
        name::{Con, Name},
        SelectedLanguage,
    },
    polytope::{concrete::Concrete, r#abstract::rank::Rank, Polytope},
};

use bevy::prelude::*;
use bevy_egui::{egui, egui::Ui, EguiContext};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

/// The plugin that loads the library.
pub struct LibraryPlugin;

impl Plugin for LibraryPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // This must run after the Config resource has been added.
        let lib_path = app
            .world()
            .get_resource::<Config>()
            .unwrap()
            .data
            .lib_path
            .clone();

        app.insert_resource(Library::new_folder(&lib_path))
            .add_system(show_library.system().after("show_top_panel"));
    }
}

/// Represents any of the special polytopes in Miratope's library, namely those
/// families that are generated by code.
///
/// The variants of the special library store whatever value is currently being
/// stored on screen. When the user clicks on the button to load them, they're
/// sent together with their values as a [`ShowResult`] to the [`show_library`]
/// system, which then actually loads the polytope.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, Display)]
pub enum SpecialLibrary {
    /// A regular polygon.
    #[strum(serialize = "Regular polygon")]
    Polygons(usize, usize),

    /// A (uniform 3D) prism.
    Prisms(usize, usize),

    /// A (uniform 3D) antiprism.
    Antiprisms(usize, usize),

    /// A (4D uniform) duoprism.
    Duoprisms(usize, usize, usize, usize),

    /// A (4D uniform) antiprismatic prism.
    #[strum(serialize = "Antiprismatic prisms")]
    AntiprismPrisms(usize, usize),

    /// A simplex.
    Simplex(Rank),

    /// A hypercube.
    Hypercube(Rank),

    /// An orthoplex.
    Orthoplex(Rank),
}

/// The result of showing the Miratope library every frame.
pub enum ShowResult {
    /// Nothing happened this frame.
    None,

    /// We asked to load a file.
    Load(OsString),

    /// We asked to load a special polytope.
    Special(SpecialLibrary),
}

impl SpecialLibrary {
    /// Shows the special component of the library. Returns the action selected
    /// by the user, if any.
    pub fn show(&mut self, ui: &mut Ui, _selected_language: SelectedLanguage) -> ShowResult {
        let text = self.to_string();

        match self {
            // An {n / d} regular polygon or uniform polygonal prism.
            Self::Polygons(n, d) | Self::Prisms(n, d) => {
                let mut clicked = false;

                ui.horizontal(|ui| {
                    clicked = ui.button(text).clicked();

                    // Number of sides.
                    ui.label("n:");
                    ui.add(
                        egui::DragValue::new(n)
                            .speed(0.25)
                            .clamp_range(2..=usize::MAX),
                    );

                    // Turning number.
                    let max_n = *n / 2;
                    ui.label("d:");
                    ui.add(egui::DragValue::new(d).speed(0.25).clamp_range(1..=max_n));
                });

                if clicked {
                    ShowResult::Special(*self)
                } else {
                    ShowResult::None
                }
            }

            // An {n / d} uniform antiprism.
            Self::Antiprisms(n, d) | Self::AntiprismPrisms(n, d) => {
                let mut clicked = false;

                ui.horizontal(|ui| {
                    clicked = ui.button(text).clicked();

                    // Number of sides.
                    ui.label("n:");
                    ui.add(
                        egui::DragValue::new(n)
                            .speed(0.25)
                            .clamp_range(2..=usize::MAX),
                    );

                    // Turning number.
                    let max_n = *n * 2 / 3;
                    ui.label("d:");
                    ui.add(egui::DragValue::new(d).speed(0.25).clamp_range(1..=max_n));
                });

                if clicked {
                    ShowResult::Special(*self)
                } else {
                    ShowResult::None
                }
            }

            // An step prism based on two uniform polygons..
            Self::Duoprisms(n1, d1, n2, d2) => {
                let mut clicked = false;

                ui.horizontal_wrapped(|ui| {
                    clicked = ui.button(text).clicked();

                    // Number of sides.
                    ui.label("n₁:");
                    ui.add(
                        egui::DragValue::new(n1)
                            .speed(0.25)
                            .clamp_range(2..=usize::MAX),
                    );

                    // Turning number.
                    let max_n1 = *n1 / 2;
                    ui.label("d₁:");
                    ui.add(egui::DragValue::new(d1).speed(0.25).clamp_range(1..=max_n1));

                    // Number of sides.
                    ui.label("n₂:");
                    ui.add(
                        egui::DragValue::new(n2)
                            .speed(0.25)
                            .clamp_range(2..=usize::MAX),
                    );

                    // Turning number.
                    let max_n2 = *n2 / 2;
                    ui.label("d₂:");
                    ui.add(egui::DragValue::new(d2).speed(0.25).clamp_range(1..=max_n2));
                });

                if clicked {
                    ShowResult::Special(*self)
                } else {
                    ShowResult::None
                }
            }

            // A simplex, hypercube, or orthoplex of a given rank.
            Self::Simplex(rank) | Self::Hypercube(rank) | Self::Orthoplex(rank) => {
                let mut clicked = false;

                ui.horizontal(|ui| {
                    clicked = ui.button(text).clicked();

                    // Rank.
                    ui.label("Rank:");
                    ui.add(egui::DragValue::new(rank).speed(0.05).clamp_range(-1..=20));
                });

                if clicked {
                    ShowResult::Special(*self)
                } else {
                    ShowResult::None
                }
            }
        }
    }
}

/// The display name for a file or folder.
#[derive(Clone, Serialize, Deserialize)]
pub enum DisplayName {
    /// A name in its language-independent representation.
    Name(Name<Con>),

    /// A literal string name.
    Literal(String),
}

impl DisplayName {
    /// This is running at 60 FPS but name parsing isn't blazing fast. Maybe
    /// do some sort of cacheing in the future?
    pub fn parse(&self, selected_language: SelectedLanguage) -> String {
        match self {
            Self::Name(name) => selected_language.parse_uppercase(name, Default::default()),
            Self::Literal(name) => name.clone(),
        }
    }
}

/// Represents any of the files or folders that make up the Miratope library.
///
/// The library is internally stored is a tree-like structure. Once a folder
/// loads, it's (currently) never unloaded.
#[derive(Serialize, Deserialize)]
pub enum Library {
    /// A folder whose contents have not yet been read.
    UnloadedFolder {
        /// The name of the folder in disk.
        folder_name: String,

        /// The display name of the folder.
        name: DisplayName,
    },

    /// A folder whose contents have been read.
    LoadedFolder {
        /// The name of the folder in disk.
        folder_name: String,

        /// The display name of the folder.
        name: DisplayName,

        /// The contents of the folder.
        contents: Vec<Library>,
    },

    /// A file that can be loaded into Miratope.
    File {
        /// The name of the file in disk.
        file_name: String,

        /// The display name of the file.
        name: DisplayName,
    },

    /// Any special file in the library.
    Special(SpecialLibrary),
}

/// Implements the or operator, so that `a | b` is `a` if it isn't `None`, but
/// `b` otherwise.
impl std::ops::BitOr for ShowResult {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        if let Self::None = self {
            rhs
        } else {
            self
        }
    }
}

/// Implements the or assignment operator, defined in the same way as the or
/// operator.
impl std::ops::BitOrAssign for ShowResult {
    fn bitor_assign(&mut self, rhs: Self) {
        if !matches!(rhs, Self::None) {
            *self = rhs;
        }
    }
}

/// Converts the given `PathBuf` into a `String`.
pub fn path_to_str(path: PathBuf) -> String {
    path.file_name().unwrap().to_string_lossy().into_owned()
}

impl Library {
    /// Loads the data from a file at a given path.
    pub fn new_file(path: &impl AsRef<OsStr>) -> Self {
        let path = PathBuf::from(&path);
        let name = if let Some(name) = Concrete::name_from_off(&path) {
            DisplayName::Name(name)
        } else {
            DisplayName::Literal(String::from(
                path.file_stem().map(|f| f.to_str()).flatten().unwrap_or(""),
            ))
        };

        Self::File {
            file_name: path_to_str(path),
            name,
        }
    }

    /// Creates a new unloaded folder from a given path. If the path doesn't
    /// exist or doesn't refer to a folder, we return `None`.
    pub fn new_folder<T: AsRef<OsStr>>(path: &T) -> Option<Self> {
        let path = PathBuf::from(&path);
        if !(path.exists() && path.is_dir()) {
            return None;
        }

        // Attempts to read from the .name file.
        Some(
            if let Ok(Ok(name)) = fs::read(path.join(".name"))
                .map(|file| ron::from_str(&String::from_utf8(file).unwrap()))
            {
                Self::UnloadedFolder {
                    folder_name: path_to_str(path),
                    name,
                }
            }
            // Else, takes the name from the folder itself.
            else {
                let folder_name = String::from(
                    path.file_name()
                        .map(|name| name.to_str())
                        .flatten()
                        .unwrap_or(""),
                );

                Self::UnloadedFolder {
                    name: DisplayName::Literal(folder_name.clone()),
                    folder_name,
                }
            },
        )
    }

    /// Reads a folder's data from the `.folder` file. If it doesn't exist, it
    /// defaults to loading the folder's name and its data in alphabetical
    /// order. If that also fails, it returns an `Err`.
    pub fn folder_contents(path: &impl AsRef<OsStr>) -> io::Result<Vec<Self>> {
        let path = PathBuf::from(&path);
        assert!(path.is_dir(), "Path {:?} not a directory!", path);

        // Attempts to read from the .folder file.
        Ok(
            if let Some(Ok(folder)) = fs::read(path.join(".folder"))
                .ok()
                .map(|file| ron::from_str(&String::from_utf8(file).unwrap()))
            {
                folder
            }
            // Otherwise, just manually goes through the files.
            else {
                let mut contents = Vec::new();

                for entry in fs::read_dir(path.clone())? {
                    let path = &entry?.path();

                    // Adds a new unloaded folder.
                    if let Some(unloaded_folder) = Self::new_folder(path) {
                        contents.push(unloaded_folder);
                    }
                    // Adds a new file.
                    else {
                        let ext = path.extension();

                        if ext == Some(OsStr::new("off")) || ext == Some(OsStr::new("ggb")) {
                            contents.push(Self::new_file(path));
                        }
                    }
                }

                // We cache these contents for future use.
                if fs::write(path.join(".folder"), ron::to_string(&contents).unwrap()).is_ok() {
                    println!(".folder file overwritten!");
                } else {
                    println!(".folder file could not be overwritten!");
                }

                contents
            },
        )
    }

    /// Shows the library from the root.
    pub fn show_root(&mut self, ui: &mut Ui, selected_language: SelectedLanguage) -> ShowResult {
        self.show(ui, PathBuf::new(), selected_language)
    }

    /// Shows the library.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        mut path: PathBuf,
        selected_language: SelectedLanguage,
    ) -> ShowResult {
        match self {
            // Shows a collapsing drop-down, and loads the folder in case it's clicked.
            Self::UnloadedFolder { folder_name, name } => {
                // Clones so that the closure doesn't require unique access.
                let folder_name = folder_name.clone();
                let name = name.clone();

                path.push(folder_name);
                let mut res = ShowResult::None;

                ui.collapsing(name.parse(selected_language), |ui| {
                    let mut contents = Self::folder_contents(&path).unwrap();

                    // Contents of drop down.
                    for lib in contents.iter_mut() {
                        res |= lib.show(ui, path.clone(), selected_language);
                    }

                    // Opens the folder.
                    *self = Self::LoadedFolder {
                        folder_name: path_to_str(path),
                        name,
                        contents,
                    };
                });

                res
            }

            // Shows a drop-down with all of the files and folders.
            Self::LoadedFolder {
                folder_name,
                name,
                contents,
            } => {
                path.push(&folder_name);
                let mut res = ShowResult::None;

                ui.collapsing(name.parse(selected_language), |ui| {
                    for lib in contents.iter_mut() {
                        res |= lib.show(ui, path.clone(), selected_language);
                    }
                });

                res
            }

            // Shows a button that loads the file if clicked.
            Self::File { file_name, name } => {
                path.push(file_name);

                if ui.button(name.parse(selected_language)).clicked() {
                    ShowResult::Load(path.into_os_string())
                } else {
                    ShowResult::None
                }
            }

            // Shows any of the special files.
            Self::Special(special) => special.show(ui, selected_language),
        }
    }
}

/// The system that shows the Miratope library.
fn show_library(
    egui_ctx: Res<EguiContext>,
    mut query: Query<&mut Concrete>,
    mut library: ResMut<Option<Library>>,
    selected_language: Res<SelectedLanguage>,
) {
    // Shows the polytope library.
    if let Some(library) = &mut *library {
        egui::SidePanel::left("side_panel")
            .default_width(350.0)
            .max_width(450.0)
            .show(egui_ctx.ctx(), |ui| {
                egui::containers::ScrollArea::auto_sized().show(ui, |ui| {
                    match library.show_root(ui, *selected_language) {
                        // No action needs to be taken.
                        ShowResult::None => {}

                        // Loads a selected file.
                        ShowResult::Load(file) => {
                            if let Some(mut p) = query.iter_mut().next() {
                                if let Ok(res) = Concrete::from_path(&file) {
                                    match res {
                                        Ok(q) => *p = q,
                                        Err(err) => println!("{:?}", err),
                                    }
                                } else {
                                    println!("File open failed!");
                                }
                            }
                        }

                        // Loads a special polytope.
                        ShowResult::Special(special) => match special {
                            // Loads a regular star polygon.
                            SpecialLibrary::Polygons(n, d) => {
                                if let Some(mut p) = query.iter_mut().next() {
                                    *p = Concrete::star_polygon(n, d);
                                }
                            }

                            // Loads a uniform polygonal prism.
                            SpecialLibrary::Prisms(n, d) => {
                                if let Some(mut p) = query.iter_mut().next() {
                                    *p = Concrete::uniform_prism(n, d);
                                }
                            }

                            // Loads a uniform polygonal antiprism.
                            SpecialLibrary::Antiprisms(n, d) => {
                                if let Some(mut p) = query.iter_mut().next() {
                                    *p = Concrete::uniform_antiprism(n, d);
                                }
                            }

                            // Loads a (uniform 4D) duoprism.
                            SpecialLibrary::Duoprisms(n1, d1, n2, d2) => {
                                if let Some(mut p) = query.iter_mut().next() {
                                    let p1 = Concrete::star_polygon(n1, d1);

                                    if n1 == n2 && d1 == d2 {
                                        *p = Concrete::duoprism(&p1, &p1);
                                    } else {
                                        let p2 = Concrete::star_polygon(n2, d2);
                                        *p = Concrete::duoprism(&p1, &p2);
                                    }
                                }
                            }

                            // Loads a uniform polygonal antiprism.
                            SpecialLibrary::AntiprismPrisms(n, d) => {
                                if let Some(mut p) = query.iter_mut().next() {
                                    *p = Concrete::uniform_antiprism(n, d).prism();
                                }
                            }

                            // Loads a simplex with a given rank.
                            SpecialLibrary::Simplex(rank) => {
                                if let Some(mut p) = query.iter_mut().next() {
                                    *p = Concrete::simplex(rank);
                                }
                            }

                            // Loads a hypercube with a given rank.
                            SpecialLibrary::Hypercube(rank) => {
                                if let Some(mut p) = query.iter_mut().next() {
                                    *p = Concrete::hypercube(rank);
                                }
                            }

                            // Loads an orthoplex with a given rank.
                            SpecialLibrary::Orthoplex(rank) => {
                                if let Some(mut p) = query.iter_mut().next() {
                                    *p = Concrete::orthoplex(rank);
                                }
                            }
                        },
                    }
                })
            });
    }
}
