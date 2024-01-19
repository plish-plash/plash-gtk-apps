use std::fmt::Display;

use gtk::glib;
use gtk::prelude::*;

pub trait ListContent: 'static {
    type ModelItem: IsA<gtk::glib::Object>;
    fn setup_content(&self) -> gtk::Widget;
    fn bind_content(&self, widget: gtk::Widget, item: Self::ModelItem);
}

pub trait ListColumn: ListContent + Clone + Display {
    fn sort(&self, a: &Self::ModelItem, b: &Self::ModelItem) -> gtk::Ordering;
}

pub trait ListProvider {
    type Model: IsA<gtk::gio::ListModel>;
    type ModelItem: IsA<gtk::glib::Object>;
    type Column: ListColumn<ModelItem = Self::ModelItem>;
    type Detail: ListContent<ModelItem = Self::ModelItem>;
    fn model(&self) -> Self::Model;
    fn columns(&self) -> Vec<Self::Column>;
    fn detail(&self) -> Self::Detail;
}

fn list_item_factory_for_column<C: ListColumn>(column: C) -> gtk::SignalListItemFactory {
    let list_item_factory = gtk::SignalListItemFactory::new();
    list_item_factory.connect_setup(glib::clone!(@strong column => move |_factory, object| {
        let list_item: &gtk::ListItem = object.downcast_ref().unwrap();
        let child = column.setup_content();
        list_item.set_child(Some(&child));
    }));
    list_item_factory.connect_bind(move |_factory, object| {
        let list_item: &gtk::ListItem = object.downcast_ref().unwrap();
        let item = list_item.item().and_downcast::<C::ModelItem>().unwrap();
        column.bind_content(list_item.child().unwrap(), item);
    });
    list_item_factory
}

fn build_detail_pane<P: ListProvider, V: IsA<gtk::Widget>>(
    provider: &P,
    model: gtk::SingleSelection,
    view: &V,
    detail_width: i32,
) -> gtk::Paned {
    let detail = provider.detail();
    let detail_widget = detail.setup_content();
    detail_widget.set_margin_top(6);
    detail_widget.set_margin_bottom(6);
    detail_widget.set_margin_start(6);
    detail_widget.set_margin_end(6);
    let detail_scroll = gtk::ScrolledWindow::builder()
        .width_request(detail_width)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&detail_widget)
        .visible(false)
        .build();

    model.connect_selection_changed(glib::clone!(@strong detail_scroll => move |model, _, _| {
        let item = model.selected_item().and_downcast::<P::ModelItem>();
        detail_scroll.set_visible(item.is_some());
        if let Some(item) = item {
            detail.bind_content(detail_widget.clone(), item);
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
        .css_classes(["content-pane"])
        .resize_start_child(true)
        .shrink_start_child(false)
        .start_child(&view_scroll)
        .resize_end_child(false)
        .shrink_end_child(false)
        .end_child(&detail_scroll)
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
    let column = provider.columns().into_iter().next().unwrap();
    let list_item_factory = list_item_factory_for_column(column);
    let view = gtk::ListView::new(Some(model.clone()), Some(list_item_factory));
    let pane = build_detail_pane(provider, model, &view, detail_width);
    (pane, view)
}

pub fn build_column_view<P: ListProvider>(
    provider: &P,
    detail_width: i32,
) -> (gtk::Paned, gtk::ColumnView) {
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
    for column in provider.columns() {
        let list_item_factory = list_item_factory_for_column(column.clone());
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

    let pane = build_detail_pane(provider, model, &view, detail_width);
    (pane, view)
}

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("could not connect to a display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
