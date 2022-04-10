//! Contains all code related to the right side panel.

use crate::Concrete;

use bevy::prelude::*;
use bevy_egui::{
    egui,
    EguiContext,
};
use miratope_core::{conc::{element_types::{EL_NAMES, EL_SUFFIXES}, ConcretePolytope}, Polytope, abs::Ranked, geometry::{Subspace, Point, Vector}};
use vec_like::VecLike;

use super::top_panel::{SectionDirection, SectionState};

#[derive(Clone, Copy, Debug)]
struct ElementTypeWithData {
    /// The index of the representative for this element type.
    example: usize,

    /// The number of elements of this type.
    count: usize,

    /// The number of facets.
    facets: usize,

    /// The number of facets of the figure.
    fig_facets: usize,

    /// The circumradius of the element, or distance from the origin if it's a vertex.
    radius: Option<f64>,
}

#[derive(Clone)]
pub struct ElementTypesRes {
    /// The polytope whose data we're getting.
    poly: Concrete,

    /// The element types.
    types: Vec<Vec<ElementTypeWithData>>,

    /// The components.
    components: Vec<Concrete>,

    /// Whether the loaded polytope matches `poly` and the buttons should be greyed out.
    pub main: bool,

    /// Whether we're updating `main`.
    pub main_updating: bool,
}

impl Default for ElementTypesRes {
    fn default() -> ElementTypesRes {
        ElementTypesRes {
            poly: Concrete::nullitope(),
            types: Vec::new(),
            components: vec![Concrete::nullitope()],
            main: true,
            main_updating: false,
        }
    }
}

impl ElementTypesRes {
    fn from_poly(&self, poly: Mut<'_, Concrete>) -> ElementTypesRes {
        let mut poly = poly.clone();
        poly.element_sort();

        let plain_types = poly.element_types();
        let mut types_with_data = Vec::new();
    
        for (r, types) in plain_types.clone().into_iter().enumerate() {
            let rank = poly.rank();
            if r == rank {
                break;
            }
    
            let abs = &poly.abs;
            let dual_abs = &abs.dual();
            let mut types_with_data_this_rank = Vec::new();
            
            for t in types {
                let idx = t.example;
    
                let facets = abs[(r, idx)].subs.len();
                let fig_facets = dual_abs.element_vertices(rank-r, idx).unwrap().len();
                let radius = 
                    if r == 1 {
                        Some(poly.vertices[idx].norm())
                    }
                    else {
                        if let Some(sphere) = poly.element(r, idx).unwrap().circumsphere() {
                        Some(sphere.radius())
                        } else {
                            None
                        }
                    };
    
                types_with_data_this_rank.push(ElementTypeWithData {
                    example: idx,
                    count: t.count,
                    facets,
                    fig_facets,
                    radius,
                });
            }
            types_with_data.push(types_with_data_this_rank);
        }

        let components = poly.defiss();
    
        ElementTypesRes {
            poly: poly.clone(),
            types: types_with_data,
            components,
            main: true,
            main_updating: false,
        }
    }
}

/// The plugin in charge of everything on the right panel.
pub struct RightPanelPlugin;

impl Plugin for RightPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ElementTypesRes>()
            // The top panel must be shown first.
            .add_system(
                show_right_panel
                    .system()
                    .label("show_right_panel")
                    .after("show_top_panel"),
            );
    }
}


/// The system that shows the right panel.
#[allow(clippy::too_many_arguments)]
pub fn show_right_panel(
    // Info about the application state.
    egui_ctx: Res<'_, EguiContext>,
    mut query: Query<'_, '_, &mut Concrete>,

    // The Miratope resources controlled by the right panel.
    mut element_types: ResMut<'_, ElementTypesRes>,
    mut section_direction: ResMut<'_, Vec<SectionDirection>>,
    section_state: Res<'_, SectionState>
) {
    // The right panel.
    egui::SidePanel::right("right_panel")
        .default_width(300.0)
        .max_width(450.0)
        .show(egui_ctx.ctx(), |ui| {
            
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Generate").enabled(!element_types.main)).clicked() {
                    if let Some(p) = query.iter_mut().next() {
                        element_types.main = true;
                        *element_types = element_types.from_poly(p);
                    }
                }
    
                if ui.add(egui::Button::new("Load").enabled(!element_types.main)).clicked() {
                    if let Some(mut p) = query.iter_mut().next() {
                        element_types.main = true;
                        element_types.main_updating = true;
                        *p = element_types.poly.clone();
                    }
                }
            });

            ui.separator();

            egui::containers::ScrollArea::auto_sized().show(ui, |ui| {
                for (r, types) in element_types.types.clone().into_iter().enumerate().skip(1) {
                    let poly = &element_types.poly;
                    let rank = element_types.poly.rank();

                    if r == rank {
                        break;
                    }

                    ui.heading(format!("{}", EL_NAMES[r]));
                    for t in types {
                        let i = t.example;

                        ui.horizontal(|ui| {

                            // The number of elements in this orbit
                            ui.label(format!("{} ×",t.count));

                            // Button to get the element
                            if ui.button(format!("{}-{}", 
                                t.facets,
                                EL_SUFFIXES[r],
                            )).clicked() {
                                if let Some(mut p) = query.iter_mut().next() {
                                    if let Some(mut element) = poly.element(r,i) {
                                        element.flatten();
                                        element.recenter();
                                        *p = element;
                                    } else {
                                        eprintln!("Element failed: no element at rank {}, index {}", r, i);
                                    }
                                }
                            }

                            // Button to get the element figure
                            if ui.button(format!("{}-{}",
                                t.fig_facets,
                                EL_SUFFIXES[rank - r],
                            )).clicked() {
                                if let Some(mut p) = query.iter_mut().next() {
                                    match poly.element_fig(r, i) {
                                        Ok(Some(mut figure)) => {
                                            figure.flatten();
                                            figure.recenter();
                                            *p = figure;
                                        }
                                        Ok(None) => eprintln!("Figure failed: no element at rank {}, index {}", r, i),
                                        Err(err) => eprintln!("Figure failed: {}", err),
                                    }
                                }
                            }

                            if let SectionState::Active{..} = section_state.clone() {
                                if section_direction[0].0.len() == rank-1 { // Checks if the sliced polytope and the polytope the types are of have the same rank.
                                    if ui.button("Align slice").clicked() {
                                        if let Some(element) = poly.element(r,i) {
                                            section_direction[0] = SectionDirection(Vector::from(Point::from(
                                                Subspace::from_points(element.vertices.iter())
                                                    .project(&Point::zeros(rank-1))
                                                    .normalize()
                                            )));
                                        }
                                    }
                                }
                            }

                            if let Some(radius) = t.radius {
                                ui.label(
                                    if r == 1 {format!("norm {:.10}", radius)}
                                    else if r == 2 {format!("length {:.10}", radius*2.0)}
                                    else {format!("radius {:.10}", radius)}
                                );
                            }
                        });
                    }

                    ui.separator();
                }

                ui.heading("Components");
                ui.label(format!("{} component{}",
                    element_types.components.len(),
                    if element_types.components.len() == 1 {""} else {"s"}
                ));

                for component in &element_types.components {
                    if ui.button(format!("{}-{}", 
                        if component.rank() < 1 {
                            0
                        } else {
                            component.abs[component.rank()-1].len()
                        },
                        EL_SUFFIXES[element_types.poly.rank()],
                    )).clicked() {
                        if let Some(mut p) = query.iter_mut().next() {
                            *p = component.clone();
                        }
                    }
                }

                ui.separator();
            });
    });
}