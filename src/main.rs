#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

// ============================================================
// COLORS
// ============================================================

const BACKGROUND: egui::Color32 =
    egui::Color32::from_rgb(8, 9, 12);

const GLASS_BORDER: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(24, 24, 28, 255);

const TEXT: egui::Color32 =
    egui::Color32::from_rgb(245, 245, 247);

const SECONDARY_TEXT: egui::Color32 =
    egui::Color32::from_rgb(145, 145, 155);

const BUTTON_BORDER: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(25, 25, 29, 255);

const OPERATOR: egui::Color32 =
    egui::Color32::from_rgb(255, 159, 10);

const OPERATOR_BACKGROUND: egui::Color32 =
    egui::Color32::from_rgb(45, 31, 15);


// ============================================================
// COLORS WITH ALPHA
// ============================================================

fn glass_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        255, 255, 255, 10,
    )
}

fn button_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        255, 255, 255, 16,
    )
}

fn button_hover_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        255, 255, 255, 28,
    )
}

fn button_pressed_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        255, 255, 255, 40,
    )
}

fn operator_hover_color() -> egui::Color32 {
    egui::Color32::from_rgb(58, 40, 17)
}

fn operator_pressed_color() -> egui::Color32 {
    egui::Color32::from_rgb(72, 47, 16)
}


// ============================================================
// MAIN
// ============================================================

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(
        include_bytes!("../assets/icon.png"),
    )
    .expect("Failed to load application icon");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([430.0, 760.0])
            .with_min_inner_size([380.0, 760.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Liquid Calculator",
        options,
        Box::new(|_cc| Ok(Box::new(Calculator::default()))),
    )
}


// ============================================================
// CALCULATOR
// ============================================================

struct Calculator {
    display: String,

    stored_value: Option<f64>,

    pending_operation: Option<Operation>,

    last_operation: Option<Operation>,

    last_operand: Option<f64>,

    waiting_for_number: bool,

    expression: String,

    keyboard_flash: Option<(KeyAction, f64)>,

    history: Vec<(String, String)>,
    history_open: bool,
}

#[derive(Clone, Copy)]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}


impl Default for Calculator {
    fn default() -> Self {
        Self {
            display: "0".to_string(),
            stored_value: None,
            pending_operation: None,
            last_operation: None,
            last_operand: None,
            waiting_for_number: false,
            expression: String::new(),
            keyboard_flash: None,
            history: Vec::new(),
            history_open: false,
        }
    }
}


// ============================================================
// CALCULATOR LOGIC
// ============================================================

impl Calculator {

    // --------------------------------------------------------
    // DIGITS
    // --------------------------------------------------------

    fn input_digit(&mut self, digit: char) {
        if self.waiting_for_number {
            self.display.clear();
            self.waiting_for_number = false;
        }

        if self.display == "0" {
            self.display.clear();
        }

        self.display.push(digit);

        self.update_expression();
    }


    // --------------------------------------------------------
    // DECIMAL
    // --------------------------------------------------------

    fn input_decimal(&mut self) {
        if self.waiting_for_number {
            self.display = "0".to_string();
            self.waiting_for_number = false;
        }

        if !self.display.contains('.') {
            self.display.push('.');
        }

        self.update_expression();
    }


    // --------------------------------------------------------
    // CLEAR
    // --------------------------------------------------------

    fn clear(&mut self) {
        self.display = "0".to_string();

        self.stored_value = None;
        self.pending_operation = None;

        self.last_operation = None;
        self.last_operand = None;

        self.waiting_for_number = false;
        self.expression.clear();
        self.keyboard_flash = None;
    }


    // --------------------------------------------------------
    // PLUS / MINUS
    // --------------------------------------------------------

    fn toggle_sign(&mut self) {
        if self.display == "0" {
            return;
        }

        if self.display.starts_with('-') {
            self.display.remove(0);
        } else {
            self.display.insert(0, '-');
        }

        self.update_expression();
    }


    // --------------------------------------------------------
    // BACKSPACE
    // --------------------------------------------------------

    fn delete_last(&mut self) {
        if self.waiting_for_number {
            return;
        }

        self.display.pop();

        if self.display.is_empty() || self.display == "-" {
            self.display = "0".to_string();
        }

        self.update_expression();
    }


    // --------------------------------------------------------
    // OPERATION
    // --------------------------------------------------------

    fn set_operation(&mut self, operation: Operation) {
        self.last_operation = None;
        self.last_operand = None;

        let current = match self.display.parse::<f64>() {
            Ok(value) => value,
            Err(_) => return,
        };

        // If an operation is already pending and the user
        // has entered the second number, calculate it first.
        //
        // Example:
        //
        // 5 × 5 + 6
        //
        // When "+" is pressed, we calculate 5 × 5 first.
        if let (Some(stored), Some(previous_operation)) =
            (self.stored_value, self.pending_operation)
        {
            if !self.waiting_for_number {
                let result =
                    calculate(stored, current, previous_operation);

                self.display = format_number(result);
                self.stored_value = Some(result);
            }
        } else {
            self.stored_value = Some(current);
        }

        self.pending_operation = Some(operation);
        self.waiting_for_number = true;

        let symbol = operation_symbol(operation);

        self.expression =
            format!("{} {}", self.display, symbol);
    }


    // --------------------------------------------------------
    // EQUALS
    // --------------------------------------------------------

    fn calculate_result(&mut self) {
    // --------------------------------------------------------
    // Normal calculation:
    //
    // 10 × 10 =
    //
    // --------------------------------------------------------

    if let (Some(stored), Some(operation)) =
        (self.stored_value, self.pending_operation)
    {
        let current = match self.display.parse::<f64>() {
            Ok(value) => value,
            Err(_) => return,
        };

        let result = calculate(
            stored,
            current,
            operation,
        );

        let symbol = operation_symbol(operation);

        self.expression = format!(
            "{} {} {} =",
            format_number(stored),
            symbol,
            format_number(current),
        );

        self.display = format_number(result);

        self.add_history(
            self.expression.clone(),
            self.display.clone(),
        );

        //
        //
        // 10 × 10 =
        //       =
        //       =
        //
        // 100
        // 1000
        // 10000
        self.last_operation = Some(operation);
        self.last_operand = Some(current);

        self.stored_value = None;
        self.pending_operation = None;

        self.waiting_for_number = true;

        return;
    }


    // --------------------------------------------------------
    // Repeat last operation.
    //
    //
    // 10 × 10 =
    //
    // stored result = 100
    // last operation = ×
    // last operand = 10
    //
    //
    // 100 × 10 = 1000
    // --------------------------------------------------------

    if let (Some(operation), Some(operand)) =
        (self.last_operation, self.last_operand)
    {
        let current = match self.display.parse::<f64>() {
            Ok(value) => value,
            Err(_) => return,
        };

        let result = calculate(
            current,
            operand,
            operation,
        );

        let symbol = operation_symbol(operation);

        self.expression = format!(
            "{} {} {} =",
            format_number(current),
            symbol,
            format_number(operand),
        );

        self.display = format_number(result);

        self.add_history(
            self.expression.clone(),
            self.display.clone(),
        );

        self.waiting_for_number = true;
    }
}


    // --------------------------------------------------------
    // EXPRESSION
    // --------------------------------------------------------

    fn add_history(&mut self, expression: String, result: String) {
        self.history.insert(0, (expression, result));

        if self.history.len() > 20 {
            self.history.pop();
        }
    }


    fn update_expression(&mut self) {
        if let (Some(stored), Some(operation)) =
            (self.stored_value, self.pending_operation)
        {
            let symbol = operation_symbol(operation);

            self.expression = format!(
                "{} {} {}",
                format_number(stored),
                symbol,
                self.display
            );
        } else {
            self.expression.clear();
        }
    }
}


// ============================================================
// KEYBOARD
// ============================================================

impl Calculator {
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        let mut actions = Vec::new();

        ctx.input(|input| {
            for event in &input.events {
                match event {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        ..
                    } => {
                        if let Some(action) = key_action(*key) {
                            actions.push(action);
                        }
                    }

                    egui::Event::Text(text) => {
                        for ch in text.chars() {
                            if ch == '*' {
                                actions.push(KeyAction::Multiply);
                            }
                        }
                    }

                    _ => {}
                }
            }
        });

        if !actions.is_empty() {
            let now = ctx.input(|input| input.time);
            self.keyboard_flash = Some((*actions.last().unwrap(), now + 0.10));
        }

        for action in actions {
            self.apply_key_action(action);
        }

        if let Some((_, until)) = self.keyboard_flash {
            let now = ctx.input(|input| input.time);

            if now < until {
                ctx.request_repaint();
            } else {
                self.keyboard_flash = None;
            }
        }
    }

    fn apply_key_action(&mut self, action: KeyAction) {
        match action {
            KeyAction::Digit(digit) => self.input_digit(digit),
            KeyAction::Decimal => self.input_decimal(),
            KeyAction::Add => self.set_operation(Operation::Add),
            KeyAction::Subtract => {
                self.set_operation(Operation::Subtract)
            }
            KeyAction::Multiply => {
                self.set_operation(Operation::Multiply)
            }
            KeyAction::Divide => {
                self.set_operation(Operation::Divide)
            }
            KeyAction::Equals => self.calculate_result(),
            KeyAction::Backspace => self.delete_last(),
            KeyAction::Clear => self.clear(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum KeyAction {
    Digit(char),
    Decimal,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    Backspace,
    Clear,
}

fn key_action(key: egui::Key) -> Option<KeyAction> {
    match key {
        egui::Key::Num0 => Some(KeyAction::Digit('0')),
        egui::Key::Num1 => Some(KeyAction::Digit('1')),
        egui::Key::Num2 => Some(KeyAction::Digit('2')),
        egui::Key::Num3 => Some(KeyAction::Digit('3')),
        egui::Key::Num4 => Some(KeyAction::Digit('4')),
        egui::Key::Num5 => Some(KeyAction::Digit('5')),
        egui::Key::Num6 => Some(KeyAction::Digit('6')),
        egui::Key::Num7 => Some(KeyAction::Digit('7')),
        egui::Key::Num8 => Some(KeyAction::Digit('8')),
        egui::Key::Num9 => Some(KeyAction::Digit('9')),

        egui::Key::Period => Some(KeyAction::Decimal),
        egui::Key::Plus => Some(KeyAction::Add),
        egui::Key::Minus => Some(KeyAction::Subtract),
        egui::Key::Slash => Some(KeyAction::Divide),

        egui::Key::Equals => Some(KeyAction::Equals),
        egui::Key::Enter => Some(KeyAction::Equals),

        egui::Key::Backspace => Some(KeyAction::Backspace),

        egui::Key::Escape => Some(KeyAction::Clear),
        egui::Key::Delete => Some(KeyAction::Clear),

        egui::Key::C => Some(KeyAction::Clear),

        _ => None,
    }
}


// ============================================================
// UI
// ============================================================

impl eframe::App for Calculator {

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {

        self.handle_keyboard(ui.ctx());

        // ----------------------------------------------------
        // BACKGROUND
        // ----------------------------------------------------

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(BACKGROUND)
                    .inner_margin(18.0),
            )
            .show(ui, |ui| {

                // ------------------------------------------------
                // GLASS CARD
                // ------------------------------------------------

                egui::Frame::NONE
                    .fill(glass_color())
                    .stroke(
                        egui::Stroke::new(
                            1.0,
                            GLASS_BORDER,
                        ),
                    )
                    .corner_radius(
                        egui::CornerRadius::same(30),
                    )
                    .inner_margin(22.0)
                    .show(ui, |ui| {

                        // ========================================
                        // ========================================
                        // HEADER
                        // ========================================

                        ui.horizontal(|ui| {

                            ui.label(
                                egui::RichText::new("LIQUID")
                                    .size(12.0)
                                    .strong()
                                    .color(SECONDARY_TEXT),
                            );

                            ui.label(
                                egui::RichText::new("CALCULATOR")
                                    .size(12.0)
                                    .color(SECONDARY_TEXT),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(
                                    egui::Align::Center,
                                ),
                                |ui| {
                                    let icon = if self.history_open {
                                        "CLOSE"
                                    } else {
                                        "HISTORY"
                                    };

                                    if self.history_button(
                                        ui,
                                        icon,
                                        34.0,
                                        30.0,
                                    ) {
                                        self.history_open =
                                            !self.history_open;
                                    }
                                },
                            );
                        });


                        ui.add_space(10.0);


                        // ========================================
                        // HISTORY
                        // ========================================

                        if self.history_open {

                            egui::Frame::NONE
                                .fill(
                                    egui::Color32::from_rgba_unmultiplied(
                                        255,
                                        255,
                                        255,
                                        7,
                                    ),
                                )
                                .corner_radius(
                                    egui::CornerRadius::same(18),
                                )
                                .inner_margin(12.0)
                                .show(ui, |ui| {

                                    ui.horizontal(|ui| {

                                        ui.label(
                                            egui::RichText::new("HISTORY")
                                                .size(11.0)
                                                .strong()
                                                .color(SECONDARY_TEXT),
                                        );

                                        ui.with_layout(
                                            egui::Layout::right_to_left(
                                                egui::Align::Center,
                                            ),
                                            |ui| {
                                                if !self.history.is_empty()
                                                    && ui.small_button("Clear").clicked()
                                                {
                                                    self.history.clear();
                                                }
                                            },
                                        );
                                    });

                                    ui.add_space(5.0);

                                    if self.history.is_empty() {

                                        ui.label(
                                            egui::RichText::new(
                                                "No calculations yet",
                                            )
                                            .size(13.0)
                                            .color(SECONDARY_TEXT),
                                        );

                                    } else {

                                        egui::ScrollArea::vertical()
                                            .max_height(125.0)
                                            .show(ui, |ui| {

                                                for (expression, result)
                                                    in &self.history
                                                {
                                                    ui.horizontal(|ui| {

                                                        ui.label(
                                                            egui::RichText::new(
                                                                expression,
                                                            )
                                                            .size(13.0)
                                                            .color(SECONDARY_TEXT),
                                                        );

                                                        ui.with_layout(
                                                            egui::Layout::right_to_left(
                                                                egui::Align::Center,
                                                            ),
                                                            |ui| {
                                                                ui.label(
                                                                    egui::RichText::new(
                                                                        result,
                                                                    )
                                                                    .size(15.0)
                                                                    .strong()
                                                                    .color(TEXT),
                                                                );
                                                            },
                                                        );
                                                    });

                                                    ui.add_space(4.0);
                                                }
                                            });
                                    }
                                });

                            ui.add_space(10.0);
                        }


          // DISPLAY
                        // ========================================

                        ui.allocate_ui_with_layout(
                            egui::vec2(
                                ui.available_width(),
                                160.0,
                            ),

                            egui::Layout::bottom_up(
                                egui::Align::RIGHT,
                            ),

                            |ui| {

                                // Expression
                                if !self.expression.is_empty() {

                                    ui.label(
                                        egui::RichText::new(
                                            &self.expression,
                                        )
                                        .size(15.0)
                                        .color(
                                            SECONDARY_TEXT,
                                        ),
                                    );

                                    ui.add_space(7.0);
                                }


                                // Main number
                                ui.label(
                                    egui::RichText::new(
                                        &self.display,
                                    )
                                    .size(58.0)
                                    .strong()
                                    .color(TEXT),
                                );
                            },
                        );


                        ui.add_space(12.0);


                        // ========================================
                        // BUTTON GRID
                        // ========================================

                        let gap = 10.0;

                        let width =
                            ui.available_width();

                        let button_width =
                            (width - gap * 3.0) / 4.0;

                        let button_height = 66.0;


                        // ========================================
                        // ROW 1
                        // ========================================

                        ui.horizontal(|ui| {

                            ui.spacing_mut()
                                .item_spacing
                                .x = gap;


                            if self.action_button(
                                ui,
                                "AC",
                                button_width,
                                button_height,
                            ) {
                                self.clear();
                            }


                            if self.action_button(
                                ui,
                                "BACKSPACE",
                                button_width,
                                button_height,
                            ) {
                                self.delete_last();
                            }


                            if self.action_button(
                                ui,
                                "±",
                                button_width,
                                button_height,
                            ) {
                                self.toggle_sign();
                            }


                            if self.operator_button(
                                ui,
                                "÷",
                                button_width,
                                button_height,
                            ) {
                                self.set_operation(
                                    Operation::Divide,
                                );
                            }
                        });


                        // ========================================
                        // ROW 2
                        // ========================================

                        ui.horizontal(|ui| {

                            ui.spacing_mut()
                                .item_spacing
                                .x = gap;


                            if self.number_button(
                                ui,
                                "7",
                                button_width,
                                button_height,
                            ) {
                                self.input_digit('7');
                            }


                            if self.number_button(
                                ui,
                                "8",
                                button_width,
                                button_height,
                            ) {
                                self.input_digit('8');
                            }


                            if self.number_button(
                                ui,
                                "9",
                                button_width,
                                button_height,
                            ) {
                                self.input_digit('9');
                            }


                            if self.operator_button(
                                ui,
                                "×",
                                button_width,
                                button_height,
                            ) {
                                self.set_operation(
                                    Operation::Multiply,
                                );
                            }
                        });


                        // ========================================
                        // ROW 3
                        // ========================================

                        ui.horizontal(|ui| {

                            ui.spacing_mut()
                                .item_spacing
                                .x = gap;


                            if self.number_button(
                                ui,
                                "4",
                                button_width,
                                button_height,
                            ) {
                                self.input_digit('4');
                            }


                            if self.number_button(
                                ui,
                                "5",
                                button_width,
                                button_height,
                            ) {
                                self.input_digit('5');
                            }


                            if self.number_button(
                                ui,
                                "6",
                                button_width,
                                button_height,
                            ) {
                                self.input_digit('6');
                            }


                            if self.operator_button(
                                ui,
                                "−",
                                button_width,
                                button_height,
                            ) {
                                self.set_operation(
                                    Operation::Subtract,
                                );
                            }
                        });


                        // ========================================
                        // ROW 4
                        // ========================================

                        ui.horizontal(|ui| {

                            ui.spacing_mut()
                                .item_spacing
                                .x = gap;


                            if self.number_button(
                                ui,
                                "1",
                                button_width,
                                button_height,
                            ) {
                                self.input_digit('1');
                            }


                            if self.number_button(
                                ui,
                                "2",
                                button_width,
                                button_height,
                            ) {
                                self.input_digit('2');
                            }


                            if self.number_button(
                                ui,
                                "3",
                                button_width,
                                button_height,
                            ) {
                                self.input_digit('3');
                            }


                            if self.operator_button(
                                ui,
                                "+",
                                button_width,
                                button_height,
                            ) {
                                self.set_operation(
                                    Operation::Add,
                                );
                            }
                        });


                        // ========================================
                        // ROW 5
                        // ========================================

                        ui.horizontal(|ui| {

                            ui.spacing_mut()
                                .item_spacing
                                .x = gap;


                            let zero_width =
                                button_width * 2.0 + gap;


                            if self.number_button(
                                ui,
                                "0",
                                zero_width,
                                button_height,
                            ) {
                                self.input_digit('0');
                            }


                            if self.number_button(
                                ui,
                                ".",
                                button_width,
                                button_height,
                            ) {
                                self.input_decimal();
                            }


                            if self.operator_button(
                                ui,
                                "=",
                                button_width,
                                button_height,
                            ) {
                                self.calculate_result();
                            }
                        });


                        // ========================================
                        // GITHUB CREDIT
                        // ========================================

                        ui.add_space(10.0);

                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("@kiru0real")
                                    .size(11.0)
                                    .color(SECONDARY_TEXT),
                            );
                        });
                    });
            });
    }
}


// ============================================================
// ANIMATED BUTTONS
// ============================================================

impl Calculator {

    fn number_button(
        &self,
        ui: &mut egui::Ui,
        text: &str,
        width: f32,
        height: f32,
    ) -> bool {

        self.animated_button(
            ui,
            text,
            width,
            height,
            false,
            Some(KeyAction::Digit(text.chars().next().unwrap_or(' '))),
        )
    }


    fn action_button(
        &self,
        ui: &mut egui::Ui,
        text: &str,
        width: f32,
        height: f32,
    ) -> bool {

        let action = match text {
            "AC" => Some(KeyAction::Clear),
            "BACKSPACE" => Some(KeyAction::Backspace),
            _ => None,
        };

        if text == "BACKSPACE" {
            return self.backspace_button(
                ui,
                width,
                height,
                action,
            );
        }

        self.animated_button(
            ui,
            text,
            width,
            height,
            false,
            action,
        )
    }


    fn operator_button(
        &self,
        ui: &mut egui::Ui,
        text: &str,
        width: f32,
        height: f32,
    ) -> bool {

        let action = match text {
            "+" => Some(KeyAction::Add),
            "−" => Some(KeyAction::Subtract),
            "×" => Some(KeyAction::Multiply),
            "÷" => Some(KeyAction::Divide),
            "=" => Some(KeyAction::Equals),
            _ => None,
        };

        self.animated_button(
            ui,
            text,
            width,
            height,
            true,
            action,
        )
    }


    fn backspace_button(
        &self,
        ui: &mut egui::Ui,
        width: f32,
        height: f32,
        keyboard_action: Option<KeyAction>,
    ) -> bool {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(width, height),
            egui::Sense::click(),
        );

        let keyboard_pressed = keyboard_action
            .zip(self.keyboard_flash)
            .map(|(action, (flash_action, until))| {
                action == flash_action
                    && ui.ctx().input(|input| input.time) < until
            })
            .unwrap_or(false);

        let pressed = response.is_pointer_button_down_on()
            || keyboard_pressed;

        let hovered = response.hovered();
        let fill = if pressed {
            button_pressed_color()
        } else if hovered {
            button_hover_color()
        } else {
            button_color()
        };

        ui.painter().rect(
            rect,
            egui::CornerRadius::same(18),
            fill,
            egui::Stroke::new(1.0, BUTTON_BORDER),
            egui::StrokeKind::Inside,
        );

        let center = rect.center();
        let color = TEXT;

        ui.painter().line_segment(
            [
                center + egui::vec2(7.0, -7.0),
                center + egui::vec2(7.0, 7.0),
            ],
            egui::Stroke::new(1.8, color),
        );

        ui.painter().line_segment(
            [
                center + egui::vec2(7.0, -7.0),
                center + egui::vec2(-7.0, 0.0),
            ],
            egui::Stroke::new(1.8, color),
        );

        ui.painter().line_segment(
            [
                center + egui::vec2(-7.0, 0.0),
                center + egui::vec2(7.0, 7.0),
            ],
            egui::Stroke::new(1.8, color),
        );

        ui.painter().line_segment(
            [
                center + egui::vec2(-1.0, 0.0),
                center + egui::vec2(10.0, 0.0),
            ],
            egui::Stroke::new(1.8, color),
        );

        if keyboard_pressed {
            ui.ctx().request_repaint();
        }

        response.clicked()
    }


    fn history_button(
        &self,
        ui: &mut egui::Ui,
        text: &str,
        width: f32,
        height: f32,
    ) -> bool {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(width, height),
            egui::Sense::click(),
        );

        let hovered = response.hovered();
        let pressed = response.is_pointer_button_down_on();

        let fill = if pressed {
            egui::Color32::from_rgb(42, 43, 47)
        } else if hovered {
            egui::Color32::from_rgb(36, 37, 41)
        } else {
            egui::Color32::from_rgb(31, 32, 36)
        };

        ui.painter().rect(
            rect,
            egui::CornerRadius::same(12),
            fill,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(52, 53, 58),
            ),
            egui::StrokeKind::Inside,
        );

        let center = rect.center();
        let color = SECONDARY_TEXT;

        if text == "HISTORY" {
            let radius = 7.0;
            let start = -0.7_f32;
            let end = 4.7_f32;

            ui.painter().circle_stroke(
                center,
                radius,
                egui::Stroke::new(1.6, color),
            );

            let p1 = center
                + egui::vec2(
                    start.cos() * radius,
                    start.sin() * radius,
                );

            let p2 = center
                + egui::vec2(
                    end.cos() * radius,
                    end.sin() * radius,
                );

            ui.painter().line_segment(
                [center, center + egui::vec2(0.0, -4.0)],
                egui::Stroke::new(1.6, color),
            );

            ui.painter().line_segment(
                [center, center + egui::vec2(4.0, 0.0)],
                egui::Stroke::new(1.6, color),
            );

            ui.painter().line_segment(
                [p1, p1 + egui::vec2(-3.5, 0.5)],
                egui::Stroke::new(1.6, color),
            );

            let _ = p2;
        } else {
            ui.painter().line_segment(
                [
                    center + egui::vec2(4.5, -6.0),
                    center + egui::vec2(4.5, 6.0),
                ],
                egui::Stroke::new(1.8, color),
            );

            ui.painter().line_segment(
                [
                    center + egui::vec2(4.5, -6.0),
                    center + egui::vec2(-5.5, 0.0),
                ],
                egui::Stroke::new(1.8, color),
            );

            ui.painter().line_segment(
                [
                    center + egui::vec2(-5.5, 0.0),
                    center + egui::vec2(4.5, 6.0),
                ],
                egui::Stroke::new(1.8, color),
            );

            ui.painter().line_segment(
                [
                    center + egui::vec2(-1.5, 0.0),
                    center + egui::vec2(6.5, 0.0),
                ],
                egui::Stroke::new(1.8, color),
            );
        }

        response.clicked()
    }


    fn animated_button(
        &self,
        ui: &mut egui::Ui,
        text: &str,
        width: f32,
        height: f32,
        is_operator: bool,
        keyboard_action: Option<KeyAction>,
    ) -> bool {

        // ----------------------------------------------------
        // Allocate space for the button.
        // ----------------------------------------------------

        let desired_size =
            egui::vec2(width, height);

        let (rect, response) =
            ui.allocate_exact_size(
                desired_size,
                egui::Sense::click(),
            );


        // ----------------------------------------------------
        // Animation state.
        //
        // animate_bool smoothly changes between 0 and 1.
        // ----------------------------------------------------

        let animation_id =
            response.id.with("press_animation");

        let keyboard_pressed = keyboard_action
            .zip(self.keyboard_flash)
            .map(|(action, (flash_action, until))| {
                action == flash_action
                    && ui.ctx().input(|input| input.time) < until
            })
            .unwrap_or(false);

        let pressed =
            response.is_pointer_button_down_on()
                || keyboard_pressed;

        let animation = ui.ctx().animate_bool_with_time(
            animation_id,
            pressed,
            0.08,
        );


        // Hover animation.

        let hover_id =
            response.id.with("hover_animation");

        let hover =
            ui.ctx().animate_bool_with_time(
                hover_id,
                response.hovered(),
                0.12,
            );


        // ----------------------------------------------------
        // Move the button down slightly when pressed.
        // ----------------------------------------------------

        let press_offset =
            animation * 3.0;

        let animated_rect =
            rect.translate(egui::vec2(
                0.0,
                press_offset,
            ));


        // ----------------------------------------------------
        // Background color.
        // ----------------------------------------------------

        let background = if is_operator {

            if pressed {
                operator_pressed_color()

            } else if response.hovered() {
                operator_hover_color()

            } else {
                OPERATOR_BACKGROUND
            }

        } else {

            if pressed {
                button_pressed_color()

            } else if response.hovered() {
                button_hover_color()

            } else {
                button_color()
            }
        };


        // ----------------------------------------------------
        // Slightly brighten the border on hover.
        // ----------------------------------------------------

        let border_alpha =
            (25.0 + hover * 20.0) as u8;

        let border =
            egui::Color32::from_rgba_unmultiplied(
                255,
                255,
                255,
                border_alpha,
            );


        // ----------------------------------------------------
        // Draw rounded button.
        // ----------------------------------------------------

        ui.painter().rect(
            animated_rect,
            egui::CornerRadius::same(20),
            background,
            egui::Stroke::new(
                1.0,
                if is_operator {
                    egui::Color32::from_rgb(
                        110,
                        70,
                        20,
                    )
                } else {
                    border
                },
            ),
            egui::StrokeKind::Inside,
        );


        // ----------------------------------------------------
        // Text color.
        // ----------------------------------------------------

        let text_color =
            if is_operator {
                OPERATOR
            } else if text == "AC"
                || text == "BACKSPACE"
                || text == "±"
            {
                SECONDARY_TEXT
            } else {
                TEXT
            };


        // ----------------------------------------------------
        // Text.
        // ----------------------------------------------------

        let font_size =
            if is_operator {
                27.0
            } else if text == "AC"
                || text == "BACKSPACE"
                || text == "±"
            {
                20.0
            } else {
                25.0
            };


        ui.painter().text(
            animated_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(
                font_size,
            ),
            text_color,
        );


        // ----------------------------------------------------
        // Keep animation alive.
        // ----------------------------------------------------

        if response.hovered()
            || pressed
            || animation > 0.0
            || hover > 0.0
        {
            ui.ctx().request_repaint();
        }


        response.clicked()
    }
}


// ============================================================
// MATH
// ============================================================

fn calculate(
    a: f64,
    b: f64,
    operation: Operation,
) -> f64 {

    match operation {

        Operation::Add => a + b,

        Operation::Subtract => a - b,

        Operation::Multiply => a * b,

        Operation::Divide => {

            if b == 0.0 {
                0.0
            } else {
                a / b
            }
        }
    }
}


// ============================================================
// OPERATION SYMBOL
// ============================================================

fn operation_symbol(
    operation: Operation,
) -> &'static str {

    match operation {

        Operation::Add => "+",

        Operation::Subtract => "−",

        Operation::Multiply => "×",

        Operation::Divide => "÷",
    }
}


// ============================================================
// NUMBER FORMATTING
// ============================================================

fn format_number(value: f64) -> String {

    if value.fract() == 0.0 {

        format!("{:.0}", value)

    } else {

        format!("{:.8}", value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}
