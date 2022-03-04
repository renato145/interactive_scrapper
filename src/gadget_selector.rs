use fantoccini::{Client, Locator};

#[derive(Debug)]
pub enum UserSelection {
    Empty,
    Selection(String),
    MissingGadget,
}

pub async fn get_user_selection(
    client: &mut Client,
) -> Result<UserSelection, fantoccini::error::CmdError> {
    match client
        .find_all(Locator::Id("_sg_path_field"))
        .await?
        .get_mut(0)
    {
        Some(gadget) => {
            match gadget
                .prop("value")
                .await?
                .map(|s| {
                    if s == "No valid path found." {
                        None
                    } else {
                        Some(s)
                    }
                })
                .flatten()
            {
                Some(selection) => Ok(UserSelection::Selection(selection)),
                None => Ok(UserSelection::Empty),
            }
        }
        None => Ok(UserSelection::MissingGadget),
    }
}

/// Initialize https://selectorgadget.com on current page
#[tracing::instrument(skip(client))]
pub async fn initialize_gadget_selector(
    client: &mut Client,
) -> Result<(), fantoccini::error::CmdError> {
    if let UserSelection::MissingGadget = get_user_selection(client).await? {
        tracing::info!("Setting selector gadget script...");
        client
            .execute_async(
                r#"
                const [callback] = arguments;
                (function () {
                var s = document.createElement("div");
                s.innerHTML = "Loading...";
                s.style.color = "black";
                s.style.padding = "20px";
                s.style.position = "fixed";
                s.style.zIndex = "9999";
                s.style.fontSize = "3.0em";
                s.style.border = "2px%20solid%20black";
                s.style.right = "40px";
                s.style.top = "40px";
                s.setAttribute("class", "selector_gadget_loading");
                s.style.background = "white";
                document.body.appendChild(s);
                s = document.createElement("script");
                s.setAttribute("type", "text/javascript");
                s.setAttribute(
                    "src",
                    "https://dv0akt2986vzh.cloudfront.net/unstable/lib/selectorgadget.js"
                );
                document.body.appendChild(s);
                })();
                callback();
                "#,
                vec![],
            )
            .await?;
        tracing::info!("Selector gadget ready");
    } else {
        tracing::info!("Gadget already initialized...");
    }
    Ok(())
}
