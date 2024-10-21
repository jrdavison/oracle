slint::include_modules!();

fn main() -> Result<(), slint::PlatformError>{
    let main_window = MainWindow::new();

    return main_window.run();
}
