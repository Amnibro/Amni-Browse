#[derive(Debug, Clone)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub content_height: f32,
    pub content_width: f32,
    pub device_pixel_ratio: f32,
    pub zoom: f32,
}
impl Viewport {
    pub fn new(w: f32, h: f32) -> Self {
        Self { width: w, height: h, scroll_x: 0.0, scroll_y: 0.0,
            content_height: h, content_width: w, device_pixel_ratio: 1.0, zoom: 1.0 }
    }
    pub fn max_scroll_x(&self) -> f32 { (self.content_width - self.width).max(0.0) }
    pub fn max_scroll_y(&self) -> f32 { (self.content_height - self.height).max(0.0) }
    pub fn scroll_by(&mut self, dx: f32, dy: f32) {
        self.scroll_x = (self.scroll_x + dx).clamp(0.0, self.max_scroll_x());
        self.scroll_y = (self.scroll_y + dy).clamp(0.0, self.max_scroll_y());
    }
    pub fn scroll_to(&mut self, x: f32, y: f32) {
        self.scroll_x = x.clamp(0.0, self.max_scroll_x());
        self.scroll_y = y.clamp(0.0, self.max_scroll_y());
    }
    pub fn scroll_to_element(&mut self, element_y: f32, element_h: f32) {
        if element_y < self.scroll_y {
            self.scroll_to(self.scroll_x, element_y);
        } else if element_y + element_h > self.scroll_y + self.height {
            self.scroll_to(self.scroll_x, element_y + element_h - self.height);
        }
    }
    pub fn set_size(&mut self, w: f32, h: f32) {
        self.width = w;
        self.height = h;
        self.scroll_x = self.scroll_x.clamp(0.0, self.max_scroll_x());
        self.scroll_y = self.scroll_y.clamp(0.0, self.max_scroll_y());
    }
    pub fn set_content_size(&mut self, w: f32, h: f32) {
        self.content_width = w;
        self.content_height = h;
        self.scroll_x = self.scroll_x.clamp(0.0, self.max_scroll_x());
        self.scroll_y = self.scroll_y.clamp(0.0, self.max_scroll_y());
    }
    pub fn visible_rect(&self) -> (f32, f32, f32, f32) {
        (self.scroll_x, self.scroll_y, self.width, self.height)
    }
    pub fn is_element_visible(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let (vx, vy, vw, vh) = self.visible_rect();
        x + w > vx && x < vx + vw && y + h > vy && y < vy + vh
    }
    pub fn zoom_in(&mut self) { self.zoom = (self.zoom + 0.1).min(5.0); }
    pub fn zoom_out(&mut self) { self.zoom = (self.zoom - 0.1).max(0.1); }
    pub fn zoom_reset(&mut self) { self.zoom = 1.0; }
    pub fn scroll_percent_y(&self) -> f32 {
        let max = self.max_scroll_y();
        if max <= 0.0 { 0.0 } else { self.scroll_y / max }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_scroll_clamp() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_content_size(800.0, 2000.0);
        vp.scroll_by(0.0, 99999.0);
        assert_eq!(vp.scroll_y, 1400.0);
        vp.scroll_by(0.0, -99999.0);
        assert_eq!(vp.scroll_y, 0.0);
    }
    #[test]
    fn test_scroll_to_element() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_content_size(800.0, 3000.0);
        vp.scroll_to_element(1000.0, 100.0);
        assert_eq!(vp.scroll_y, 500.0);
        vp.scroll_to_element(200.0, 50.0);
        assert_eq!(vp.scroll_y, 200.0);
    }
    #[test]
    fn test_visibility() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_content_size(800.0, 3000.0);
        assert!(vp.is_element_visible(100.0, 100.0, 50.0, 50.0));
        assert!(!vp.is_element_visible(100.0, 700.0, 50.0, 50.0));
        vp.scroll_to(0.0, 500.0);
        assert!(vp.is_element_visible(100.0, 700.0, 50.0, 50.0));
    }
    #[test]
    fn test_zoom_and_percent() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_content_size(800.0, 1200.0);
        vp.scroll_to(0.0, 300.0);
        assert!((vp.scroll_percent_y() - 0.5).abs() < 0.001);
        vp.zoom_in();
        assert!((vp.zoom - 1.1).abs() < 0.001);
        vp.zoom_reset();
        assert_eq!(vp.zoom, 1.0);
        vp.zoom_out();
        assert!((vp.zoom - 0.9).abs() < 0.001);
    }
    #[test]
    fn test_resize_clamps_scroll() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_content_size(800.0, 2000.0);
        vp.scroll_to(0.0, 1400.0);
        assert_eq!(vp.scroll_y, 1400.0);
        vp.set_size(800.0, 1800.0);
        assert_eq!(vp.scroll_y, 200.0);
    }
}
