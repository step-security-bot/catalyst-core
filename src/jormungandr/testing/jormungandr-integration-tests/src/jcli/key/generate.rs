use crate::common::jcli_wrapper;
use assert_cmd::assert::OutputAssertExt;

#[test]
pub fn test_ed25519_key_generation() {
    let generated_key = jcli_wrapper::assert_key_generate("ed25519");
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_ed25519_uppercase_key_generation() {
    let generated_key = jcli_wrapper::assert_key_generate("ED25519EXTENDED");
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_ed25510bip32_key_generation() {
    let generated_key = jcli_wrapper::assert_key_generate("Ed25519Bip32");
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_ed25519extended_key_generation() {
    let generated_key = jcli_wrapper::assert_key_generate("Ed25519Extended");
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_curve25519_2hashdh_key_generation() {
    let generated_key = jcli_wrapper::assert_key_generate("Curve25519_2HashDH");
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_sumed25519_12_key_generation() {
    let generated_key = jcli_wrapper::assert_key_generate("SumEd25519_12");
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_unknown_key_type_generation() {
    jcli_wrapper::jcli_commands::get_key_generate_command("unknown")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Invalid value for '--type <key-type>':",
        ));
}

#[test]
pub fn test_key_with_seed_generation() {
    let correct_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcada3e135";
    let generated_key =
        jcli_wrapper::assert_key_with_seed_generate("Ed25519Extended", &correct_seed);
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_key_with_too_short_seed_generation() {
    let too_short_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcadaa";
    test_key_invalid_seed_length(&too_short_seed);
}

#[test]
pub fn test_key_with_too_long_seed_generation() {
    let too_long_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcada0234212";
    test_key_invalid_seed_length(&too_long_seed);
}

fn test_key_invalid_seed_length(seed: &str) {
    jcli_wrapper::jcli_commands::get_key_generate_with_seed_command("Ed25519Extended", &seed)
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "invalid seed length, expected 32 bytes but received",
        ));
}

#[test]
pub fn test_key_with_seed_with_unknown_symbol_generation() {
    let incorrect_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcay";
    jcli_wrapper::jcli_commands::get_key_generate_with_seed_command(
        "Ed25519Extended",
        &incorrect_seed,
    )
    .assert()
    .failure()
    .stderr(predicates::str::contains("invalid Hexadecimal"));
}