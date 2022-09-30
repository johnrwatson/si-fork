use pretty_assertions_sorted::assert_eq;

use dal::func::backend::validation::ValidationKind;
use dal::test::helpers::builtins::{Builtin, SchemaBuiltinsTestHarness};
use dal::{
    DalContext, Edge, ExternalProvider, InternalProvider, StandardModel, SystemId,
    ValidationResolver,
};

use crate::dal::test;

#[test]
async fn aws_region_to_aws_ec2_intelligence(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let ec2_payload = harness
        .create_component(ctx, "server", Builtin::AwsEc2)
        .await;
    let region_payload = harness
        .create_component(ctx, "region", Builtin::AwsRegion)
        .await;

    // Initialize the tail name field.
    region_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/region",
            Some(serde_json::json!["us-east-2"]),
        )
        .await;

    // Ensure setup worked.
    assert_eq!(
        serde_json::json![{
            "domain": {
                "region": "us-east-2"
            },
            "si": {
                "name": "region"
            }
        }], // expected
        region_payload.component_view_properties(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "awsResourceType": "instance",
            },
            "si": {
                "name": "server"
            }
        }], // expected
        ec2_payload.component_view_properties(ctx).await // actual
    );

    // Find the providers we need for connection.
    let region_external_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        region_payload.schema_variant_id,
        "region",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let ec2_explicit_internal_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            ec2_payload.schema_variant_id,
            "region",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");

    // Finally, create the inter component connection.
    Edge::connect_providers_for_components(
        ctx,
        *ec2_explicit_internal_provider.id(),
        ec2_payload.component_id,
        *region_external_provider.id(),
        region_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Ensure the view did not drift.
    assert_eq!(
        serde_json::json![{
            "domain": {
                "region": "us-east-2"
            },
            "si": {
                "name": "region"
            }
        }], // expected
        region_payload.component_view_properties(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "awsResourceType": "instance",
            },
            "si": {
                "name": "server"
            }
        }], // expected
        ec2_payload.component_view_properties(ctx).await // actual
    );

    // Perform update!
    region_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/region",
            Some(serde_json::json!["us-west-2"]),
        )
        .await;

    // Observed that it worked.
    assert_eq!(
        serde_json::json![{
            "domain": {
                "region": "us-west-2"
            },
            "si": {
                "name": "region"
            }
        }], // expected
        region_payload.component_view_properties(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "awsResourceType": "instance",
                "region": "us-west-2"
            },
            "si": {
                "name": "server"
            }
        }], // expected
        ec2_payload.component_view_properties(ctx).await // actual
    );
}

#[test]
async fn aws_region_field_validation(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let region_payload = harness
        .create_component(ctx, "region", Builtin::AwsRegion)
        .await;

    let updated_region_attribute_value_id = region_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/region",
            Some(serde_json::json!["us-poop-1"]),
        )
        .await;

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "region"
            },
            "domain": {
                "region": "us-poop-1"
            }
        }], // actual
        region_payload.component_view_properties(ctx).await // expected
    );

    let validation_statuses =
        ValidationResolver::find_status(ctx, region_payload.component_id, SystemId::NONE)
            .await
            .expect("could not find status for validation(s) of a given component");

    let mut expected_validation_status = None;
    for validation_status in &validation_statuses {
        if validation_status.attribute_value_id == updated_region_attribute_value_id {
            if expected_validation_status.is_some() {
                panic!(
                    "found more than one expected validation status: {:?}",
                    validation_statuses
                );
            }
            expected_validation_status = Some(validation_status.clone());
        }
    }
    let expected_validation_status =
        expected_validation_status.expect("did not find expected validation status");

    let mut found_expected_validation_error = false;
    for validation_error in &expected_validation_status.errors {
        if validation_error.kind == ValidationKind::ValidateStringArray {
            if found_expected_validation_error {
                panic!(
                    "found more than one expected validation error: {:?}",
                    validation_error
                );
            }
            found_expected_validation_error = true;
        }
    }
    assert!(found_expected_validation_error);

    let updated_region_attribute_value_id = region_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/region",
            Some(serde_json::json!["us-east-1"]),
        )
        .await;

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "region"
            },
            "domain": {
                "region": "us-east-1"
            }
        }], // actual
        region_payload.component_view_properties(ctx).await // expected
    );

    // TODO(nick): now, ensure we have the right value! Huzzah.
    let validation_statuses =
        ValidationResolver::find_status(ctx, region_payload.component_id, SystemId::NONE)
            .await
            .expect("could not find status for validation(s) of a given component");

    let mut expected_validation_status = None;
    for validation_status in &validation_statuses {
        if validation_status.attribute_value_id == updated_region_attribute_value_id {
            if expected_validation_status.is_some() {
                panic!(
                    "found more than one expected validation status: {:?}",
                    validation_statuses
                );
            }
            expected_validation_status = Some(validation_status.clone());
        }
    }
    let expected_validation_status =
        expected_validation_status.expect("did not find expected validation status");
    assert!(expected_validation_status.errors.is_empty());
}