use headless_chrome::protocol::target::methods::CreateTarget;
use headless_chrome::{protocol::page::ScreenshotFormat, Browser, LaunchOptionsBuilder};
use std::error::Error;
use std::fs;

pub fn generate_report_image(url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let browser = Browser::new(LaunchOptionsBuilder::default().build().unwrap())?;
    let tab = browser.new_tab_with_options(CreateTarget {
        url: url, //"https://en125.tribalwars.net/public_report/b191ba459f32079f30d07f1a1903a56f",
        width: Some(2000),
        height: Some(2000),
        browser_context_id: None,
        enable_begin_frame_control: None,
    })?;
    let viewport = tab
        .wait_for_element("td:nth-child(2) > table.vis > tbody > tr")?
        .get_box_model()?
        .content_viewport();

    let png_data = tab.capture_screenshot(ScreenshotFormat::PNG, Some(viewport), true)?;

    fs::write("report.png", png_data.clone())?;

    Ok(png_data)
}
