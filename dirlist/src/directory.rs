use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bytesize::ByteSize;
use gtk::gio::{File, FileInfo, FileType};
use gtk::glib;
use gtk::prelude::*;

use gtk_list_provider::{ListColumn, ListDetail, ListProvider};

struct NameColumn;

impl NameColumn {
    fn setup_content() -> gtk::Widget {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let icon = gtk::Image::new();
        widget.append(&icon);
        let label = gtk::Label::new(None);
        widget.append(&label);
        widget.upcast()
    }
    fn bind_content(widget: gtk::Widget, item: FileInfo) {
        let icon: gtk::Image = widget.first_child().and_downcast().unwrap();
        icon.set_gicon(item.icon().as_ref());
        let label: gtk::Label = icon.next_sibling().and_downcast().unwrap();
        label.set_text(&item.name().to_string_lossy());
    }
    fn sort(a: &FileInfo, b: &FileInfo) -> gtk::Ordering {
        if a.file_type() == FileType::Directory && b.file_type() != FileType::Directory {
            gtk::Ordering::Smaller
        } else if a.file_type() != FileType::Directory && b.file_type() == FileType::Directory {
            gtk::Ordering::Larger
        } else {
            a.name().cmp(&b.name()).into()
        }
    }
}

struct SizeColumn;

impl SizeColumn {
    fn setup_content() -> gtk::Widget {
        let label = gtk::Label::builder().hexpand(true).xalign(0.0).build();
        label.upcast()
    }
    fn bind_content(widget: gtk::Widget, item: FileInfo) {
        let label: gtk::Label = widget.downcast().unwrap();
        if item.file_type() == FileType::Regular {
            label.set_text(&ByteSize(item.size() as u64).to_string());
        } else {
            label.set_text("");
        }
    }
    fn sort(a: &FileInfo, b: &FileInfo) -> gtk::Ordering {
        a.size().cmp(&b.size()).into()
    }
}

struct ModifiedColumn;

impl ModifiedColumn {
    fn setup_content() -> gtk::Widget {
        let label = gtk::Label::builder().hexpand(true).xalign(0.0).build();
        label.upcast()
    }
    fn bind_content(widget: gtk::Widget, item: FileInfo) {
        let label: gtk::Label = widget.downcast().unwrap();
        let formatter = timeago::Formatter::new();
        let text = item
            .modification_date_time()
            .map(|dt| {
                let now = glib::DateTime::now(&dt.timezone()).unwrap();
                formatter.convert(Duration::from_micros(
                    now.difference(&dt).as_microseconds() as u64
                ))
            })
            .unwrap_or_default();
        label.set_text(&text);
    }
    fn sort(a: &FileInfo, b: &FileInfo) -> gtk::Ordering {
        a.modification_date_time()
            .cmp(&b.modification_date_time())
            .into()
    }
}

#[derive(Clone, Copy)]
pub enum DirectoryColumn {
    Name,
    Size,
    Modified,
}

impl fmt::Display for DirectoryColumn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DirectoryColumn::Name => write!(f, "Name"),
            DirectoryColumn::Size => write!(f, "Size"),
            DirectoryColumn::Modified => write!(f, "Modified"),
        }
    }
}

impl ListColumn for DirectoryColumn {
    type ModelItem = FileInfo;
    fn setup_content(&self) -> gtk::Widget {
        match self {
            DirectoryColumn::Name => NameColumn::setup_content(),
            DirectoryColumn::Size => SizeColumn::setup_content(),
            DirectoryColumn::Modified => ModifiedColumn::setup_content(),
        }
    }
    fn bind_content(&self, widget: gtk::Widget, item: Self::ModelItem) {
        match self {
            DirectoryColumn::Name => NameColumn::bind_content(widget, item),
            DirectoryColumn::Size => SizeColumn::bind_content(widget, item),
            DirectoryColumn::Modified => ModifiedColumn::bind_content(widget, item),
        }
    }
    fn sort(&self, a: &Self::ModelItem, b: &Self::ModelItem) -> gtk::Ordering {
        match self {
            DirectoryColumn::Name => NameColumn::sort(a, b),
            DirectoryColumn::Size => SizeColumn::sort(a, b),
            DirectoryColumn::Modified => ModifiedColumn::sort(a, b),
        }
    }
}

pub struct FileDetail;

impl ListDetail for FileDetail {
    type ModelItem = FileInfo;
    fn setup_content() -> gtk::Widget {
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
        let content = gtk::Label::new(None);
        outer.append(&content);
        outer.upcast()
    }
    fn bind_content(widget: gtk::Widget, item: Self::ModelItem) {
        let outer: gtk::Box = widget.downcast().unwrap();
        let top = outer.first_child().unwrap();
        let image: gtk::Image = top.first_child().unwrap().downcast().unwrap();
        let name: gtk::Label = image.next_sibling().unwrap().downcast().unwrap();
        let content: gtk::Label = top.next_sibling().unwrap().downcast().unwrap();
        image.set_gicon(item.icon().as_ref());
        name.set_markup(&format!(
            "<big>{}</big>",
            gtk::glib::markup_escape_text(&item.name().to_string_lossy())
        ));
        if item.file_type() == FileType::Directory {
            content.set_text("Directory");
        } else {
            content.set_text(&ByteSize(item.size() as u64).to_string());
        }
    }
}

#[derive(Clone)]
pub struct DirectoryProvider {
    pub(crate) directory: gtk::DirectoryList,
}

impl DirectoryProvider {
    pub fn new(path: &Path) -> Self {
        DirectoryProvider {
            directory: gtk::DirectoryList::new(
                Some("standard::name,standard::icon,standard::size,time::modified"),
                Some(&File::for_path(path)),
            ),
        }
    }
    pub fn path(&self) -> PathBuf {
        self.directory.file().unwrap().path().unwrap()
    }
    pub fn set_path(&self, path: &Path) {
        self.directory.set_file(Some(&File::for_path(path)))
    }
}

impl ListProvider for DirectoryProvider {
    type Model = gtk::DirectoryList;
    type ModelItem = FileInfo;
    type Column = DirectoryColumn;
    type Detail = FileDetail;
    fn model(&self) -> Self::Model {
        self.directory.clone()
    }
    fn columns(&self) -> &[Self::Column] {
        &[
            DirectoryColumn::Name,
            DirectoryColumn::Size,
            DirectoryColumn::Modified,
        ]
    }
}
