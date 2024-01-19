mod package_info;

use std::fmt;

use gtk::{
    gio::{AppInfo, ListStore},
    glib,
    prelude::*,
};
use gtk_list_provider::*;

const APP_ID: &str = "com.github.plish-plash.plash-gtk-apps.Applist";

#[derive(Clone, Copy)]
struct AppListColumn;

impl fmt::Display for AppListColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Applications")
    }
}

impl ListContent for AppListColumn {
    type ModelItem = AppInfo;
    fn setup_content(&self) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let image = gtk::Image::builder().pixel_size(32).build();
        outer.append(&image);
        let inner = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .valign(gtk::Align::Center)
            .build();
        let name = gtk::Label::builder().xalign(0.0).build();
        let description = gtk::Label::builder().xalign(0.0).build();
        inner.append(&name);
        inner.append(&description);
        outer.append(&inner);
        outer.upcast()
    }
    fn bind_content(&self, widget: gtk::Widget, item: Self::ModelItem) {
        let image: gtk::Image = widget.first_child().unwrap().downcast().unwrap();
        let name: gtk::Label = image
            .next_sibling()
            .unwrap()
            .first_child()
            .unwrap()
            .downcast()
            .unwrap();
        let description: gtk::Label = name.next_sibling().unwrap().downcast().unwrap();
        image.set_gicon(item.icon().as_ref());
        name.set_markup(&format!(
            "<b>{}</b>",
            gtk::glib::markup_escape_text(&item.display_name())
        ));
        description.set_text(&item.description().unwrap_or_default());
    }
}

impl ListColumn for AppListColumn {
    fn sort(&self, a: &Self::ModelItem, b: &Self::ModelItem) -> gtk::Ordering {
        a.display_name().cmp(&b.display_name()).into()
    }
}

struct AppListDetail;

impl ListContent for AppListDetail {
    type ModelItem = AppInfo;
    fn setup_content(&self) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
        let top = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk::Align::Center)
            .build();
        let image = gtk::Image::builder().pixel_size(48).build();
        let name = gtk::Label::new(None);
        top.append(&image);
        top.append(&name);
        outer.append(&top);
        outer.upcast()
    }
    fn bind_content(&self, widget: gtk::Widget, item: Self::ModelItem) {
        let outer: gtk::Box = widget.downcast().unwrap();
        let top = outer.first_child().unwrap();
        let image: gtk::Image = top.first_child().unwrap().downcast().unwrap();
        let name: gtk::Label = image.next_sibling().unwrap().downcast().unwrap();
        image.set_gicon(item.icon().as_ref());
        name.set_markup(&format!(
            "<big>{}</big>",
            gtk::glib::markup_escape_text(&item.display_name())
        ));

        if let Some(content) = top.next_sibling() {
            outer.remove(&content);
        }
        let info = package_info::get_package_info(&item.executable());
        let content: gtk::Widget = match info {
            Ok(info) => {
                let grid = gtk::Grid::new();
                for (row, (key, value)) in info.into_iter().enumerate() {
                    let key_label = gtk::Label::builder().xalign(0.0).build();
                    key_label.set_text(&key);
                    let value_label = gtk::Label::builder().xalign(0.0).wrap(true).build();
                    value_label.set_text(&value);
                    grid.attach(&key_label, 0, row as i32, 1, 1);
                    grid.attach(&value_label, 1, row as i32, 1, 1);
                }
                grid.upcast()
            }
            Err(error) => {
                let label = gtk::Label::new(Some(&error));
                label.set_wrap(true);
                label.upcast()
            }
        };
        outer.append(&content);
    }
}

struct AppListProvider {
    model: ListStore,
}

impl ListProvider for AppListProvider {
    type Model = ListStore;
    type ModelItem = AppInfo;
    type Column = AppListColumn;
    type Detail = AppListDetail;
    fn model(&self) -> Self::Model {
        self.model.clone()
    }
    fn columns(&self) -> Vec<Self::Column> {
        vec![AppListColumn]
    }
    fn detail(&self) -> Self::Detail {
        AppListDetail
    }
}

fn build_window(app: &gtk::Application) {
    let mut apps = gtk::gio::AppInfo::all();
    apps.retain(|a| a.should_show());
    apps.sort_by(|a, b| a.display_name().cmp(&b.display_name()));
    let model = ListStore::new(AppInfo::static_type());
    model.extend_from_slice(&apps);

    let provider = AppListProvider { model };
    let (pane, view) = build_list_view(&provider, 240);

    let app_window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Applications")
        .icon_name("start-here-archlinux")
        .default_width(640)
        .default_height(480)
        .child(&pane)
        .build();
    app_window.present();

    view.connect_activate(move |view, position| {
        let item = view
            .model()
            .unwrap()
            .item(position)
            .and_downcast::<AppInfo>();
        if let Some(item) = item {
            if let Err(error) = item.launch(&[], gtk::gio::AppLaunchContext::NONE) {
                let alert = gtk::AlertDialog::builder()
                    .modal(true)
                    .message("Error launching application")
                    .detail(error.to_string())
                    .build();
                alert.show(Some(&app_window));
            }
        }
    });
}

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| load_css());
    app.connect_activate(build_window);
    app.run()
}
