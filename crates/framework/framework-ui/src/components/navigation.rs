//! Explicit breadcrumb data, keyboard menus and menubars with checked link policy.
use super::{ChoiceMove, ChoiceState, ControlError, Popover, PopupRole, safe_href};
use leptos::prelude::*;

/// One breadcrumb; the final entry is current and never rendered as a link.
#[derive(Clone, Debug)]
pub struct Crumb {
    /// Escaped visible label.
    pub label: String,
    /// Optional checked navigation location.
    pub href: Option<String>,
}
/// Bounded breadcrumb trail; no recursive child scanning or route inference.
#[derive(Clone, Debug)]
pub struct BreadcrumbTrail(Vec<Crumb>);
impl BreadcrumbTrail {
    /// Admit <=100 labels and safe optional URLs, including an empty trail.
    /// # Errors
    /// Rejects excessive labels or unsafe locations before rendering.
    pub fn new(items: Vec<Crumb>) -> Result<Self, ControlError> {
        if items.len() > 100
            || items.iter().any(|item| {
                item.label.len() > 1024
                    || item
                        .href
                        .as_ref()
                        .is_some_and(|href| href.len() > 2048 || !safe_href(href))
            })
        {
            return Err(ControlError);
        }
        Ok(Self(items))
    }
}
/// Native breadcrumb navigation with a visible decorative separator and current page.
#[component]
pub fn Breadcrumbs(
    /// Accessible navigation name.
    #[prop(into)]
    label: String,
    /// Explicit validated trail in document order.
    trail: BreadcrumbTrail,
    /// Escaped decorative separator; omitted from the accessible name.
    #[prop(default="/".to_owned())]
    separator: String,
) -> impl IntoView {
    let count = trail.0.len();
    view! {<nav aria-label=label><ol class="sp-breadcrumbs">{trail.0.into_iter().enumerate().map(|(index,item)|{
        let current=index+1==count;let linked=item.href.filter(|_|!current);
        view!{<li>{(index>0).then(||view!{<span aria-hidden="true">{separator.clone()}</span>})}
            {match linked{Some(href)=>view!{<a href=href>{item.label}</a>}.into_any(),None=>view!{<span aria-current=current.then_some("page")>{item.label}</span>}.into_any()}}</li>}
    }).collect_view()}</ol></nav>}
}
/// Keyboard menu contents: arrows/Home/End/typeahead, one Tab stop, disabled admission.
/// The caller places it inside a Popover role=Menu and supplies action handlers.
#[component]
pub fn Menu(
    /// Shared menuitem ID prefix.
    #[prop(into)]
    id: String,
    /// Root-owned enabled-choice state.
    state: ChoiceState,
    /// Controlled parent popover state.
    open: RwSignal<bool>,
    /// App-owned action, called once only for an admitted selection.
    action: Callback<String>,
    /// Optional menubar left/right transition; standalone menus leave it absent.
    #[prop(optional)]
    horizontal: Option<Callback<bool>>,
) -> impl IntoView {
    let search = RwSignal::new(super::TypeaheadState::default());
    view! {<div class="sp-menu-items">{state.options().entries().iter().enumerate().map(|(index,option)|{
        let key=option.value.clone();let prefix=id.clone();
        view!{<button id=format!("{id}-item-{index}") type="button" role="menuitem" disabled=option.disabled
            tabindex=move ||if state.focused()==Some(index){0}else{-1} on:focus=move |_|{state.focus(index);}
            on:click=move |_|{if state.choose(&key){open.set(false);action.run(key.clone());}}
            on:keydown=move |event:leptos::ev::KeyboardEvent|{
                let next=match event.key().as_str(){"ArrowDown"=>state.navigate(ChoiceMove::Next,false),"ArrowUp"=>state.navigate(ChoiceMove::Previous,false),"Home"=>state.navigate(ChoiceMove::First,false),"End"=>state.navigate(ChoiceMove::Last,false),key if key.chars().count()==1 && !event.ctrl_key() && !event.meta_key() && key!=" "=>{let mut prefix=None;search.update(|buffer|prefix=buffer.push(key,event.time_stamp()));prefix.and_then(|prefix|state.typeahead(&prefix))},_=>None};
                if let Some(next)=next{event.prevent_default();#[cfg(target_arch="wasm32")]super::dialog_dom::focus_id(&format!("{prefix}-item-{next}"));#[cfg(not(target_arch="wasm32"))]let _=(&prefix,next);}
                if event.key()=="Tab"{open.set(false);}
                if matches!(event.key().as_str(),"ArrowLeft"|"ArrowRight") && let Some(horizontal)=horizontal{event.prevent_default();horizontal.run(event.key()=="ArrowRight");}
            }>{option.label.clone()}</button>}
    }).collect_view()}</div>}
}
/// A checked menubar entry owns a separate, bounded menu-choice controller.
#[derive(Clone, Copy)]
pub struct MenuEntry {
    /// Choice controller for the corresponding menu.
    pub choices: ChoiceState,
}
/// Horizontal menubar. Left/right/Home/End rove triggers; Down/Enter open a menu.
#[component]
pub fn Menubar(
    /// DOM prefix for every associated trigger/popup/item.
    #[prop(into)]
    id: String,
    /// Accessible menubar name.
    #[prop(into)]
    label: String,
    /// Bounded trigger labels/disabled policy.
    state: ChoiceState,
    /// Exactly one entry per trigger; mismatches render no menus.
    entries: Vec<MenuEntry>,
    /// App-owned selected action.
    action: Callback<String>,
) -> impl IntoView {
    let options = state.options();
    let valid = entries.len() == options.entries().len();
    let openings = StoredValue::new(
        (0..entries.len())
            .map(|_| RwSignal::new(false))
            .collect::<Vec<_>>(),
    );
    view! {<div role="menubar" aria-label=label class="sp-actions">{options.entries().iter().enumerate().filter(|_|valid).map(|(index,option)|{
        let choices=entries[index].choices;let open=openings.get_value()[index];let trigger=format!("{id}-trigger-{index}");let popup=format!("{id}-popup-{index}");let menu=format!("{id}-menu-{index}");let focus=entries[index].choices.focused().map(|i|format!("{menu}-item-{i}")).unwrap_or_default();let prefix=id.clone();let horizontal_prefix=id.clone();
        let horizontal=Callback::new(move |forward|{
            if let Some(next)=state.navigate(if forward{ChoiceMove::Next}else{ChoiceMove::Previous},false){
                for signal in openings.get_value(){signal.set(false);}
                openings.get_value()[next].set(true);
                #[cfg(target_arch="wasm32")]super::dialog_dom::focus_id(&format!("{horizontal_prefix}-trigger-{next}"));
                #[cfg(not(target_arch="wasm32"))]let _=(&horizontal_prefix,next);
            }
        });
        view!{<div role="none"><button id=trigger.clone() type="button" role="menuitem" aria-haspopup="menu" aria-controls=popup.clone() aria-expanded=move ||open.get().to_string()
            disabled=option.disabled tabindex=move ||if state.focused()==Some(index){0}else{-1}
            on:focus=move |_|{state.focus(index);} on:click=move |_|open.update(|v|*v= !*v)
            on:keydown=move |event:leptos::ev::KeyboardEvent|{
                let next=match event.key().as_str(){"ArrowRight"=>state.navigate(ChoiceMove::Next,false),"ArrowLeft"=>state.navigate(ChoiceMove::Previous,false),"Home"=>state.navigate(ChoiceMove::First,false),"End"=>state.navigate(ChoiceMove::Last,false),_=>None};
                if let Some(next)=next{event.prevent_default();open.set(false);#[cfg(target_arch="wasm32")]super::dialog_dom::focus_id(&format!("{prefix}-trigger-{next}"));#[cfg(not(target_arch="wasm32"))]let _=(&prefix,next);}
                if event.key()=="ArrowDown"{event.prevent_default();open.set(true);}
            }>{option.label.clone()}</button>
            <Popover id=popup anchor=trigger label=option.label.clone() open role=PopupRole::Menu initial_focus=focus><Menu id=menu state=choices open action horizontal/></Popover></div>}
    }).collect_view()}</div>}
}
