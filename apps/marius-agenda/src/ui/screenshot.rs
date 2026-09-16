//! Off-screen widget capture (Wayland-safe; scrot often captures black).

use gtk::gdk::prelude::*;
use gtk::gsk::prelude::*;
use gtk::prelude::*;
use libadwaita as adw;
use std::path::Path;

pub fn capture_widget_png(widget: &impl IsA<gtk::Widget>, path: &Path) {
    widget.queue_draw();
    let ctx = gtk::glib::MainContext::default();
    for _ in 0..20 {
        while ctx.iteration(false) {}
    }
    gtk::prelude::WidgetExt::display(widget).sync();

    let width = widget.width().max(1);
    let height = widget.height().max(1);
    let w = width as f64;
    let h = height as f64;

    let paintable = gtk::WidgetPaintable::new(Some(widget));
    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(&snapshot, w, h);

    let node = match snapshot.to_node() {
        Some(n) => n,
        None => {
            eprintln!("screenshot: empty render node → {}", path.display());
            return;
        }
    };

    let surface = widget
        .root()
        .and_then(|w| w.downcast::<adw::ApplicationWindow>().ok())
        .and_then(|win| gtk::prelude::NativeExt::surface(&win));
    let renderer = match surface.as_ref().and_then(|s| gtk::gsk::Renderer::for_surface(s)) {
        Some(r) => r,
        None => {
            eprintln!("screenshot: no GSK renderer → {}", path.display());
            return;
        }
    };
    if !renderer.is_realized() {
        if let Some(surf) = surface.clone() {
            if let Err(e) = renderer.realize(Some(&surf)) {
                eprintln!("screenshot: realize failed: {} → {}", e, path.display());
                return;
            }
        }
    }

    let viewport = gtk::graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
    let texture = renderer.render_texture(&node, Some(&viewport));
    match texture.save_to_png(path) {
        Ok(()) => eprintln!("screenshot → {}", path.display()),
        Err(e) => eprintln!("screenshot: save_to_png failed: {} → {}", e, path.display()),
    }
    renderer.unrealize();
}
