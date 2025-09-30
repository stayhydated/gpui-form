use gpui_form::core::registry::GpuiFormShape;
use gpui_form_prototyping_core::{
    code_gen::FormShapeAdapter,
    implementations::{ComponentIdentities as _, ComponentShape as _},
};
use heck::ToSnakeCase as _;

use quote::{format_ident, quote};
use std::{fs, path::Path};

// import targetted lib to get inventory registrations
#[allow(unused_imports)]
use some_lib::*;

fn main() {
    let output_dir = &Path::new(env!("CARGO_MANIFEST_DIR")).join("output");
    fs::create_dir_all(output_dir).expect("Failed to create output directory");
    println!("Generating forms in: {}", output_dir.display());

    for struct_info in inventory::iter::<GpuiFormShape>() {
        println!("Thing : {:?}", struct_info);
        let syn_file = layout(struct_info);
        let struct_snek_case_name = struct_info.struct_name.to_snake_case();
        let file_path = output_dir.join(format!("{}.rs", struct_snek_case_name));

        let formatted_code = prettyplease::unparse(&syn_file);

        fs::write(&file_path, formatted_code)
            .unwrap_or_else(|_| panic!("Failed to write file: {}", file_path.display()));

        println!("Generated and formatted: {}", file_path.display());
    }
    println!("Form generation complete.");
}

fn layout(data: &GpuiFormShape) -> syn::File {
    let adapter = FormShapeAdapter::new(data);

    let struct_name_str = adapter.identities.struct_name();
    let context_str = format!("{}Form", struct_name_str);
    let struct_name_ident = adapter.identities.struct_name_ident();
    let struct_name_uw_ident = format_ident!("{}FormValueHolder", struct_name_ident);
    let struct_name_form_ident = adapter.identities.struct_form_ident();
    let struct_name_form_fields_ident = adapter.identities.struct_form_fields_ident();
    let form_id_literal = adapter.identities.form_id_literal();

    let struct_name_path_qualifier =
        syn::parse_str::<syn::Ident>(&adapter.identities.struct_name().to_snake_case()).unwrap();
    let target_types_import = quote! {
      use some_lib::structs::#struct_name_path_qualifier::*;
    };

    let component_creations_tokens = adapter.cx_new_calls().unwrap_or_default();

    let field_initializers_tokens = adapter.field_initializers().unwrap_or_default();

    let render_children_tokens = adapter.child_elements();

    let subscription_calls_tokens = adapter.subscription_calls().unwrap_or_default();

    let (subscriptions_field, subscriptions_init) = if subscription_calls_tokens.is_empty() {
        (quote! {}, quote! {})
    } else {
        (
            quote! {
                _subscriptions: Vec<Subscription>,
            },
            quote! {
              _subscriptions,
            },
        )
    };

    let event_handlers_tokens = adapter.event_handlers().unwrap_or_default();

    let action_token = {
        let action_context_ident = format_ident!("{}_story", struct_name_path_qualifier);
        quote! {
          actions!(#action_context_ident, [Tab, TabPrev]);
        }
    };

    let import_tokens = quote! {
      #target_types_import
      use gpui::{
          App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
          KeyBinding, ParentElement as _, Render, Styled, Subscription, Window, actions,
      };
      use gpui_component::{
          AxisExt, Selectable, Sizable, Size,
          button::{Button, ButtonGroup},
          checkbox::Checkbox,
          date_picker::{DatePicker, DatePickerEvent, DatePickerState},
          divider::Divider,
          dropdown::{Dropdown, DropdownEvent, DropdownItem, DropdownState, SearchableVec},
          form::{form_field, v_form},
          h_flex,
          input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction, TextInput},
          switch::Switch,
          v_flex,
      };
      use rust_decimal::Decimal;
      use std::sync::{Arc, Mutex};
      use std::str::FromStr;
      use gpui_storybook::story::Story;
    };

    let layout_tokens = quote! {
      #import_tokens

      #action_token

      const CONTEXT: &str = #context_str;

      #[gpui_storybook::story_init]
      pub fn init(cx: &mut App) {
          cx.bind_keys([
              KeyBinding::new("shift-tab", TabPrev, Some(CONTEXT)),
              KeyBinding::new("tab", Tab, Some(CONTEXT)),
          ])
      }

      #[gpui_storybook::story]
      pub struct #struct_name_form_ident {
          original_data: Arc<#struct_name_ident>,
          current_data: #struct_name_uw_ident,
          fields: #struct_name_form_fields_ident,
          focus_handle: FocusHandle,
          #subscriptions_field
      }

      impl Focusable for #struct_name_form_ident {
          fn focus_handle(&self, cx: &App) -> FocusHandle {
              self.focus_handle.clone()
          }
      }

      impl gpui_storybook::Story for #struct_name_form_ident {
          fn title() -> String {
              #struct_name_ident::this_ftl()
          }

          fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
              Self::view(window, cx, #struct_name_ident::default())
          }
      }

      impl #struct_name_form_ident {
          pub fn view(window: &mut Window, cx: &mut App, original_data: #struct_name_ident) -> Entity<Self> {
              cx.new(|cx| Self::new(window, cx, original_data))
          }

          fn tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
              self.cycle_focus(true, window, cx);
          }

          fn tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
              self.cycle_focus(false, window, cx);
          }

          #event_handlers_tokens

          fn new(window: &mut Window, cx: &mut Context<Self>, original_data: #struct_name_ident) -> Self {
            #component_creations_tokens

            #subscription_calls_tokens

              Self {
                  original_data: Arc::new(original_data.clone()),
                  current_data: original_data.into(),
                  fields: #struct_name_form_fields_ident {
                    #field_initializers_tokens
                  },
                  focus_handle: cx.focus_handle(),
                  #subscriptions_init
              }
          }
      }

      impl Render for #struct_name_form_ident {
          fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
              v_flex()
                  .key_context(CONTEXT)
                  .id(#form_id_literal)
                  .on_action(cx.listener(Self::tab))
                  .on_action(cx.listener(Self::tab_prev))
                  .size_full()
                  .p_4()
                  .justify_start()
                  .gap_3()
                  .child(Divider::horizontal())
                  .child(
                      v_form()
                        #render_children_tokens
                  )
                  .child(Divider::horizontal())
                  .absolute()
                  .child(format!("{:?}", self.current_data))
          }
      }
    };
    syn::parse2(layout_tokens)
        .expect("Failed to parse generated tokens into syn::File for form scaffold")
}
