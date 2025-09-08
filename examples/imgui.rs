use std::collections::VecDeque;

use karna::{
    render::{imgui, imgui::Condition, Color, Mesh, Rect2D},
    App, Context, Scene,
};

pub struct ImguiDemo {
    player: Rect2D,
    grid: Vec<Rect2D>,
    delta_time_history: VecDeque<f32>,
    frame_time_history: VecDeque<f32>,
    fps_history: VecDeque<f32>,
    draw_calls_history: VecDeque<u32>,
    max_history_size: usize,

    // Statistics tracking
    min_frame_time: f32,
    max_frame_time: f32,
    avg_frame_time: f32,
    frame_drops: u32,
    target_frame_time: f32,

    // UI state
    show_histogram: bool,
    auto_scale: bool,
    graph_height: f32,
}

impl ImguiDemo {
    fn new() -> Self {
        Self {
            player: Rect2D::default().with_position([10, 10]).with_size(50.0),
            grid: (0..5)
                .flat_map(|i| {
                    (0..10).map(move |j| {
                        let x_t = i as f32 / 4.0;
                        let y_t = j as f32 / 9.0;

                        Rect2D::default()
                            .with_position([i as f32 * 60.0 + 200.0, j as f32 * 60.0 + 100.0])
                            .with_size(50.0)
                            .with_color(Color::rgb(x_t, y_t, 1.0 - x_t))
                    })
                })
                .collect(),
            delta_time_history: VecDeque::new(),
            frame_time_history: VecDeque::new(),
            fps_history: VecDeque::new(),
            draw_calls_history: VecDeque::new(),
            max_history_size: 240, // 4 seconds at 60fps

            min_frame_time: f32::MAX,
            max_frame_time: 0.0,
            avg_frame_time: 0.0,
            frame_drops: 0,
            target_frame_time: 1.0 / 60.0 * 1000.0,

            show_histogram: false,
            auto_scale: true,
            graph_height: 100.0,
        }
    }

    fn calculate_statistics(&mut self) {
        if self.frame_time_history.is_empty() {
            return;
        }

        let sum: f32 = self.frame_time_history.iter().sum();
        self.avg_frame_time = sum / self.frame_time_history.len() as f32;

        self.min_frame_time = self
            .frame_time_history
            .iter()
            .copied()
            .fold(f32::MAX, f32::min);
        self.max_frame_time = self.frame_time_history.iter().copied().fold(0.0, f32::max);

        self.frame_drops = self
            .frame_time_history
            .iter()
            .filter(|&&time| time > self.target_frame_time)
            .count() as u32;
    }

    fn create_histogram(&self, data: &VecDeque<f32>, bin_count: usize) -> Vec<f32> {
        if data.is_empty() {
            return vec![0.0; bin_count];
        }

        let min_val = data.iter().copied().fold(f32::MAX, f32::min);
        let max_val = data.iter().copied().fold(f32::MIN, f32::max);
        let range = max_val - min_val;

        if range == 0.0 {
            return vec![0.0; bin_count];
        }

        let mut bins = vec![0.0; bin_count];

        for &value in data {
            let bin_index = (((value - min_val) / range) * (bin_count - 1) as f32) as usize;
            let bin_index = bin_index.min(bin_count - 1);
            bins[bin_index] += 1.0;
        }

        bins
    }
}

impl Scene for ImguiDemo {
    fn load(&mut self, _ctx: &mut Context) {}

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        let delta_ms = ctx.time.delta().as_millis() as f32;
        let frame_ms = ctx.time.frame().as_millis() as f32;
        let fps = ctx.time.fps() as f32;
        let draw_calls = ctx.render.draw_calls();

        self.delta_time_history.push_back(delta_ms);
        self.frame_time_history.push_back(frame_ms);
        self.fps_history.push_back(fps);
        self.draw_calls_history.push_back(draw_calls);

        // Keep buffer size limited
        if self.delta_time_history.len() > self.max_history_size {
            self.delta_time_history.pop_front();
        }
        if self.frame_time_history.len() > self.max_history_size {
            self.frame_time_history.pop_front();
        }
        if self.fps_history.len() > self.max_history_size {
            self.fps_history.pop_front();
        }
        if self.draw_calls_history.len() > self.max_history_size {
            self.draw_calls_history.pop_front();
        }

        self.calculate_statistics();
    }

    fn render(&mut self, ctx: &mut Context) {
        let time_fps = ctx.time.fps();
        let time_ups = ctx.time.ups();
        let delta_time = ctx.time.delta();
        let frame_time = ctx.time.frame();
        let update_time = ctx.time.update();
        let draw_calls = ctx.render.draw_calls();

        ctx.render.imgui.render_frame(|ui| {
            // Main performance window with tabs
            ui.window("Performance Monitor")
                .size([700.0, 600.0], Condition::FirstUseEver)
                .position([10.0, 10.0], Condition::FirstUseEver)
                .build(|| {
                    // Fixed-width current stats to prevent jumping
                    ui.text("═══ Current Performance ═══");
                    ui.separator();

                    // Use fixed-width formatting to prevent jumping
                    let fps_color = if time_fps >= 55 {
                        [0.0, 1.0, 0.0, 1.0]
                    } else if time_fps >= 30 {
                        [1.0, 1.0, 0.0, 1.0]
                    } else {
                        [1.0, 0.0, 0.0, 1.0]
                    };

                    ui.text_colored(fps_color, format!("FPS: {:3}", time_fps));
                    ui.same_line();
                    ui.text(format!(" │ UPS: {:3}", time_ups));

                    ui.text(format!(
                        "Frame: {:5.1}ms │ Delta: {:5.1}ms │ Update: {:5.1}ms",
                        frame_time.as_secs_f32() * 1000.0,
                        delta_time.as_secs_f32() * 1000.0,
                        update_time.as_secs_f32() * 1000.0,
                    ));
                    ui.text(format!("Draw calls: {:4}", draw_calls));

                    ui.spacing();
                    ui.text(format!(
                        "Stats (last {:3} frames):",
                        self.frame_time_history.len()
                    ));
                    ui.separator();

                    ui.text(format!(
                        "Avg: {:5.1}ms │ Min: {:5.1}ms │ Max: {:5.1}ms",
                        self.avg_frame_time, self.min_frame_time, self.max_frame_time
                    ));

                    let frame_drop_percentage = if !self.frame_time_history.is_empty() {
                        (self.frame_drops as f32 / self.frame_time_history.len() as f32) * 100.0
                    } else {
                        0.0
                    };

                    let drop_color = if frame_drop_percentage < 1.0 {
                        [0.0, 1.0, 0.0, 1.0]
                    } else if frame_drop_percentage < 5.0 {
                        [1.0, 1.0, 0.0, 1.0]
                    } else {
                        [1.0, 0.0, 0.0, 1.0]
                    };

                    ui.text_colored(
                        drop_color,
                        format!(
                            "Frame drops: {:3} ({:4.1}%)",
                            self.frame_drops, frame_drop_percentage
                        ),
                    );

                    ui.spacing();

                    // Graph controls
                    ui.checkbox("Show Histograms", &mut self.show_histogram);
                    ui.same_line();
                    ui.checkbox("Auto Scale", &mut self.auto_scale);
                    ui.same_line();
                    ui.slider("Height", 50.0, 200.0, &mut self.graph_height);

                    ui.separator();

                    // Tabbed interface for different views
                    if let Some(_tab_bar) = ui.tab_bar("GraphTabs") {
                        if let Some(_tab) = ui.tab_item("Timeline") {
                            self.render_timeline_graphs(ui);
                        }

                        if let Some(_tab) = ui.tab_item("Histograms") {
                            self.render_histogram_graphs(ui);
                        }

                        if let Some(_tab) = ui.tab_item("Scatter") {
                            self.render_scatter_graphs(ui);
                        }

                        if let Some(_tab) = ui.tab_item("Comparison") {
                            self.render_comparison_graphs(ui);
                        }
                    }
                });

            // Compact HUD overlay
            ui.window("Performance HUD")
                .size([280.0, 140.0], Condition::FirstUseEver)
                .position([1000.0, 10.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text(format!(
                        "FPS: {:3} │ {:4.1}ms",
                        time_fps,
                        frame_time.as_millis()
                    ));
                    ui.text(format!(
                        "Avg: {:4.1}ms │ Max: {:4.1}ms",
                        self.avg_frame_time, self.max_frame_time
                    ));
                    ui.text(format!(
                        "Drops: {:3} │ Calls: {:3}",
                        self.frame_drops, draw_calls
                    ));

                    // Performance bar
                    let performance_ratio = (self.target_frame_time
                        / (frame_time.as_secs_f32() * 1000.0).max(0.1))
                    .min(1.0);
                    let bar_color = if performance_ratio > 0.9 {
                        [0.0, 1.0, 0.0, 1.0]
                    } else if performance_ratio > 0.6 {
                        [1.0, 1.0, 0.0, 1.0]
                    } else {
                        [1.0, 0.0, 0.0, 1.0]
                    };

                    // ui.text("Performance:");
                    // ui.progress_bar(performance_ratio as f32)
                    //     .size([250.0, 20.0])
                    //     .overlay(format!("{:.0}%", performance_ratio * 100.0).as_str())
                    //     .build();

                    // Sparkline (mini recent history)
                    if !self.frame_time_history.is_empty() {
                        let recent: Vec<f32> = self
                            .frame_time_history
                            .iter()
                            .rev()
                            .take(60)
                            .rev()
                            .copied()
                            .collect();

                        ui.plot_lines("##sparkline", &recent)
                            .scale_min(0.0)
                            .scale_max(self.target_frame_time * 2.0)
                            .graph_size([250.0, 30.0])
                            .build();
                    }
                });
        });

        // Render game objects
        self.player.render(&mut ctx.render);
        for Rect2D in &self.grid {
            Rect2D.render(&mut ctx.render);
        }
    }
}

impl ImguiDemo {
    fn render_timeline_graphs(&mut self, ui: &imgui::Ui) {
        if !self.frame_time_history.is_empty() {
            let frame_values: Vec<f32> = self.frame_time_history.iter().copied().collect();
            let max_scale = if self.auto_scale {
                self.max_frame_time * 1.1
            } else {
                50.0
            };

            ui.text("Frame Time Timeline (Lower is Better)");
            ui.plot_lines("##frame_timeline", &frame_values)
                .scale_min(0.0)
                .scale_max(max_scale)
                .graph_size([650.0, self.graph_height])
                .build();

            ui.text(format!(
                "Target: {:.1}ms │ Current range: {:.1}-{:.1}ms",
                self.target_frame_time, self.min_frame_time, self.max_frame_time
            ));
        }

        if !self.fps_history.is_empty() {
            let fps_values: Vec<f32> = self.fps_history.iter().copied().collect();

            ui.text("FPS Timeline (Higher is Better)");
            ui.plot_lines("##fps_timeline", &fps_values)
                .scale_min(0.0)
                .scale_max(if self.auto_scale {
                    self.fps_history.iter().copied().fold(0.0, f32::max) * 1.1
                } else {
                    120.0
                })
                .graph_size([650.0, self.graph_height])
                .build();
        }

        if !self.draw_calls_history.is_empty() {
            let draw_values: Vec<f32> = self.draw_calls_history.iter().map(|&x| x as f32).collect();

            ui.text("Draw Calls Timeline");
            ui.plot_lines("##draws_timeline", &draw_values)
                .scale_min(0.0)
                .scale_max(draw_values.iter().copied().fold(0.0, f32::max) * 1.1)
                .graph_size([650.0, 80.0])
                .build();
        }
    }

    fn render_histogram_graphs(&mut self, ui: &imgui::Ui) {
        ui.text("Performance Distribution (Histogram View)");
        ui.separator();

        if !self.frame_time_history.is_empty() {
            let histogram = self.create_histogram(&self.frame_time_history, 30);

            ui.text("Frame Time Distribution");
            ui.plot_histogram("##frame_hist", &histogram)
                .scale_min(0.0)
                .scale_max(histogram.iter().copied().fold(0.0, f32::max) * 1.1)
                .graph_size([650.0, self.graph_height])
                .build();

            ui.text(format!(
                "Range: {:.1}ms - {:.1}ms │ Most common around: {:.1}ms",
                self.min_frame_time, self.max_frame_time, self.avg_frame_time
            ));
        }

        if !self.fps_history.is_empty() {
            let fps_histogram = self.create_histogram(&self.fps_history, 25);

            ui.text("FPS Distribution");
            ui.plot_histogram("##fps_hist", &fps_histogram)
                .scale_min(0.0)
                .scale_max(fps_histogram.iter().copied().fold(0.0, f32::max) * 1.1)
                .graph_size([650.0, self.graph_height])
                .build();
        }
    }

    fn render_scatter_graphs(&mut self, ui: &imgui::Ui) {
        ui.text("Frame Time vs Draw Calls (Scatter Plot)");
        ui.separator();

        if !self.frame_time_history.is_empty() && !self.draw_calls_history.is_empty() {
            // Create correlation data (simulated scatter plot using line plot)
            let correlation_data: Vec<f32> = self
                .frame_time_history
                .iter()
                .zip(self.draw_calls_history.iter())
                .map(|(&frame_time, &draw_calls)| {
                    // Normalize draw calls to frame time scale for visualization
                    frame_time + (draw_calls as f32 * 0.1)
                })
                .collect();

            ui.text("Correlation: Frame Time + Draw Call Impact");
            ui.plot_lines("##correlation", &correlation_data)
                .scale_min(0.0)
                .scale_max(correlation_data.iter().copied().fold(0.0, f32::max) * 1.1)
                .graph_size([650.0, self.graph_height])
                .build();

            ui.text("Higher values may indicate draw call impact on frame time");
        }

        // Frame time variance over time
        if self.frame_time_history.len() > 10 {
            let variance_data: Vec<f32> = self
                .frame_time_history
                .iter()
                .enumerate()
                .skip(5)
                .map(|(i, _)| {
                    let window = &self.frame_time_history.as_slices().0
                        [i - 5..=i.min(self.frame_time_history.len() - 1)];
                    let avg = window.iter().sum::<f32>() / window.len() as f32;
                    let variance =
                        window.iter().map(|x| (x - avg).powi(2)).sum::<f32>() / window.len() as f32;
                    variance.sqrt() // Standard deviation
                })
                .collect();

            ui.text("Frame Time Stability (Standard Deviation)");
            ui.plot_lines("##stability", &variance_data)
                .scale_min(0.0)
                .scale_max(variance_data.iter().copied().fold(0.0, f32::max) * 1.1)
                .graph_size([650.0, 80.0])
                .build();

            ui.text("Lower values = more stable performance");
        }
    }

    fn render_comparison_graphs(&mut self, ui: &imgui::Ui) {
        ui.text("Performance Comparison");
        ui.separator();

        // Side-by-side comparison of recent vs older performance
        if self.frame_time_history.len() > 60 {
            let mid_point = self.frame_time_history.len() / 2;
            let recent: Vec<f32> = self
                .frame_time_history
                .iter()
                .skip(mid_point)
                .copied()
                .collect();
            let older: Vec<f32> = self
                .frame_time_history
                .iter()
                .take(mid_point)
                .copied()
                .collect();

            let recent_avg = recent.iter().sum::<f32>() / recent.len() as f32;
            let older_avg = older.iter().sum::<f32>() / older.len() as f32;

            ui.text("Recent Performance");
            ui.plot_lines("##recent", &recent)
                .scale_min(0.0)
                .scale_max(self.max_frame_time * 1.1)
                .graph_size([650.0, 80.0])
                .build();

            ui.text("Earlier Performance");
            ui.plot_lines("##older", &older)
                .scale_min(0.0)
                .scale_max(self.max_frame_time * 1.1)
                .graph_size([650.0, 80.0])
                .build();

            let improvement = older_avg - recent_avg;
            let color = if improvement > 0.0 {
                [0.0, 1.0, 0.0, 1.0]
            } else {
                [1.0, 0.0, 0.0, 1.0]
            };

            ui.text_colored(
                color,
                format!(
                    "Performance change: {:.2}ms ({})",
                    improvement.abs(),
                    if improvement > 0.0 {
                        "IMPROVED"
                    } else {
                        "DEGRADED"
                    }
                ),
            );
        }

        // Performance rating
        let rating = if self.avg_frame_time < self.target_frame_time * 0.8 {
            "EXCELLENT"
        } else if self.avg_frame_time < self.target_frame_time {
            "GOOD"
        } else if self.avg_frame_time < self.target_frame_time * 1.5 {
            "FAIR"
        } else {
            "POOR"
        };

        ui.spacing();
        ui.text(format!("Overall Performance Rating: {}", rating));
    }
}

fn main() {
    App::new()
        .with_size((1280, 720))
        .with_scene("default", ImguiDemo::new())
        .run()
        .expect("Failed to run app");
}
