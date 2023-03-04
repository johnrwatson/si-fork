use std::path::PathBuf;

use si_pkg::{
    PkgSpec, PropSpec, PropSpecKind, SchemaSpec, SchemaVariantSpec, SiPkg, SiPkgError, SiPkgProp,
    SiPkgSchema, SiPkgSchemaVariant, SpecError,
};
use thiserror::Error;

use crate::{
    component::ComponentKind,
    installed_pkg::{
        asset::{InstalledPkgAsset, InstalledPkgAssetKind, InstalledPkgAssetTyped},
        InstalledPkg, InstalledPkgError, InstalledPkgId,
    },
    schema::{
        variant::definition::{hex_color_to_i64, SchemaVariantDefinitionError},
        SchemaUiMenu,
    },
    DalContext, Prop, PropError, PropId, PropKind, Schema, SchemaError, SchemaId, SchemaVariant,
    SchemaVariantError, SchemaVariantId, StandardModel, StandardModelError,
};

#[derive(Debug, Error)]
pub enum PkgError {
    #[error(transparent)]
    Pkg(#[from] SiPkgError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    SchemaVariantDefinition(#[from] SchemaVariantDefinitionError),
    #[error(transparent)]
    PkgSpec(#[from] SpecError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("Package with that hash already installed: {0}")]
    PackageAlreadyInstalled(String),
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    #[error("Installed schema id {0} does not exist")]
    InstalledSchemaMissing(SchemaId),
    #[error("Installed schema variant {0} does not exist")]
    InstalledSchemaVariantMissing(SchemaVariantId),
}

pub type PkgResult<T> = Result<T, PkgError>;

pub async fn import_pkg_from_pkg(ctx: &DalContext, pkg: &SiPkg, file_name: &str) -> PkgResult<()> {
    // We have to write the installed_pkg row first, so that we have an id, and rely on transaction
    // semantics to remove the row if anything in the installation process fails
    let root_hash = pkg.hash()?.to_string();

    if !InstalledPkg::find_by_attr(ctx, "root_hash", &root_hash)
        .await?
        .is_empty()
    {
        return Err(PkgError::PackageAlreadyInstalled(root_hash));
    }

    let installed_pkg = InstalledPkg::new(ctx, &file_name, pkg.hash()?.to_string()).await?;

    // TODO: gather up a record of what wasn't installed and why (the id of the package that
    // already contained the schema or variant)
    for schema_spec in pkg.schemas()? {
        create_schema(ctx, schema_spec, *installed_pkg.id()).await?;
    }

    Ok(())
}

pub async fn import_pkg(
    ctx: &DalContext,
    pkg_file_path: impl Into<PathBuf> + Clone,
) -> PkgResult<SiPkg> {
    let pkg_file_path_str = Into::<PathBuf>::into(pkg_file_path.clone())
        .to_string_lossy()
        .to_string();

    let pkg = SiPkg::load_from_file(pkg_file_path).await?;

    import_pkg_from_pkg(ctx, &pkg, &pkg_file_path_str).await?;

    Ok(pkg)
}

// TODO(fnichol): another first-pass function with arguments. At the moment we're passing a list of
// `SchemaVariantId`s in an effort to export specific schema/variant combos but this will change in
// the future to be more encompassing. And yes, to many function args, way too many--and they're
// all `String`s
pub async fn export_pkg(
    _ctx: &DalContext,
    pkg_file_path: impl Into<PathBuf>,
    name: impl Into<String>,
    version: impl Into<String>,
    description: Option<impl Into<String>>,
    created_by: impl Into<String>,
    _variant_ids: Vec<SchemaVariantId>,
) -> PkgResult<()> {
    let mut spec_builder = PkgSpec::builder();
    spec_builder
        .name(name)
        .version(version)
        .created_by(created_by);
    if let Some(description) = description {
        spec_builder.description(description);
    }

    // // TODO(fnichol): this is merely an example to see if a chained builder pattern works and
    // // compile--and it does!
    // //
    spec_builder
        .schema(
            SchemaSpec::builder()
                .name("Laika")
                .category("Space Dogs")
                .variant(
                    SchemaVariantSpec::builder()
                        .name("v0")
                        .color("4695E7")
                        .prop(
                            PropSpec::builder()
                                .name("age")
                                .kind(PropSpecKind::Number)
                                .build()?,
                        )
                        .prop(
                            PropSpec::builder()
                                .name("praises")
                                .kind(PropSpecKind::Array)
                                .type_prop(
                                    PropSpec::builder()
                                        .name("praiseString")
                                        .kind(PropSpecKind::String)
                                        .build()?,
                                )
                                .build()?,
                        )
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let spec = spec_builder.build()?;
    let pkg = SiPkg::load_from_spec(spec)?;
    pkg.write_to_file(pkg_file_path).await?;

    Ok(())
}

async fn create_schema(
    ctx: &DalContext,
    schema_spec: SiPkgSchema<'_>,
    installed_pkg_id: InstalledPkgId,
) -> PkgResult<()> {
    let hash = schema_spec.hash().to_string();
    let existing_schema =
        InstalledPkgAsset::list_for_kind_and_hash(ctx, InstalledPkgAssetKind::Schema, &hash)
            .await?
            .pop();

    let mut schema = match existing_schema {
        None => {
            let schema = Schema::new(ctx, schema_spec.name(), &ComponentKind::Standard).await?;
            let ui_menu =
                SchemaUiMenu::new(ctx, schema_spec.name(), schema_spec.category()).await?;
            ui_menu.set_schema(ctx, schema.id()).await?;

            schema
        }
        Some(installed_schema_record) => match installed_schema_record.as_installed_schema()? {
            InstalledPkgAssetTyped::Schema { id, .. } => match Schema::get_by_id(ctx, &id).await? {
                Some(schema) => schema,
                None => return Err(PkgError::InstalledSchemaMissing(id)),
            },
            _ => unreachable!(),
        },
    };

    // Even if the asset is already installed, we write a record of the asset installation so that
    // we can track the installed packages that share schemas.
    InstalledPkgAsset::new(
        ctx,
        InstalledPkgAssetTyped::new_for_schema(*schema.id(), installed_pkg_id, hash),
    )
    .await?;

    for variant_spec in schema_spec.variants()? {
        create_schema_variant(ctx, &mut schema, variant_spec, installed_pkg_id).await?;
    }

    Ok(())
}

async fn create_schema_variant(
    ctx: &DalContext,
    schema: &mut Schema,
    variant_spec: SiPkgSchemaVariant<'_>,
    installed_pkg_id: InstalledPkgId,
) -> PkgResult<()> {
    let hash = variant_spec.hash().to_string();
    let existing_schema_variant =
        InstalledPkgAsset::list_for_kind_and_hash(ctx, InstalledPkgAssetKind::SchemaVariant, &hash)
            .await?
            .pop();

    let variant_id = match existing_schema_variant {
        Some(installed_sv_record) => match installed_sv_record.as_installed_schema_variant()? {
            InstalledPkgAssetTyped::SchemaVariant { id, .. } => id,
            _ => unreachable!(),
        },
        None => {
            let (mut schema_variant, root_prop) =
                SchemaVariant::new(ctx, *schema.id(), variant_spec.name()).await?;

            schema
                .set_default_schema_variant_id(ctx, Some(schema_variant.id()))
                .await?;

            let color = match variant_spec.color() {
                Some(color_str) => Some(hex_color_to_i64(color_str)?),
                None => None,
            };
            schema_variant.set_color(ctx, color).await?;

            let domain_prop_id = root_prop.domain_prop_id;
            variant_spec
                .visit_prop_tree(create_prop, Some(domain_prop_id), ctx)
                .await?;

            schema_variant.finalize(ctx, None).await?;
            *schema_variant.id()
        }
    };

    InstalledPkgAsset::new(
        ctx,
        InstalledPkgAssetTyped::new_for_schema_variant(variant_id, installed_pkg_id, hash),
    )
    .await?;

    Ok(())
}

async fn create_prop(
    spec: SiPkgProp<'_>,
    parent_prop_id: Option<PropId>,
    ctx: &DalContext,
) -> Result<Option<PropId>, SiPkgError> {
    let prop = Prop::new(
        ctx,
        spec.name(),
        match spec {
            SiPkgProp::String { .. } => PropKind::String,
            SiPkgProp::Number { .. } => PropKind::Integer,
            SiPkgProp::Boolean { .. } => PropKind::Boolean,
            SiPkgProp::Map { .. } => PropKind::Map,
            SiPkgProp::Array { .. } => PropKind::Array,
            SiPkgProp::Object { .. } => PropKind::Object,
        },
        None,
    )
    .await
    .map_err(SiPkgError::visit_prop)?;

    if let Some(parent_prop_id) = parent_prop_id {
        prop.set_parent_prop(ctx, parent_prop_id)
            .await
            .map_err(SiPkgError::visit_prop)?;
    }

    Ok(Some(*prop.id()))
}