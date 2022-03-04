pub mod gadget_selector;
pub mod listener;
pub mod app_events;

pub const HELP_MSG: &str = r#"INSTRUCTIONS:
1. Navigate to a web page and initialize the selector gadget by pressing `S`.
2. Use the selector gadget to set the elements to scrape.
3. Press `A` to get the selected elements.

HOTKEYS:
-  S : Initialize selector gadget.
-  A : Get selected elements. 
-  H : Show this help message. 
- Esc: quit the program."#;
