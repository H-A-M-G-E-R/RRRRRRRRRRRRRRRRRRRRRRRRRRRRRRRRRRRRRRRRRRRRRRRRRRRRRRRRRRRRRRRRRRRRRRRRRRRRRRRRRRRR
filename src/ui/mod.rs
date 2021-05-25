//! All of the code that configures the UI.

use approx::abs_diff_eq;
use bevy_egui::egui::{self, Ui, Widget};

use crate::{geometry::Point, Consts, Float};

pub mod camera;
pub mod egui_windows;
pub mod library;
pub mod main_window;
pub mod memory;
pub mod top_panel;

pub struct MiratopePlugins;

impl bevy::prelude::PluginGroup for MiratopePlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(camera::InputPlugin)
            .add(egui_windows::EguiWindowPlugin)
            .add(library::LibraryPlugin)
            .add(main_window::MainWindowPlugin)
            .add(top_panel::TopPanelPlugin);
    }
}

/// A widget that sets a point.
pub struct PointWidget<'a> {
    label: String,
    point: &'a mut Point,
}

impl<'a> PointWidget<'a> {
    pub fn new(point: &'a mut Point, label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            point,
        }
    }
}

impl<'a> Widget for PointWidget<'a> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label(self.label);
            for c in self.point.iter_mut() {
                ui.add(egui::DragValue::new(c).speed(0.01));
            }
        })
        .response
    }
}

/// A widget that sets up a point of unit norm.
pub struct UnitPointWidget<'a>(PointWidget<'a>);

impl<'a> UnitPointWidget<'a> {
    /// Initializes a new unit point widget.
    pub fn new(point: &'a mut Point, label: impl ToString) -> Self {
        Self(PointWidget::new(point, label))
    }
}

impl<'a> Widget for UnitPointWidget<'a> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label(self.0.label);

            let old_point = self.0.point.clone();
            let mut modified_coord = 0;

            for (idx, coord) in self.0.point.iter_mut().enumerate() {
                ui.add(egui::DragValue::new(coord).speed(0.01));

                // The index of the modified coordinate.
                if abs_diff_eq!(old_point[idx], *coord, epsilon = Float::EPS) {
                    modified_coord = idx;
                }

                // Gets rid of floating point shenanigans.
                if abs_diff_eq!(*coord, 0.0, epsilon = Float::EPS.sqrt()) {
                    *coord = 0.0;
                } else if abs_diff_eq!(*coord, 1.0, epsilon = Float::EPS) {
                    *coord = 1.0;
                } else if abs_diff_eq!(*coord, -1.0, epsilon = Float::EPS) {
                    *coord = -1.0;
                }
            }

            // Normalizes the point.
            if self.0.point.try_normalize_mut(Float::EPS).is_none() {
                // If this fails, sets it to the axis direction corresponding
                // to the last modified coordinate.
                for coord in self.0.point.iter_mut() {
                    *coord = 0.0;
                }
                self.0.point[modified_coord] = 1.0;
            }
        })
        .response
    }
}
