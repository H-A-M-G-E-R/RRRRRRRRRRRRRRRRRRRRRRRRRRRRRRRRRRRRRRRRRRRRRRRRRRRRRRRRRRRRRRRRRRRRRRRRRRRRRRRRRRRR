//! The systems that update the main window.

use crate::{
    lang::{Options, SelectedLanguage},
    polytope::{concrete::Concrete, Polytope},
};

use super::top_panel::SectionState;

use bevy::prelude::*;
use bevy_egui::EguiSettings;

/// The plugin in charge of the Miratope main window, and of drawing the
/// polytope onto it.
pub struct MainWindowPlugin;

impl Plugin for MainWindowPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(CoreStage::PreUpdate, update_visible.system())
            .add_system(update_scale_factor.system())
            .add_system_to_stage(CoreStage::PostUpdate, update_changed_polytopes.system());
    }
}

pub fn update_visible(
    keyboard: Res<Input<KeyCode>>,
    mut polies_vis: Query<&mut Visible, With<Concrete>>,
    mut wfs_vis: Query<&mut Visible, Without<Concrete>>,
) {
    if keyboard.just_pressed(KeyCode::V) {
        if let Some(mut visible) = polies_vis.iter_mut().next() {
            let vis = visible.is_visible;
            visible.is_visible = !vis;
        }
    }

    if keyboard.just_pressed(KeyCode::B) {
        if let Some(mut visible) = wfs_vis.iter_mut().next() {
            let vis = visible.is_visible;
            visible.is_visible = !vis;
        }
    }
}

/// Resizes the UI when the screen is resized.
pub fn update_scale_factor(mut egui_settings: ResMut<EguiSettings>, windows: Res<Windows>) {
    if let Some(window) = windows.get_primary() {
        egui_settings.scale_factor = 1.0 / window.scale_factor();
    }
}

/// Updates polytopes after an operation.
pub fn update_changed_polytopes(
    mut meshes: ResMut<Assets<Mesh>>,

    polies: Query<(&Concrete, &Handle<Mesh>, &Children), Changed<Concrete>>,
    wfs: Query<&Handle<Mesh>, Without<Concrete>>,

    mut windows: ResMut<Windows>,
    mut section_state: ResMut<SectionState>,
    selected_language: Res<SelectedLanguage>,
) {
    for (poly, mesh_handle, children) in polies.iter() {
        if cfg!(debug_assertions) {
            println!("Polytope updated");
            poly.abs.is_valid().unwrap();
        }

        *meshes.get_mut(mesh_handle).unwrap() = poly.mesh();

        // Sets the window's name to the polytope's name.
        windows
            .get_primary_mut()
            .unwrap()
            .set_title(selected_language.parse(poly.name(), Options::default()));

        // Updates all wireframes.
        for child in children.iter() {
            if let Ok(wf_handle) = wfs.get_component::<Handle<Mesh>>(*child) {
                *meshes.get_mut(wf_handle).unwrap() = poly.wireframe();
            }
        }

        // We reset the cross-section view if we didn't use it to change the polytope.
        if !section_state.is_changed() {
            section_state.close();
        }
    }
}
