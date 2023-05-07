use std::fmt::Display;

use gtk::glib::clone;
use gtk::prelude::*;

pub trait ListColumn: Copy + Display + 'static {
    type ModelItem: IsA<gtk::glib::Object>;
    fn setup_content(&self) -> gtk::Widget;
    fn bind_content(&self, widget: gtk::Widget, item: Self::ModelItem);
    fn sort(&self, a: &Self::ModelItem, b: &Self::ModelItem) -> gtk::Ordering;
}

pub trait ListDetail {
    type ModelItem: IsA<gtk::glib::Object>;
    fn setup_content() -> gtk::Widget;
    fn bind_content(widget: gtk::Widget, item: Self::ModelItem);
}

pub trait ListProvider {
    type Model: IsA<gtk::gio::ListModel>;
    type ModelItem: IsA<gtk::glib::Object>;
    type Column: ListColumn<ModelItem = Self::ModelItem>;
    type Detail: ListDetail<ModelItem = Self::ModelItem>;
    fn model(&self) -> Self::Model;
    fn columns(&self) -> &[Self::Column];
}

fn list_item_factory_for_column<C: ListColumn>(column: C) -> gtk::SignalListItemFactory {
    let list_item_factory = gtk::SignalListItemFactory::new();
    list_item_factory.connect_setup(move |_factory, object| {
        let list_item: &gtk::ListItem = object.downcast_ref().unwrap();
        let child = column.setup_content();
        list_item.set_child(Some(&child));
    });
    list_item_factory.connect_bind(move |_factory, object| {
        let list_item: &gtk::ListItem = object.downcast_ref().unwrap();
        let item = list_item.item().and_downcast::<C::ModelItem>().unwrap();
        column.bind_content(list_item.child().unwrap(), item);
    });
    list_item_factory
}

fn build_detail_pane<P: ListProvider, V: IsA<gtk::Widget>>(model: gtk::SingleSelection, view: &V, detail_width: i32) -> gtk::Paned {
    let detail = P::Detail::setup_content();
    detail.set_margin_start(6);
    detail.set_margin_end(6);
    let detail_scroll = gtk::ScrolledWindow::builder()
        .width_request(detail_width)
        .child(&detail)
        .build();
    let detail_frame = gtk::Frame::builder().child(&detail_scroll).build();
    detail_frame.set_visible(false);

    model.connect_selection_changed(clone!(@strong detail_frame => move |model, _, _| {
        let item = model.selected_item().and_downcast::<P::ModelItem>();
        detail_frame.set_visible(item.is_some());
        if let Some(item) = item {
            P::Detail::bind_content(detail.clone(), item);
        }
    }));

    // Right-click deselect
    // let right_click = gtk::GestureClick::new();
    // right_click.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);
    // right_click.connect_pressed(move |gesture, _, _, _| {
    //     gesture.set_state(gtk::EventSequenceState::Claimed);
    //     model.unselect_item(model.selected());
    // });
    // view.add_controller(right_click);

    let view_scroll = gtk::ScrolledWindow::builder().child(view).build();
    gtk::Paned::builder()
        .orientation(gtk::Orientation::Horizontal)
        .hexpand(true)
        .vexpand(true)
        .resize_start_child(true)
        .shrink_start_child(false)
        .start_child(&view_scroll)
        .resize_end_child(false)
        .shrink_end_child(false)
        .end_child(&detail_frame)
        .build()
}

pub fn build_list_view<P: ListProvider>(
    provider: &P,
    detail_width: i32,
) -> (gtk::Paned, gtk::ListView) {
    let model = gtk::SingleSelection::builder()
        .autoselect(false)
        .can_unselect(true)
        .model(&provider.model())
        .build();
    let column = provider.columns()[0];
    let list_item_factory = list_item_factory_for_column(column);
    let view = gtk::ListView::new(Some(model.clone()), Some(list_item_factory));
    let pane = build_detail_pane::<P, _>(model, &view, detail_width);
    (pane, view)
}

pub fn build_column_view<P: ListProvider>(provider: &P, detail_width: i32) -> (gtk::Paned, gtk::ColumnView) {
    let view = gtk::ColumnView::new(gtk::SelectionModel::NONE.cloned());
    let model = gtk::SingleSelection::builder()
        .autoselect(false)
        .can_unselect(true)
        .model(&gtk::SortListModel::new(
            Some(provider.model()),
            view.sorter(),
        ))
        .build();
    view.set_model(Some(&model));

    let mut first_column = None;
    for &column in provider.columns() {
        let list_item_factory = list_item_factory_for_column(column);
        let view_column =
            gtk::ColumnViewColumn::new(Some(&column.to_string()), Some(list_item_factory));
        view_column.set_sorter(Some(&gtk::CustomSorter::new(move |a, b| {
            let a = a.downcast_ref::<P::ModelItem>().unwrap();
            let b = b.downcast_ref::<P::ModelItem>().unwrap();
            column.sort(a, b)
        })));
        view.append_column(&view_column);
        if first_column.is_none() {
            view_column.set_expand(true);
            first_column = Some(view_column);
        }
    }
    view.sort_by_column(first_column.as_ref(), gtk::SortType::Ascending);

    let pane = build_detail_pane::<P, _>(model, &view, detail_width);
    (pane, view)
}
