use crate::engine::pipeline::RenderPipeline;

#[derive(Debug, Clone, PartialEq)]
pub enum PageSize {
    A4,
    Letter,
    Legal,
    Custom(f32, f32),
}
impl PageSize {
    pub fn dimensions_mm(&self) -> (f32, f32) {
        match self {
            PageSize::A4 => (210.0, 297.0),
            PageSize::Letter => (215.9, 279.4),
            PageSize::Legal => (215.9, 355.6),
            PageSize::Custom(w, h) => (*w, *h),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Orientation { Portrait, Landscape }
#[derive(Debug, Clone)]
pub struct PrintMargins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}
impl PrintMargins {
    pub fn default_mm() -> Self { Self { top: 10.0, right: 10.0, bottom: 10.0, left: 10.0 } }
}
#[derive(Debug, Clone)]
pub struct PrintJob {
    pub title: String,
    pub url: String,
    pub page_size: PageSize,
    pub margins: PrintMargins,
    pub orientation: Orientation,
}
impl PrintJob {
    pub fn new(title: &str, url: &str) -> Self {
        Self { title: title.to_string(), url: url.to_string(),
            page_size: PageSize::A4, margins: PrintMargins::default_mm(), orientation: Orientation::Portrait }
    }
    pub fn content_dimensions_mm(&self) -> (f32, f32) {
        let (pw, ph) = self.page_size.dimensions_mm();
        let (pw, ph) = match self.orientation {
            Orientation::Portrait => (pw, ph),
            Orientation::Landscape => (ph, pw),
        };
        (pw - self.margins.left - self.margins.right, ph - self.margins.top - self.margins.bottom)
    }
}
pub fn render_page_to_image(html: &str, css: &[&str], width: f32, height: f32) -> Vec<u8> {
    let mut pipeline = RenderPipeline::new();
    let rendered = pipeline.render_to_pixels(html, css, width, height);
    rendered.pixels
}
pub fn estimate_page_count(content_height: f32, page_height: f32) -> usize {
    if page_height <= 0.0 { return 1; }
    ((content_height / page_height).ceil() as usize).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_page_size_dimensions() {
        let (w, h) = PageSize::A4.dimensions_mm();
        assert!((w - 210.0).abs() < 0.01);
        assert!((h - 297.0).abs() < 0.01);
    }
    #[test]
    fn test_estimate_pages() {
        assert_eq!(estimate_page_count(3000.0, 1000.0), 3);
        assert_eq!(estimate_page_count(3001.0, 1000.0), 4);
        assert_eq!(estimate_page_count(500.0, 1000.0), 1);
        assert_eq!(estimate_page_count(100.0, 0.0), 1);
    }
    #[test]
    fn test_print_job_content_dims() {
        let job = PrintJob::new("Test", "http://example.com");
        let (w, h) = job.content_dimensions_mm();
        assert!((w - 190.0).abs() < 0.01);
        assert!((h - 277.0).abs() < 0.01);
    }
    #[test]
    fn test_default_margins() {
        let m = PrintMargins::default_mm();
        assert_eq!(m.top, 10.0);
        assert_eq!(m.right, 10.0);
        assert_eq!(m.bottom, 10.0);
        assert_eq!(m.left, 10.0);
    }
}
