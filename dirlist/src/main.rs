mod directory;

use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use gtk::{
    gio::{AppInfo, AppLaunchContext, FileInfo, FileType},
    glib,
    prelude::*,
};

use directory::DirectoryProvider;

const APP_ID: &str = "com.github.plish-plash.plash-gtk-apps.Dirlist";

pub fn expand_user_path(path: &Path) -> PathBuf {
    if let Ok(path) = path.strip_prefix("~") {
        let mut home = glib::home_dir();
        home.push(path);
        home
    } else {
        path.to_owned()
    }
}

struct DirectoryWindow {
    app_window: gtk::ApplicationWindow,
    entry: gtk::Entry,
    navigate_up: gtk::Button,
    provider: DirectoryProvider,
    view: gtk::ColumnView,
    queued_selection: RefCell<Option<PathBuf>>,
}

impl DirectoryWindow {
    fn deselect(&self) {
        let model = self.view.model().and_downcast::<gtk::SingleSelection>();
        if let Some(model) = model {
            model.unselect_item(model.selected());
        }
    }
    fn dequeue_selection(&self) {
        if let Some(selection) = self.queued_selection.take() {
            let model = self.view.model().and_downcast::<gtk::SingleSelection>();
            if let Some(model) = model {
                for position in 0..model.n_items() {
                    if model
                        .item(position)
                        .and_downcast::<FileInfo>()
                        .unwrap()
                        .name()
                        == selection
                    {
                        model.set_selected(position);
                        return;
                    }
                }
            }
        }
    }
    fn set_path(&self, path: &Path) {
        let mut path = if let Ok(path) = expand_user_path(path).canonicalize() {
            path
        } else {
            let alert = gtk::AlertDialog::builder()
                .modal(true)
                .message("Invalid path")
                .detail(path.to_string_lossy())
                .build();
            alert.show(Some(&self.app_window));
            return;
        };
        if !path.is_dir() {
            let file_name = if let Some(file_name) = path.file_name() {
                Path::new(file_name).to_owned()
            } else {
                return;
            };
            path.pop();
            self.queued_selection.replace(Some(file_name));
        }

        self.deselect();
        let title = path
            .file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy();
        self.app_window.set_title(Some(&title));
        self.entry.set_text(&path.to_string_lossy());
        self.navigate_up.set_sensitive(path.parent().is_some());
        self.provider.set_path(&path);
        if !self.provider.directory.is_loading() {
            self.dequeue_selection();
        }
    }
}

fn open_window(app: &gtk::Application, path: &Path) {
    const SPACING: i32 = 6;
    let root = gtk::Box::new(gtk::Orientation::Vertical, SPACING);

    let entry = gtk::Entry::builder().hexpand(true).build();
    let navigate_up = gtk::Button::from_icon_name("go-up");
    let entry_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(SPACING)
        .margin_top(SPACING)
        .margin_start(SPACING)
        .margin_end(SPACING)
        .build();
    entry_row.append(&entry);
    entry_row.append(&navigate_up);
    root.append(&entry_row);

    let provider = DirectoryProvider::new(path);
    let (pane, view) = gtk_list_provider::build_column_view(&provider, 240);
    root.append(&pane);

    let app_window = gtk::ApplicationWindow::builder()
        .application(app)
        .default_width(640)
        .default_height(480)
        .child(&root)
        .build();

    let window = Rc::new(DirectoryWindow {
        app_window,
        entry,
        navigate_up,
        provider,
        view,
        queued_selection: RefCell::new(None),
    });
    window.set_path(path);
    window.app_window.present();

    window
        .entry
        .connect_activate(glib::clone!(@strong window => move |_| {
            let entry_text = window.entry.text();
            window.set_path(Path::new(&entry_text));
        }));
    window
        .navigate_up
        .connect_clicked(glib::clone!(@strong window => move |_| {
            let mut path = window.provider.path();
            if path.pop() {
                window.set_path(&path);
            }
        }));
    window.view.connect_activate(glib::clone!(@strong window => move |view, position| {
        let item = view
            .model()
            .unwrap()
            .item(position)
            .and_downcast::<FileInfo>();
        if let Some(item) = item {
            let mut path = window.provider.path();
            path.push(item.name());
            if item.file_type() == FileType::Directory {
                window.set_path(&path);
            } else {
                let uri = format!("file://{}", path.to_string_lossy());
                if let Err(error) = AppInfo::launch_default_for_uri(&uri, AppLaunchContext::NONE) {
                    eprintln!("{}", error);
                }
            }
        }
    }));
    window.provider.directory.connect_loading_notify(
        glib::clone!(@strong window => move |directory| {
            if !directory.is_loading() {
                window.dequeue_selection();
            }
        }),
    );
}

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gtk::gio::ApplicationFlags::HANDLES_OPEN)
        .build();
    app.connect_startup(|_| gtk_list_provider::load_css());
    app.connect_activate(|app| {
        let path = std::env::current_dir().expect("couldn't get current directory");
        open_window(app, &path);
    });
    app.connect_open(|app, files, _hint| {
        for file in files {
            let path = file.path().unwrap();
            open_window(app, &path);
        }
    });
    app.run()
}
