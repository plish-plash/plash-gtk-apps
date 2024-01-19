mod config;
mod project_info;

use std::{fmt, rc::Rc, time::Duration};

use config::AppConfig;
use gtk::{gio::ListStore, glib, prelude::*};
use gtk_list_provider::*;

use once_cell::unsync::OnceCell;
use project_info::{ProjectInfo, ProjectInfoInner};

const APP_ID: &str = "com.github.plish-plash.plash-gtk-apps.Projlist";
const APP_CONFIG_DIR: &str = "projlist";
const APP_PROJECTS_FILE: &str = "Projects.toml";

struct NameColumn;

impl NameColumn {
    fn bind_content(widget: gtk::Label, item: ProjectInfo) {
        widget.set_text(item.name());
    }
    fn sort(a: &ProjectInfo, b: &ProjectInfo) -> gtk::Ordering {
        a.name().cmp(b.name()).into()
    }
}

struct TypeColumn;

impl TypeColumn {
    fn bind_content(widget: gtk::Label, item: ProjectInfo) {
        widget.set_text(item.project_type());
    }
    fn sort(app_config: &AppConfig, a: &ProjectInfo, b: &ProjectInfo) -> gtk::Ordering {
        let a = app_config.project_type_index(a.project_type());
        let b = app_config.project_type_index(b.project_type());
        a.cmp(&b).into()
    }
}

struct StatusColumn;

impl StatusColumn {
    fn bind_content(widget: gtk::Label, item: ProjectInfo) {
        widget.set_text(item.status());
    }
    fn sort(app_config: &AppConfig, a: &ProjectInfo, b: &ProjectInfo) -> gtk::Ordering {
        let a = app_config.status_index(a.status());
        let b = app_config.status_index(b.status());
        a.cmp(&b).into()
    }
}

struct LastOpenedColumn;

impl LastOpenedColumn {
    fn bind_content(widget: gtk::Label, item: ProjectInfo) {
        let formatter = timeago::Formatter::new();
        let text = item
            .last_opened()
            .map(|dt| {
                let now = glib::DateTime::now(&dt.timezone()).unwrap();
                formatter.convert(Duration::from_micros(
                    now.difference(&dt).as_microseconds() as u64
                ))
            })
            .unwrap_or_default();
        widget.set_text(&text);
    }
    fn sort(a: &ProjectInfo, b: &ProjectInfo) -> gtk::Ordering {
        a.last_opened().cmp(&b.last_opened()).into()
    }
}

struct PathColumn;

impl PathColumn {
    fn bind_content(widget: gtk::Label, item: ProjectInfo) {
        widget.set_text(item.short_path());
    }
    fn sort(a: &ProjectInfo, b: &ProjectInfo) -> gtk::Ordering {
        a.path().cmp(b.path()).into()
    }
}

#[derive(Clone)]
enum ProjectColumn {
    Name,
    Type(Rc<OnceCell<AppConfig>>),
    Status(Rc<OnceCell<AppConfig>>),
    LastOpened,
    Path,
}

impl fmt::Display for ProjectColumn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProjectColumn::Name => write!(f, "Name"),
            ProjectColumn::Type(_) => write!(f, "Type"),
            ProjectColumn::Status(_) => write!(f, "Status"),
            ProjectColumn::LastOpened => write!(f, "Last Opened"),
            ProjectColumn::Path => write!(f, "Path"),
        }
    }
}

impl ListContent for ProjectColumn {
    type ModelItem = ProjectInfo;
    fn setup_content(&self) -> gtk::Widget {
        gtk::Label::builder().xalign(0.0).build().upcast()
    }
    fn bind_content(&self, widget: gtk::Widget, item: Self::ModelItem) {
        let widget: gtk::Label = widget.downcast().unwrap();
        match self {
            ProjectColumn::Name => NameColumn::bind_content(widget, item),
            ProjectColumn::Type(_) => TypeColumn::bind_content(widget, item),
            ProjectColumn::Status(_) => StatusColumn::bind_content(widget, item),
            ProjectColumn::LastOpened => LastOpenedColumn::bind_content(widget, item),
            ProjectColumn::Path => PathColumn::bind_content(widget, item),
        }
    }
}

impl ListColumn for ProjectColumn {
    fn sort(&self, a: &Self::ModelItem, b: &Self::ModelItem) -> gtk::Ordering {
        match self {
            ProjectColumn::Name => NameColumn::sort(a, b),
            ProjectColumn::Type(app_config) => TypeColumn::sort(app_config.get().unwrap(), a, b),
            ProjectColumn::Status(app_config) => {
                StatusColumn::sort(app_config.get().unwrap(), a, b)
            }
            ProjectColumn::LastOpened => LastOpenedColumn::sort(a, b),
            ProjectColumn::Path => PathColumn::sort(a, b),
        }
    }
}

struct ProjectDetail(Rc<OnceCell<AppConfig>>);

impl ListContent for ProjectDetail {
    type ModelItem = ProjectInfo;
    fn setup_content(&self) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
        let name = gtk::Label::new(None);
        outer.append(&name);
        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let open_project = gtk::Button::builder().label("Open Project").build();
        button_box.append(&open_project);
        let open_folder = gtk::Button::builder().label("Open Folder").build();
        button_box.append(&open_folder);
        outer.append(&button_box);
        let status_strings: Vec<&str> = self
            .0
            .get()
            .unwrap()
            .statuses()
            .iter()
            .map(|s| -> &str { s })
            .collect();
        let status = gtk::DropDown::from_strings(&status_strings);
        outer.append(&status);
        let notes = gtk::TextView::builder().hexpand(true).vexpand(true).build();
        outer.append(&notes);
        outer.upcast()
    }
    fn bind_content(&self, widget: gtk::Widget, item: Self::ModelItem) {
        let name: gtk::Label = widget.first_child().unwrap().downcast().unwrap();
        name.set_markup(&format!(
            "<big>{}</big>",
            gtk::glib::markup_escape_text(item.name())
        ));
    }
}

#[derive(Clone)]
struct ProjectProvider {
    app_config: Rc<OnceCell<AppConfig>>,
    model: ListStore,
}

impl ListProvider for ProjectProvider {
    type Model = ListStore;
    type ModelItem = ProjectInfo;
    type Column = ProjectColumn;
    type Detail = ProjectDetail;
    fn model(&self) -> Self::Model {
        self.model.clone()
    }
    fn columns(&self) -> Vec<Self::Column> {
        vec![
            ProjectColumn::Name,
            ProjectColumn::Type(self.app_config.clone()),
            ProjectColumn::Status(self.app_config.clone()),
            ProjectColumn::LastOpened,
            ProjectColumn::Path,
        ]
    }
    fn detail(&self) -> Self::Detail {
        ProjectDetail(self.app_config.clone())
    }
}

fn load_config() -> Result<AppConfig, String> {
    let mut config_dir = gtk::glib::user_config_dir();
    config_dir.push(APP_CONFIG_DIR);
    config::load_config(&config_dir)
}

fn load_projects(model: &ListStore) -> Result<(), String> {
    let mut projects_file = gtk::glib::home_dir();
    projects_file.push(APP_PROJECTS_FILE);
    let projects = config::load_projects(&projects_file)?;
    let projects: Vec<_> = projects
        .into_iter()
        .map(ProjectInfoInner::from_config)
        .map(ProjectInfo::new)
        .collect();
    model.remove_all();
    model.extend_from_slice(&projects);
    Ok(())
}

fn build_window(app: &gtk::Application, provider: &ProjectProvider) {
    let (pane, view) = build_column_view(provider, 240);

    let app_window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Projects")
        .icon_name("applications-development")
        .default_width(640)
        .default_height(480)
        .child(&pane)
        .build();
    app_window.present();

    let app_config = provider.app_config.clone();
    view.connect_activate(move |view, position| {
        let item = view
            .model()
            .unwrap()
            .item(position)
            .and_downcast::<ProjectInfo>();
        if let Some(item) = item {
            if let Some(application) = app_config
                .get()
                .unwrap()
                .project_type_application(item.project_type())
            {
                let file = gtk::gio::File::for_path(item.path());
                if let Err(error) = application.launch(&[file], gtk::gio::AppLaunchContext::NONE) {
                    eprintln!("{}", error);
                }
            } else {
                eprintln!(
                    "Could not find application to open {} project.",
                    item.project_type()
                );
            }
        }
    });
}

fn main() -> glib::ExitCode {
    let provider = ProjectProvider {
        app_config: Rc::default(),
        model: ListStore::new(ProjectInfo::static_type()),
    };

    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_startup(glib::clone!(@strong provider => move |_| {
        provider.app_config.set(load_config().unwrap()).map_err(|_| "config loaded multiple times").unwrap();
        load_projects(&provider.model).unwrap();
    }));
    app.connect_shutdown(|_| {}); // TODO save projects
    app.connect_activate(move |app| {
        build_window(app, &provider);
    });
    app.run()
}
