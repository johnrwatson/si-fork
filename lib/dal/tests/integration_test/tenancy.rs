use dal::{DalContext, JwtSecretKey, Tenancy, WorkspacePk};
use dal_test::{test, test_harness::billing_account_signup};

#[test]
async fn check_workspace_pk_identical(ctx: &mut DalContext, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let tenancy = Tenancy::new(*nba.workspace.pk());

    let check = tenancy
        .check(ctx.pg_txn(), &tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_pk_mismatched(ctx: &mut DalContext, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let tenancy = Tenancy::new(*nba.workspace.pk());
    let other_tenancy = Tenancy::new(WorkspacePk::NONE);

    let check = tenancy
        .check(ctx.pg_txn(), &other_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let check = other_tenancy
        .check(ctx.pg_txn(), &tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}