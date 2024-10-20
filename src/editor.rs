use crate::util;
use nih_plug::prelude::{AtomicF32, Editor};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::Del2Params;
use crate::DelayData;
use crate::DelayDataOutput;

const COLUMN_WIDTH: Units = Pixels(269.0);

#[derive(Lens, Clone)]
pub(crate) struct Data {
    pub(crate) params: Arc<Del2Params>,
    pub(crate) delay_data: Arc<Mutex<DelayDataOutput>>,
    pub input_meter: Arc<AtomicF32>,
    pub output_meter: Arc<AtomicF32>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (1200, 450))
}

pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        // Add the stylesheet to the app
        cx.add_stylesheet(include_style!("src/style.css"))
            .expect("Failed to load stylesheet");

        editor_data.clone().build(cx);

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(cx, "global").class("global-title");
                HStack::new(cx, |cx| {
                    make_column(cx, "gain", |cx| {
                        let gain_params = Data::params.map(|p| p.global.gain_params.clone());
                        GenericUi::new_custom(cx, gain_params, |cx, param_ptr| {
                            HStack::new(cx, |cx| {
                                Label::new(cx, unsafe { param_ptr.name() }).class("label");
                                GenericUi::draw_widget(cx, gain_params, param_ptr);
                            })
                            .class("row");
                        });
                    });

                    make_column(cx, "timing", |cx| {
                        let timing_params = Data::params.map(|p| p.global.timing_params.clone());
                        GenericUi::new_custom(cx, timing_params, |cx, param_ptr| {
                            let param_name = unsafe { param_ptr.name() };

                            // Check if the parameter is `max_tap_seconds` and replace with specific logic
                            if param_name == "max tap" {
                                HStack::new(cx, |cx| {
                                    Label::new(cx, param_name).class("label");
                                    ParamSlider::new(cx, Data::params, |params| {
                                        &params.global.timing_params.max_tap_seconds
                                    })
                                    .set_style(ParamSliderStyle::FromLeft)
                                    .class("widget");
                                })
                                .class("row");
                            } else {
                                // Default widget drawing for others
                                HStack::new(cx, |cx| {
                                    Label::new(cx, param_name).class("label");
                                    GenericUi::draw_widget(cx, timing_params, param_ptr);
                                })
                                .class("row");
                            }
                        });
                    });
                });
                Label::new(cx, "filters").class("dsp-title");

                HStack::new(cx, |cx| {
                    make_column(cx, "velocity tracking", |cx| {
                        HStack::new(cx, |cx| {
                            Label::new(cx, "cutoff").class("slider-label");
                            ParamSlider::new(cx, Data::params, |params| {
                                &params.taps.velocity_to_cutoff_amount
                            })
                            .class("widget");
                        })
                        .class("row");
                    });
                    make_column(cx, "note tracking", |cx| {
                        HStack::new(cx, |cx| {
                            Label::new(cx, "cutoff").class("slider-label");
                            ParamSlider::new(cx, Data::params, |params| {
                                &params.taps.note_to_cutoff_amount
                            })
                            .class("widget");
                        })
                        .class("row");
                    });
                })
                .class("cutoff-tracking");

                HStack::new(cx, |cx| {
                    make_column(cx, "low velocity", |cx| {
                        let velocity_low_params = Data::params.map(|p| p.taps.velocity_low.clone());
                        GenericUi::new_custom(cx, velocity_low_params, |cx, param_ptr| {
                            HStack::new(cx, |cx| {
                                Label::new(
                                    cx,
                                    unsafe { param_ptr.name() }
                                        .strip_prefix("low velocity ")
                                        .expect("Expected parameter name prefix, this is a bug"),
                                )
                                .class("label");

                                GenericUi::draw_widget(cx, velocity_low_params, param_ptr);
                            })
                            .class("row");
                        });
                    });

                    make_column(cx, "high velocity", |cx| {
                        let velocity_high_params =
                            Data::params.map(|p| p.taps.velocity_high.clone());
                        GenericUi::new_custom(cx, velocity_high_params, |cx, param_ptr| {
                            HStack::new(cx, |cx| {
                                Label::new(
                                    cx,
                                    unsafe { param_ptr.name() }
                                        .strip_prefix("high velocity ")
                                        .expect("Expected parameter name prefix, this is a bug"),
                                )
                                .class("label");

                                GenericUi::draw_widget(cx, velocity_high_params, param_ptr);
                            })
                            .class("row");
                        });
                    });
                })
                .size(Auto);
            })
            .class("parameters");
            // .row_between(Pixels(9.0))
            // .child_left(Stretch(1.0))
            // .child_right(Stretch(1.0))
            // .child_left(Pixels(9.0))
            // .child_right(Pixels(9.0));
            VStack::new(cx, |cx| {
                ZStack::new(cx, |cx| {
                    Label::new(cx, "DEL2").class("plugin-name");
                    DelayGraph::new(cx, Data::delay_data);
                });
                VStack::new(cx, |cx| {
                    //meters
                    HStack::new(cx, |cx| {
                        PeakMeter::new(
                            cx,
                            Data::input_meter.map(|input_meter| {
                                util::gain_to_db(input_meter.load(Ordering::Relaxed))
                            }),
                            Some(Duration::from_millis(600)),
                        );
                        Label::new(cx, "input").class("meter-label");
                    });
                    HStack::new(cx, |cx| {
                        PeakMeter::new(
                            cx,
                            Data::output_meter.map(|output_meter| {
                                util::gain_to_db(output_meter.load(Ordering::Relaxed))
                            }),
                            Some(Duration::from_millis(600)),
                        );
                        Label::new(cx, "output").class("meter-label");
                    });
                })
                .class("meters_and_name");
                ResizeHandle::new(cx);
            });
        });
    })
}

///////////////////////////////////////////////////////////////////////////////
//                             DelayGraph                            //
///////////////////////////////////////////////////////////////////////////////

pub struct DelayGraph {
    delay_data: Arc<Mutex<DelayDataOutput>>,
}

impl DelayGraph {
    pub fn new<DelayDataL>(cx: &mut Context, delay_data: DelayDataL) -> Handle<Self>
    where
        DelayDataL: Lens<Target = Arc<Mutex<DelayDataOutput>>>,
    {
        Self {
            delay_data: delay_data.get(cx),
        }
        .build(cx, |_cx| {
            // put other widgets here
        })
    }
}

// TODO: add grid to show bars & beats
impl View for DelayGraph {
    // For CSS:
    fn element(&self) -> Option<&'static str> {
        Some("delay-graph")
    }

    fn draw(&self, draw_context: &mut DrawContext, canvas: &mut Canvas) {
        let mut locked_delay_data = self.delay_data.lock().unwrap();
        let delay_data = locked_delay_data.read();

        let bounding_box = draw_context.bounds();

        let background_color: vg::Color = draw_context.background_color().into();
        let border_color: vg::Color = draw_context.border_color().into();
        let outline_color: vg::Color = draw_context.outline_color().into();
        let selection_color: vg::Color = draw_context.selection_color().into();
        let border_width = draw_context.border_width();
        let path_line_width = draw_context.outline_width();

        // Compute the time scaling factor
        let time_scaling_factor = self.compute_time_scaling_factor(
            &delay_data,
            bounding_box.w,
            border_width,
            path_line_width,
        );

        // Draw components
        self.draw_background(canvas, bounding_box, background_color, border_width);
        self.draw_delay_times_as_lines(
            canvas,
            &delay_data,
            bounding_box,
            border_color,
            1.0,
            time_scaling_factor,
        );
        self.draw_time_line(
            canvas,
            &delay_data,
            bounding_box,
            selection_color,
            path_line_width,
            time_scaling_factor,
            border_width,
        );
        self.draw_tap_velocities(
            canvas,
            &delay_data,
            bounding_box,
            outline_color,
            path_line_width,
            time_scaling_factor,
            border_width,
        );
        self.draw_tap_notes_as_diamonds(
            canvas,
            &delay_data,
            bounding_box,
            selection_color,
            path_line_width,
            time_scaling_factor,
            border_width,
            true,
        );
        self.draw_bounding_outline(canvas, bounding_box, border_color, border_width);
    }
}

// Functional Breakdown
impl DelayGraph {
    fn compute_time_scaling_factor(
        &self,
        delay_data: &DelayData,
        rect_width: f32,
        border_width: f32,
        path_line_width: f32,
    ) -> f32 {
        let max_delay_time = if delay_data.current_tap > 0 {
            delay_data.delay_times[delay_data.current_tap - 1]
        } else {
            0
        };
        ((max_delay_time as f32 + delay_data.max_tap_samples as f32)
            / (rect_width - border_width - path_line_width * 0.5))
            .recip()
    }

    fn draw_delay_times_as_lines(
        &self,
        canvas: &mut Canvas,
        delay_data: &DelayData,
        bounds: BoundingBox,
        border_color: vg::Color,
        border_width: f32,
        time_scaling_factor: f32,
    ) {
        let mut path = vg::Path::new();

        for i in 0..delay_data.current_tap {
            // Combine delay time with time scaling factor for correct horizontal scaling
            let x_offset =
                delay_data.delay_times[i] as f32 * time_scaling_factor + border_width * 0.5;

            // Line from bottom to top border considering border thickness
            let start_y = bounds.y + bounds.h - border_width * 0.5;
            let end_y = bounds.y + border_width * 0.5;

            path.move_to(bounds.x + x_offset, start_y);
            path.line_to(bounds.x + x_offset, end_y);
        }

        canvas.stroke_path(
            &path,
            &vg::Paint::color(border_color).with_line_width(border_width),
        );
    }

    fn draw_background(
        &self,
        canvas: &mut Canvas,
        bounds: BoundingBox,
        color: vg::Color,
        border_width: f32,
    ) {
        let mut path = vg::Path::new();
        path.rect(
            bounds.x + border_width * 0.5,
            bounds.y,
            bounds.w - border_width,
            bounds.h - border_width * 0.5,
        );
        path.close();

        let paint = vg::Paint::color(color);
        canvas.fill_path(&path, &paint);
    }

    fn draw_time_line(
        &self,
        canvas: &mut Canvas,
        delay_data: &DelayData,
        bounds: BoundingBox,
        color: vg::Color,
        line_width: f32,
        scaling_factor: f32,
        border_width: f32,
    ) {
        let max_delay_time = if delay_data.current_tap > 0 {
            delay_data.delay_times[delay_data.current_tap - 1]
        } else {
            0
        };
        if delay_data.current_time > max_delay_time {
            let x_offset = delay_data.current_time as f32 * scaling_factor + border_width * 0.5;
            let mut path = vg::Path::new();
            path.move_to(
                bounds.x + x_offset,
                bounds.y + bounds.h - border_width * 0.5,
            );
            path.line_to(bounds.x + x_offset, bounds.y);
            path.close();

            canvas.stroke_path(&path, &vg::Paint::color(color).with_line_width(line_width));
        }
    }

    fn draw_tap_velocities(
        &self,
        canvas: &mut Canvas,
        delay_data: &DelayData,
        bounds: BoundingBox,
        color: vg::Color,
        line_width: f32,
        scaling_factor: f32,
        border_width: f32,
    ) {
        let mut path = vg::Path::new();
        for i in 0..delay_data.current_tap {
            let x_offset = delay_data.delay_times[i] as f32 * scaling_factor + border_width * 0.5;
            let velocity_height = (bounds.h - border_width * 0.5)
                - (delay_data.velocities[i] * (bounds.h - border_width * 0.5));

            path.move_to(
                bounds.x + x_offset,
                bounds.y + bounds.h - border_width * 0.5,
            );
            path.line_to(bounds.x + x_offset, bounds.y + velocity_height);
        }

        canvas.stroke_path(&path, &vg::Paint::color(color).with_line_width(line_width));
    }

    fn draw_tap_notes_as_diamonds(
        &self,
        canvas: &mut Canvas,
        delay_data: &DelayData,
        bounds: BoundingBox,
        color: vg::Color,
        line_width: f32,
        scaling_factor: f32,
        border_width: f32,
        zoomed: bool,
    ) {
        let mut path = vg::Path::new();

        // Determine the min and max note values if zoomed
        let (min_note_value, max_note_value) = if zoomed {
            let used_notes = &delay_data.notes[0..delay_data.current_tap];
            let min = used_notes.iter().copied().min().unwrap_or(0);
            let max = used_notes.iter().copied().max().unwrap_or(127);
            (min as f32, max as f32)
        } else {
            (0.0, 127.0)
        };

        let diamond_size = line_width * 2.0; // Width and height of a diamond
        let available_height = bounds.h - border_width - 2.0 * diamond_size;

        for i in 0..delay_data.current_tap {
            let x_offset = delay_data.delay_times[i] as f32 * scaling_factor + border_width * 0.5;

            // Adjust note height to scale within bounds
            let normalized_note = if max_note_value != min_note_value {
                (delay_data.notes[i] as f32 - min_note_value) / (max_note_value - min_note_value)
            } else {
                delay_data.notes[i] as f32 / 127.0
            };

            // Use scaling to ensure the diamond fits within the available height
            let note_height = diamond_size + (1.0 - normalized_note) * available_height;

            let diamond_center_x = bounds.x + x_offset;
            let diamond_center_y = bounds.y + note_height;

            let diamond_half_size = line_width;

            // Form a diamond shape, fully scaled
            path.move_to(diamond_center_x + diamond_half_size, diamond_center_y);
            path.line_to(diamond_center_x, diamond_center_y + diamond_half_size);
            path.line_to(diamond_center_x - diamond_half_size, diamond_center_y);
            path.line_to(diamond_center_x, diamond_center_y - diamond_half_size);
            path.close();
        }

        canvas.stroke_path(&path, &vg::Paint::color(color).with_line_width(line_width));
    }
    // TODO: .overflow(Overflow::Visible);

    fn u8_note_to_name(note_nr: u8) -> String {
        let note_name = util::NOTES[(note_nr % 12) as usize];
        let octave = (note_nr / 12) - 1;
        format!("{note_name}{octave}")
    }

    fn draw_bounding_outline(
        &self,
        canvas: &mut Canvas,
        bounds: BoundingBox,
        color: vg::Color,
        border_width: f32,
    ) {
        let mut path = vg::Path::new();
        path.rect(
            bounds.x + border_width * 0.5,
            bounds.y,
            bounds.w - border_width,
            bounds.h - border_width * 0.5,
        );
        path.close();

        canvas.stroke_path(
            &path,
            &vg::Paint::color(color).with_line_width(border_width),
        );
    }
}

fn make_column(cx: &mut Context, title: &str, contents: impl FnOnce(&mut Context)) {
    VStack::new(cx, |cx| {
        Label::new(cx, title).class("column-title");

        contents(cx);
    })
    // .class("column");
    .width(COLUMN_WIDTH)
    .height(Auto);
}
