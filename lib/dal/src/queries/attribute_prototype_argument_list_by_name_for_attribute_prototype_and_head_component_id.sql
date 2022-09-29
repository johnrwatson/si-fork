/*
    This query groups arguments that belong to an attribute prototype by name. For every argument that shares the same
    name, they will be in the same "array".

    { key: name, value: [argument_with_same_name_1, argument_with_same_name_2] },
    { key: name, value: [argument_that_only_has_this_name] }
 */

SELECT name, array_agg(arguments) as arguments
FROM (SELECT DISTINCT ON (attribute_prototype_arguments.id) attribute_prototype_arguments.id,
                                                            attribute_prototype_arguments.visibility_change_set_pk,
                                                            attribute_prototype_arguments.visibility_deleted_at,
                                                            fa.name                                      AS name,
                                                            row_to_json(attribute_prototype_arguments.*) AS arguments
      FROM attribute_prototype_arguments
               JOIN func_arguments fa on attribute_prototype_arguments.func_argument_id = fa.id
          AND in_tenancy_and_visible_v1($1, $2, fa)
      WHERE in_tenancy_and_visible_v1($1, $2, attribute_prototype_arguments)
        AND attribute_prototype_arguments.attribute_prototype_id = $3
        AND CASE
                WHEN external_provider_id != -1 THEN
                    head_component_id = $4
                ELSE
                    TRUE
          END

      ORDER BY attribute_prototype_arguments.id,
               visibility_change_set_pk DESC,
               visibility_deleted_at DESC NULLS FIRST) as apa_found
GROUP BY name;
