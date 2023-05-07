use glib::Object;
use gtk::{glib::{self, DateTime}, subclass::prelude::ObjectSubclassIsExt};

use crate::config;

mod imp {
    use once_cell::unsync::OnceCell;

    use gtk::glib;
    use gtk::subclass::prelude::*;
    
    #[derive(Default)]
    pub struct ProjectInfo {
        pub inner: OnceCell<super::ProjectInfoInner>,
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for ProjectInfo {
        const NAME: &'static str = "ProjlistProjectInfo";
        type Type = super::ProjectInfo;
        type ParentType = glib::Object;
    }
    
    impl ObjectImpl for ProjectInfo {}
}

#[derive(Debug)]
pub struct ProjectInfoInner {
    name: String,
    project_type: String,
    status: String,
    last_opened: Option<glib::DateTime>,
    path: String,
    short_path: String,
    notes: String,
}

impl ProjectInfoInner {
    pub fn from_config(info: config::ProjectInfo) -> Self {
        let last_opened = if info.last_opened == 0 { None } else { Some(DateTime::from_unix_utc(info.last_opened).unwrap()) };
        let mut short_path = info.path.clone();
        if let Ok(home_dir) = expanduser::expanduser("~") {
            let home_dir = home_dir.to_string_lossy();
            if let Some(s) = short_path.strip_prefix(&*home_dir) {
                short_path = format!("~{}", s);
            }
        }
        ProjectInfoInner {
            name: info.name,
            project_type: info.project_type,
            status: info.status,
            last_opened,
            path: info.path,
            short_path,
            notes: info.notes,
        }
    }
}

glib::wrapper! {
    pub struct ProjectInfo(ObjectSubclass<imp::ProjectInfo>);
}

impl ProjectInfo {
    pub fn new(inner: ProjectInfoInner) -> Self {
        let info: Self = Object::builder().build();
        info.imp().inner.set(inner).unwrap();
        info
    }
    pub fn name(&self) -> &str {
        &self.imp().inner.get().unwrap().name
    }
    pub fn project_type(&self) -> &str {
        &self.imp().inner.get().unwrap().project_type
    }
    pub fn status(&self) -> &str {
        &self.imp().inner.get().unwrap().status
    }
    pub fn last_opened(&self) -> Option<glib::DateTime> {
        self.imp().inner.get().unwrap().last_opened.clone()
    }
    pub fn path(&self) -> &str {
        &self.imp().inner.get().unwrap().path
    }
    pub fn short_path(&self) -> &str {
        &self.imp().inner.get().unwrap().short_path
    }
    pub fn notes(&self) -> &str {
        &self.imp().inner.get().unwrap().notes
    }
}