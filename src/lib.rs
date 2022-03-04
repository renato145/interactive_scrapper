pub mod gadget_selector;
pub mod listener;
pub mod app_events;

pub const HELP_MSG: &str = r#"INSTRUCTIONS:
1. Navigate to a web page and initialize the selector gadget by pressing `Alt+S`.
2. Use the selector gadget to set the elements to scrape.
3. Press `Alt+A` to get the selected elements.
            
HOTKEYS:
- Alt+S  : Initialize selector gadget.
- Alt+A  : Get selected elements. 
- Alt+H  : Show this help message. 
- Alt+Esc: quit the program."#;
