macro_rules! declare {
    ($name:ident) => {
        pub const $name: &'static str = egui_material_icons::icons::$name.codepoint;
    };
}

declare!(ICON_ADD);
declare!(ICON_ARROW_BACK);
declare!(ICON_ARROW_BACK_2);
declare!(ICON_ARROW_DOWNWARD);
declare!(ICON_ARROW_FORWARD);
declare!(ICON_ARROW_UPWARD);
declare!(ICON_BACKSPACE);
declare!(ICON_CALENDAR_MONTH);
declare!(ICON_CANCEL);
declare!(ICON_CHECK);
declare!(ICON_CLOSE);
declare!(ICON_CONTENT_COPY);
declare!(ICON_CONTENT_PASTE);
declare!(ICON_DELETE);
declare!(ICON_DIALOGS);
declare!(ICON_EDIT);
declare!(ICON_FILTER_ALT);
declare!(ICON_FORMAT_LIST_NUMBERED);
declare!(ICON_FULLSCREEN);
declare!(ICON_GRID_ON);
declare!(ICON_HELP);
declare!(ICON_LIST);
declare!(ICON_MAGNIFICATION_LARGE);
declare!(ICON_MAGNIFICATION_SMALL);
declare!(ICON_PLAY_ARROW);
declare!(ICON_PUBLIC);
declare!(ICON_PUBLISH);
declare!(ICON_SEARCH);
declare!(ICON_SETTINGS);
declare!(ICON_SORT_BY_ALPHA);
declare!(ICON_UNDO);
declare!(ICON_VIEW_DAY);
declare!(ICON_WARNING);
