use std::path::Path;

use super::*;

#[test]
fn quotes_the_executable_path_in_the_run_value() {
    assert_eq!(
        run_value(Path::new(r"F:\Apps\Lumi\lumi.exe")),
        r#""F:\Apps\Lumi\lumi.exe""#
    );
}

#[test]
#[ignore = "writes and removes the current user's Lumi Run value"]
fn current_user_run_value_round_trip() {
    let executable = std::env::current_exe().unwrap();
    set_enabled_for_executable(true, &executable).unwrap();
    let value = CURRENT_USER
        .open(RUN_KEY)
        .unwrap()
        .get_string(VALUE_NAME)
        .unwrap();
    assert_eq!(value, run_value(&executable));

    set_enabled_for_executable(false, &executable).unwrap();
    assert!(CURRENT_USER
        .open(RUN_KEY)
        .and_then(|key| key.get_string(VALUE_NAME))
        .is_err());
}
