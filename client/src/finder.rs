use crate::*;

#[derive(Properties, Clone, PartialEq)]
pub struct FinderProps {
    pub on_find: Callback<Option<String>>,
}

#[function_component]
pub fn Finder(props: &FinderProps) -> Html {
    let key = use_state(|| <Option<String>>::None);
    let change_key = {
        let key = key.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            let value = input.value();
            // log!(format!("key change: {:?}", value));
            let value = value.trim();
            let value = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
            // log!(format!("key change final: {:?}", value));
            key.set(value);
        })
    };
    let props = props.clone();
    html! { <>
        <div>
            <input type="text" placeholder="recipe id" oninput={change_key}/>
            <button onclick={move |_| props.on_find.emit((*key).clone())}>
                {"Find this recipe"}
            </button>
        </div>
    </> }
}
