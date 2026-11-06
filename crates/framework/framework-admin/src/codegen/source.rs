//! Source UI emission uses portable descriptors, never Typed contracts or private source models.
use super::{
    Component,
    templates::{Asset, Renderer, literal, source_error},
};
use stackpress_framework_schema::{
    Role, SchemaResult, SourceUiCatalog,
    codegen::{DefinitionRegistry, SourceGraph, source_components, source_module},
};
use stackpress_idea_transformer::{OutputPlan, Result};

/// Reuse existing controls and the source argument definitions; unsupported controls stay explicit.
/// # Errors
/// Preserves duplicate/import/prop definition errors.
pub fn source_definitions() -> SchemaResult<DefinitionRegistry> {
    let source = source_components();
    let mut registry = DefinitionRegistry::standard();
    for Component {
        definition,
        mut import,
    } in super::components().into_iter().filter(|component| {
        !source
            .iter()
            .any(|source| source.name == component.definition.name)
    }) {
        if definition.name == "field.switch" {
            import.module = "stackpress_framework_admin::source::roles".into();
            import.symbol = "SwitchField".into();
        }
        registry = registry.with_component(definition, Some(import))?;
    }
    for mut definition in source {
        let symbol = match definition.name.as_str() {
            "field.editor" => "EditorField",
            "field.fieldset" => "FieldsetField",
            "view.fieldset" => "FieldsetView",
            "field.code" | "field.json" | "field.markdown" | "field.mask" => "RichField",
            "field.country" | "field.currency" => "CountryField",
            "field.color" => "ColorField",
            "view.country" | "view.currency" => "CountryView",
            "view.code" | "view.json" | "view.markdown" => "RichView",
            "view.formula" => "FormulaView",
            "view.carousel" | "view.film" => "MediaView",
            "field.file" | "field.image" | "field.filelist" | "field.imagelist" => "FileField",
            "field.radio" => "RadioField",
            "field.small" => "SmallField",
            "field.suggest" => "SuggestField",
            "view.html" => "HtmlView",
            "view.chars" | "view.words" => "OverflowView",
            "view.color" => "ColorView",
            "view.email" => "EmailView",
            "view.transform" => "TransformView",
            "view.rating" => "RatingView",
            "view.rel" | "view.relative" => "RelativeView",
            "view.comma" | "view.line" | "view.spread" => "SpreadView",
            "view.list" | "view.ol" | "view.ul" => "ListView",
            "view.tabular" => "TabularView",
            "field.input" | "field.url" | "field.password" | "field.email" => "InputField",
            "field.relation" => "RelationField",
            "field.metadata" => "MetadataField",
            "field.stringlist" => "TextListField",
            "field.datelist" | "field.datetimelist" | "field.numberlist" | "field.timelist" => {
                "SourceTextListField"
            }
            "field.select" => "SelectField",
            "view.overflow" => "OverflowView",
            "view.date" => "DateView",
            "view.image" => "ImageView",
            "view.link" => "LinkView",
            "view.tags" => "TagsView",
            "view.template" => "TemplateView",
            "view.metadata" => "MetadataView",
            _ => continue,
        };
        definition.available = true;
        let import = stackpress_framework_schema::codegen::ComponentImport {
            component: definition.key.clone(),
            module: "stackpress_framework_admin::source::roles".into(),
            symbol: symbol.into(),
            default: false,
        };
        registry = registry.with_component(definition, Some(import))?;
    }
    Ok(registry)
}

/// Emit source role wrappers over JSON rows and caller-owned drafts; no app serde_json edge.
/// # Errors
/// Rejects unavailable roles/imports before contributing files.
pub fn emit_source_roles(
    graph: &SourceGraph,
    ui: &SourceUiCatalog,
    definitions: &DefinitionRegistry,
    output: &mut OutputPlan,
) -> Result<()> {
    let mut renderer = Renderer::new().map_err(source_error)?;
    let mut files = Vec::new();
    for model in ui.models.iter().chain(&ui.fieldsets) {
        let metadata = graph
            .models()
            .iter()
            .chain(graph.fieldsets())
            .find(|item| item.name == model.id)
            .ok_or_else(|| {
                source_error(stackpress_framework_schema::SchemaError::new(
                    stackpress_framework_schema::SchemaErrorKind::Invalid,
                    "source UI model missing",
                ))
            })?;
        let mut roles = Vec::new();
        for role in [
            Role::Field,
            Role::Filter,
            Role::Span,
            Role::List,
            Role::View,
        ] {
            let nested_target = nested_target(ui, &model.id, role, definitions);
            let cells = role == Role::List || (role == Role::View && nested_target);
            let controls=model.fields.iter().filter_map(|field|field.roles.iter().find(|binding|binding.role==role).map(|binding|(field,binding)))
                .map(|(field,binding)|{
                    let import=super::bindings::import(binding,definitions)?;
                    if import.module=="stackpress_framework_admin::source::roles" && import.symbol=="DateView"
                        && binding.props.get("format").and_then(serde_json::Value::as_str)!=Some("m d, Y h:iA") {
                        return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Unsupported,"source date format requires an application adapter"));
                    }
                    if import.module=="stackpress_framework_admin::source::roles" && import.symbol=="MetadataView" && binding.props.contains_key("style") {
                        return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Unsupported,"source metadata styles require an application adapter"));
                    }
                    // Metadata may describe more than the selected shared adapter implements.
                    // Diagnose that boundary instead of silently discarding source styling/options.
                    if import.module=="stackpress_framework_admin::source::roles" && import.symbol=="SelectField"
                        && (binding.props.keys().any(|key|!matches!(key.as_str(),"options"|"placeholder"))
                            || binding.props.get("options").is_some_and(|value|value.as_array().is_none_or(|values|values.iter().any(|value|!value.is_string())))) {
                        return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Unsupported,"source select props require an application adapter"));
                    }
                    let nested=if import.module=="stackpress_framework_admin::source::roles" && matches!(import.symbol.as_str(),"FieldsetField"|"FieldsetView") {
                        let stackpress_framework_schema::ValueType::Fieldset(name)=&field.value_type.value else {
                            return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Invalid,"source fieldset role requires a fieldset type"));
                        };
                        Some(source_module(graph.fieldsets().iter().find(|fieldset|fieldset.name==*name).expect("normalized fieldset")).to_owned())
                    }else{None};
                    Ok(serde_json::json!({"wire":literal(&field.wire_name),"module":import.module,"symbol":import.symbol,"nested":nested.is_some(),"nested_module":nested.unwrap_or_default(),"nested_display":matches!(role,Role::List|Role::View),"nested_function":match role{Role::List=>"list_cells",Role::View=>"view_cells",_=>super::emit::role_name(role)},"row":matches!(role,Role::List|Role::View)&&import.module=="stackpress_framework_admin::source::roles"&&matches!(import.symbol.as_str(),"TemplateView"|"FormulaView")}))
                }).collect::<SchemaResult<Vec<_>>>().map_err(source_error)?;
            let uses_row = controls
                .iter()
                .any(|control| control["row"].as_bool() == Some(true));
            roles.push(serde_json::json!({"name":super::emit::role_name(role),"function":match role {Role::List=>"list_cells",Role::View if cells=>"view_cells",_=>super::emit::role_name(role)},"cells":cells,"prefix":nested_target && !matches!(role,Role::View|Role::List),"list":role==Role::List,"variant":format!("{role:?}"),"display":matches!(role,Role::List|Role::View),"uses_row":uses_row,"has_controls":!controls.is_empty(),"controls":controls}));
        }
        let text=renderer.render(Asset::SourceRoles,&serde_json::json!({"model":literal(&serde_json::to_string(model).expect("portable descriptor")),"rendering":roles.iter().any(|role|role["has_controls"].as_bool()==Some(true)),"roles":roles})).map_err(source_error)?;
        files.push((source_module(metadata).to_owned(), text));
    }
    for (module, text) in files {
        output.write(format!("ui/{module}.rs"), text)?;
        output.contribute_module("ui", &module)?;
    }
    Ok(())
}

/// Emit ordinary source page functions and a data-only host catalog; registration stays app-owned.
/// # Errors
/// Rejects missing selection and preserves plan/template errors without DB or startup work.
pub fn emit_source_pages(
    graph: &SourceGraph,
    ui: &SourceUiCatalog,
    definitions: &DefinitionRegistry,
    output: &mut OutputPlan,
) -> Result<()> {
    emit_source_roles(graph, ui, definitions, output)?;
    let mut renderer = Renderer::new().map_err(source_error)?;
    let models=ui.models.iter().map(|model|{
        let metadata=graph.models().iter().find(|item|item.name==model.id).expect("projection model");
        let module=source_module(metadata);
        serde_json::json!({"module":module,"module_literal":literal(module),"name_literal":literal(&model.id),"label_literal":literal(&model.label),"plural_literal":literal(metadata.documentation.labels.get(1).unwrap_or(&model.label)),"icon_literal":metadata.documentation.icon.as_deref().map(literal).unwrap_or_else(||"null".into()),"soft_remove":metadata.fields.iter().any(|field|field.active)})
    }).collect::<Vec<_>>();
    // Catalog helper names avoid every actual model module, including Pages/Host/Catalog.
    let fresh = |mut name: String| {
        while graph
            .models()
            .iter()
            .chain(graph.fieldsets())
            .any(|model| source_module(model) == name)
        {
            name.insert(0, '_');
        }
        name
    };
    let pages = fresh("pages".into());
    let host = fresh("host".into());
    output.write(
        format!("ui/{pages}.rs"),
        renderer
            .render(Asset::SourcePages, &serde_json::json!({"models":models}))
            .map_err(source_error)?,
    )?;
    output.contribute_module("ui", &pages)?;
    output.write(
        format!("ui/{host}.rs"),
        renderer
            .render(Asset::SourceHost, &serde_json::json!({"models":models}))
            .map_err(source_error)?,
    )?;
    output.contribute_module("ui", &host)
}

/// Only actual nested-fieldset targets need the extra cell renderer or form-name adapter.
/// Keeping those additions local preserves existing unrelated generated-source fixture bytes.
fn nested_target(
    ui: &SourceUiCatalog,
    target: &str,
    role: Role,
    definitions: &DefinitionRegistry,
) -> bool {
    let symbol = if matches!(role, Role::View | Role::List) {
        "FieldsetView"
    } else {
        "FieldsetField"
    };
    // Incoming generated fieldset roles determine which child wrapper needs composition hooks.
    ui.models
        .iter()
        .chain(&ui.fieldsets)
        .flat_map(|model| &model.fields)
        .filter(|field| {
            matches!(&field.value_type.value,stackpress_framework_schema::ValueType::Fieldset(name) if name==target)
        })
        .any(|field| {
            field.roles.iter().filter(|binding| binding.role == role).any(|binding| {
                definitions.imports().iter().any(|import| {
                    import.component == binding.component
                        && import.module == "stackpress_framework_admin::source::roles"
                        && import.symbol == symbol
                })
            })
        })
}
