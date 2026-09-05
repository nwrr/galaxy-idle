//! Kamera ortografik + easing frame-rate-independent — port near-verbatim dari
//! `andromeda-simulation-tui`'s `src/camera.rs`.

use glam::{Vec2, Vec3};

/// Konstanta waktu animasi kamera (detik).
pub const CAM_TAU: f32 = 0.35;

/// Kamera ortografik (benar utk objek sejauh galaksi -- tanpa perspektif). `view_center`
/// hidup di koordinat view-plane (stlh rotasi inklinasi+PA, sblm zoom), shg mengikuti POI
/// (Phase 7) = memproyeksikan posisinya saja.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub view_center: Vec2,
    /// Zoom dlm piksel per kpc.
    pub zoom: f32,
    pub incl_cs: Vec2,
    pub pa_cs: Vec2,
    pub cell_aspect: f32,
    /// Lebar dunia (kpc) yg dimuat viewport pd zoom overview.
    pub span_kpc: f32,
}

impl Camera {
    pub fn new(cell_aspect: f32, incl_deg: f32, pa_deg: f32, span_kpc: f32) -> Self {
        let i = incl_deg.to_radians();
        let p = pa_deg.to_radians();
        Self {
            view_center: Vec2::ZERO,
            zoom: 4.0,
            incl_cs: Vec2::new(i.cos(), i.sin()),
            pa_cs: Vec2::new(p.cos(), p.sin()),
            cell_aspect,
            span_kpc,
        }
    }

    /// world (kpc, bidang disk + z) -> koordinat view-plane.
    pub fn view_plane(&self, w: Vec3) -> Vec2 {
        let vy = w.y * self.incl_cs.x - w.z * self.incl_cs.y;
        Vec2::new(
            w.x * self.pa_cs.x - vy * self.pa_cs.y,
            w.x * self.pa_cs.y + vy * self.pa_cs.x,
        )
    }

    /// view-plane -> piksel layar.
    pub fn to_pixel(self, q: Vec2, screen: Vec2) -> Vec2 {
        let d = q - self.view_center;
        Vec2::new(
            d.x * self.zoom + screen.x * 0.5,
            d.y * self.zoom * self.cell_aspect + screen.y * 0.5,
        )
    }

    pub fn project(&self, w: Vec3, screen: Vec2) -> Vec2 {
        self.to_pixel(self.view_plane(w), screen)
    }

    pub fn overview_zoom(&self, screen_w: f32) -> f32 {
        (screen_w / self.span_kpc).max(0.1)
    }

    /// Geser kamera sejauh (dx,dy) piksel layar.
    pub fn pan_px(&mut self, dx: f32, dy: f32) {
        self.view_center.x += dx / self.zoom;
        self.view_center.y += dy / (self.zoom * self.cell_aspect);
    }
}

/// Interpolasi eksponensial frame-rate-independent menuju target.
pub fn approach(cur: f32, target: f32, dt: f32, tau: f32) -> f32 {
    cur + (target - cur) * (1.0 - (-dt / tau).exp())
}

pub fn approach_v2(cur: Vec2, target: Vec2, dt: f32, tau: f32) -> Vec2 {
    Vec2::new(
        approach(cur.x, target.x, dt, tau),
        approach(cur.y, target.y, dt, tau),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_round_trip_pan() {
        let mut cam = Camera::new(1.0, 77.0, -38.0, 56.0);
        cam.zoom = 5.0;
        let screen = Vec2::new(200.0, 100.0);
        let w = Vec3::new(3.0, -2.0, 0.7);
        let p0 = cam.project(w, screen);
        cam.pan_px(10.0, -6.0);
        let p1 = cam.project(w, screen);
        assert!((p1.x - (p0.x - 10.0)).abs() < 1e-3);
        assert!((p1.y - (p0.y + 6.0)).abs() < 1e-3);
    }

    #[test]
    fn cell_aspect_only_affects_y() {
        let mut cam = Camera::new(1.0, 77.0, -38.0, 56.0);
        cam.zoom = 5.0;
        let screen = Vec2::new(200.0, 100.0);
        let w = Vec3::new(4.0, 5.0, -1.0);
        let p1 = cam.project(w, screen);
        cam.cell_aspect = 2.0;
        let p2 = cam.project(w, screen);
        assert!((p1.x - p2.x).abs() < 1e-4);
        assert!((p2.y - 50.0).abs() > (p1.y - 50.0).abs() - 1e-4);
    }

    #[test]
    fn approach_converges_and_is_stable() {
        let mut x = 0.0;
        for _ in 0..300 {
            x = approach(x, 10.0, 1.0 / 30.0, CAM_TAU);
        }
        assert!((x - 10.0).abs() < 1e-3);
        // dt besar tidak overshoot.
        let y = approach(0.0, 10.0, 5.0, CAM_TAU);
        assert!(y <= 10.0);
    }

    /// `overview_zoom` HARUS genuinely turun saat `span_kpc` naik (dunia lbh lebar -> zoom
    /// keluar lbh jauh) & TAK PERNAH nol/negatif (`max(0.1)` guard).
    #[test]
    fn overview_zoom_scales_inversely_with_span() {
        let narrow = Camera::new(1.0, 35.0, 15.0, 20.0);
        let wide = Camera::new(1.0, 35.0, 15.0, 80.0);
        assert!(narrow.overview_zoom(100.0) > wide.overview_zoom(100.0));
        assert!(wide.overview_zoom(100.0) >= 0.1);
    }
}
