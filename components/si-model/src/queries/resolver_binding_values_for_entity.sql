SELECT resolver_binding_values.obj        AS resolver_binding_values,
       resolver_binding_values.created_at AS created_at,
       resolver_binding_values.schema_id AS schema_id
FROM resolver_binding_values
WHERE resolver_binding_values.schema_id = si_id_to_primary_key_v1($1)
   OR resolver_binding_values.entity_id = si_id_to_primary_key_v1($2)
ORDER BY schema_id, prop_id, created_at DESC;