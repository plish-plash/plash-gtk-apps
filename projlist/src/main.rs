mod config;
mod project_info;

use std::{fmt, time::Duration};

use gtk::{gio::ListStore, glib, prelude::*};
use gtk_list_provider::*;

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
    fn sort(a: &ProjectInfo, b: &ProjectInfo) -> gtk::Ordering {
        let a = config::project_type_to_index(a.project_type());
        let b = config::project_type_to_index(b.project_type());
        a.cmp(&b).into()
    }
}

struct StatusColumn;

impl StatusColumn {
    fn bind_content(widget: gtk::Label, item: ProjectInfo) {
        widget.set_text(item.status());
    }
    fn sort(a: &ProjectInfo, b: &ProjectInfo) -> gtk::Ordering {
        let a = config::status_to_index(a.status());
        let b = config::status_to_index(b.status());
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

#[derive(Clone, Copy)]
enum ProjectColumn {
    Name,
    Type,
    Status,
    LastOpened,
    Path,
}

impl fmt::Display for ProjectColumn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProjectColumn::Name => write!(f, "Name"),
            ProjectColumn::Type => write!(f, "Type"),
            ProjectColumn::Status => write!(f, "Status"),
            ProjectColumn::LastOpened => write!(f, "Last Opened"),
            ProjectColumn::Path => write!(f, "Path"),
        }
    }
}

impl ListColumn for ProjectColumn {
    type ModelItem = ProjectInfo;
    fn setup_content(&self) -> gtk::Widget {
        gtk::Label::builder().xalign(0.0).build().upcast()
    }
    fn bind_content(&self, widget: gtk::Widget, item: Self::ModelItem) {
        let widget: gtk::Label = widget.downcast().unwrap();
        match self {
            ProjectColumn::Name => NameColumn::bind_content(widget, item),
            ProjectColumn::Type => TypeColumn::bind_content(widget, item),
            ProjectColumn::Status => StatusColumn::bind_content(widget, item),
            ProjectColumn::LastOpened => LastOpenedColumn::bind_content(widget, item),
            ProjectColumn::Path => PathColumn::bind_content(widget, item),
        }
    }
    fn sort(&self, a: &Self::ModelItem, b: &Self::ModelItem) -> gtk::Ordering {
        match self {
            ProjectColumn::Name => NameColumn::sort(a, b),
            ProjectColumn::Type => TypeColumn::sort(a, b),
            ProjectColumn::Status => StatusColumn::sort(a, b),
            ProjectColumn::LastOpened => LastOpenedColumn::sort(a, b),
            ProjectColumn::Path => PathColumn::sort(a, b),
        }
    }
}

struct ProjectDetail;

impl ListDetail for ProjectDetail {
    type ModelItem = ProjectInfo;
    fn setup_content() -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
        let name = gtk::Label::new(None);
        outer.append(&name);
        outer.upcast()
    }
    fn bind_content(widget: gtk::Widget, item: Self::ModelItem) {
        let name: gtk::Label = widget.first_child().unwrap().downcast().unwrap();
        name.set_markup(&format!(
            "<big>{}</big>",
            gtk::glib::markup_escape_text(item.name())
        ));
    }
}

struct ProjectProvider {
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
    fn columns(&self) -> &[Self::Column] {
        &[
            ProjectColumn::Name,
            ProjectColumn::Type,
            ProjectColumn::Status,
            ProjectColumn::LastOpened,
            ProjectColumn::Path,
        ]
    }
}

fn build_window(app: &gtk::Application) {
    let mut config_dir = gtk::glib::user_config_dir();
    config_dir.push(APP_CONFIG_DIR);
    let mut projects_file = gtk::glib::home_dir();
    projects_file.push(APP_PROJECTS_FILE);
    let projects = match config::load_config(&config_dir)
        .and_then(|_| config::load_projects(&projects_file))
    {
        Ok(projects) => projects,
        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    };
    let projects: Vec<_> = projects
        .into_iter()
        .map(ProjectInfoInner::from_config)
        .map(ProjectInfo::new)
        .collect();

    let model = ListStore::new(ProjectInfo::static_type());
    model.extend_from_slice(&projects);
    let provider = ProjectProvider { model };
    let (pane, view) = build_column_view(&provider, 240);

    let app_window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Projects")
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
            .and_downcast::<ProjectInfo>();
        if let Some(item) = item {
            if let Some(application) = config::project_type_application(item.project_type()) {
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
    let app = gtk::Application::builder().application_id(APP_ID).build();
    // app.connect_startup(|_| load_css());
    app.connect_activate(build_window);
    app.run()
}
